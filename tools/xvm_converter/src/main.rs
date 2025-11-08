use std::{collections::HashMap, ops::Range};

use anyhow::{bail, Context};
use binrw::BinRead;
use clap::Parser;

use mm_file_formats::adf::AdfFile;

mod adf;
use adf::{XvmFormatDebugStrings, XvmFormatFunction, XvmFormatModule, XvmFunctionDebugArray};

mod ssa;
use ssa::{SsaBinaryOperation, SsaBlock, SsaConstant, SsaInstruction, SsaLocal, SsaUnaryOperation};

mod ast;
use ast::{AstExpression, AstFunction, AstLValue, AstStatement};

mod xvm;
use xvm::{XvmControlFlowGraph, XvmInstruction, XvmObject, XvmObjectType, XvmOperation};

#[derive(Clone)]
struct BlockStackInfo {
    /// Number of stack arguments this block expects
    arg_count: u16,
    /// Stack depth at block entry (including args)
    entry_depth: u16,
    /// Stack depth at block exit
    exit_depth: u16,
}

fn calculate_block_stack_depths(
    _function: &XvmFormatFunction,
    operations: &[(XvmOperation, u16)],
    cfg: &XvmControlFlowGraph,
) -> Vec<BlockStackInfo> {
    let mut block_infos: Vec<Option<BlockStackInfo>> = vec![None; cfg.count];

    // Entry block starts with zero stack depth (only function arguments available)
    block_infos[0] = Some(BlockStackInfo {
        arg_count: 0,
        entry_depth: 0,
        exit_depth: 0,
    });

    // Fixed-point iteration to determine stack depths
    let mut changed = true;
    while changed {
        changed = false;

        for &block_idx in cfg.post_order.iter().rev() {
            let block_idx_usize = block_idx as usize;

            // Skip if we don't have info for any predecessor yet
            if block_idx != 0 && cfg.predecessors[block_idx_usize].is_empty() {
                continue;
            }

            // Determine entry depth from predecessors
            let entry_depth = if block_idx == 0 {
                0
            } else {
                // Get exit depth from any predecessor (they should all match)
                let mut pred_exit_depth = None;
                for &pred_idx in &cfg.predecessors[block_idx_usize] {
                    if let Some(pred_info) = &block_infos[pred_idx as usize] {
                        pred_exit_depth = Some(pred_info.exit_depth);
                        break;
                    }
                }
                pred_exit_depth.unwrap_or(0)
            };

            // Simulate the block to find exit depth
            let mut stack_depth = entry_depth;
            let range = &cfg.ranges[block_idx_usize];
            let block_ops = &operations[range.start as usize..range.end as usize];

            for &(operation, operand) in block_ops {
                let (pop_count, push_count) = match operation {
                    XvmOperation::BuildList => (operand, 1),
                    XvmOperation::Call => (operand + 1, 1),
                    XvmOperation::Print => (operand & 0b01111111111, 0),
                    XvmOperation::Return => ((operand == 1) as u16, 0),
                    _ => (operation.pop_count(), operation.push_count()),
                };

                stack_depth = stack_depth.saturating_sub(pop_count) + push_count;
            }

            let new_info = BlockStackInfo {
                arg_count: entry_depth,
                entry_depth,
                exit_depth: stack_depth,
            };

            if block_infos[block_idx_usize].as_ref() != Some(&new_info) {
                block_infos[block_idx_usize] = Some(new_info);
                changed = true;
            }
        }
    }

    block_infos.into_iter().map(|info| info.unwrap()).collect()
}

fn convert_to_ssa_blocks(
    module: &XvmFormatModule,
    constants: &[(XvmObject, u64)],
    debug_strings: &XvmFormatDebugStrings,
    function: &XvmFormatFunction,
    operations: &[(XvmOperation, u16)],
    cfg: &XvmControlFlowGraph,
    block_stack_info: &[BlockStackInfo],
) -> Vec<SsaBlock> {
    let mut blocks = Vec::with_capacity(cfg.count);

    for block_idx in 0..cfg.count {
        let mut context = SsaContext::new(
            module,
            constants,
            debug_strings,
            function,
            &cfg.targets,
            block_stack_info[block_idx].arg_count,
        );

        // Convert instructions
        let range = &cfg.ranges[block_idx];
        let block_ops = &operations[range.start as usize..range.end as usize];
        let mut instructions = Vec::with_capacity(block_ops.len());

        for (i, &(operation, operand)) in block_ops.iter().enumerate() {
            let is_last = i == block_ops.len() - 1;
            instructions.push(context.instruction(operation, operand, is_last));
        }

        // Create block arguments (these are the stack values passed from predecessors)
        let mut arguments = Vec::new();
        for i in 0..block_stack_info[block_idx].arg_count {
            arguments.push(SsaLocal::Argument(i));
        }

        blocks.push(SsaBlock {
            instructions,
            predecessors: cfg.predecessors[block_idx].clone(),
            successors: cfg.successors[block_idx].clone(),
            arguments,
        });
    }

    blocks
}

impl PartialEq for BlockStackInfo {
    fn eq(&self, other: &Self) -> bool {
        self.arg_count == other.arg_count
            && self.entry_depth == other.entry_depth
            && self.exit_depth == other.exit_depth
    }
}

fn convert_ssa_to_ast(
    function: &XvmFormatFunction,
    blocks: &[SsaBlock],
    cfg: &XvmControlFlowGraph,
) -> AstFunction {
    let mut converter = SsaToAstConverter::new(function);
    converter.convert(blocks, cfg)
}

struct SsaToAstConverter {
    function_name: String,
    arg_count: u16,
    var_names: std::collections::HashMap<SsaLocal, String>,
}

impl SsaToAstConverter {
    fn new(function: &XvmFormatFunction) -> Self {
        Self {
            function_name: String::from_utf8_lossy(&function.name[0..function.name.len() - 1])
                .into(),
            arg_count: function.arg_count,
            var_names: std::collections::HashMap::new(),
        }
    }

    fn get_var_name(&mut self, local: &SsaLocal) -> String {
        if let Some(name) = self.var_names.get(local) {
            return name.clone();
        }

        let name = match local {
            SsaLocal::Argument(n) => {
                // Arguments are reversed in the bytecode
                let arg_idx = self.arg_count - 1 - n;
                format!("arg{}", arg_idx)
            }
            SsaLocal::Local(n) => format!("t{}", n),
        };

        self.var_names.insert(*local, name.clone());
        name
    }

    fn convert_constant(&self, constant: &SsaConstant) -> AstExpression {
        match constant {
            SsaConstant::None => AstExpression::None,
            SsaConstant::Float(f) => AstExpression::Float(*f),
            SsaConstant::String(s) => AstExpression::String(s.clone()),
            SsaConstant::StringHash(h) => AstExpression::StringHash(*h),
        }
    }

    fn convert_local_to_expr(&mut self, local: &SsaLocal) -> AstExpression {
        let name = self.get_var_name(local);
        AstExpression::Variable { name }
    }

    fn convert_instruction_to_expr(&mut self, instr: &SsaInstruction) -> Option<AstExpression> {
        match instr {
            SsaInstruction::LoadConst { value, .. } => Some(self.convert_constant(value)),
            SsaInstruction::LoadBool { value, .. } => Some(AstExpression::Bool(*value)),
            SsaInstruction::LoadGlobal { name, .. } => {
                Some(AstExpression::Global { name: name.clone() })
            }
            SsaInstruction::LoadLocal { local, .. } => Some(self.convert_local_to_expr(local)),
            SsaInstruction::LoadAttr { src, name, .. } => Some(AstExpression::Attr {
                obj: Box::new(self.convert_local_to_expr(src)),
                name: name.clone(),
            }),
            SsaInstruction::LoadSubscript { idx, src, .. } => Some(AstExpression::Subscript {
                obj: Box::new(self.convert_local_to_expr(src)),
                index: Box::new(self.convert_local_to_expr(idx)),
            }),
            SsaInstruction::BinaryOp { op, lhs, rhs, .. } => Some(AstExpression::BinaryOp {
                op: *op,
                lhs: Box::new(self.convert_local_to_expr(lhs)),
                rhs: Box::new(self.convert_local_to_expr(rhs)),
            }),
            SsaInstruction::UnaryOp { op, src, .. } => Some(AstExpression::UnaryOp {
                op: *op,
                src: Box::new(self.convert_local_to_expr(src)),
            }),
            SsaInstruction::BuildList { elements, .. } => {
                let items = elements
                    .iter()
                    .map(|e| self.convert_local_to_expr(e))
                    .collect();
                Some(AstExpression::List(items))
            }
            SsaInstruction::Call { func, args, .. } => {
                let func_expr = Box::new(self.convert_local_to_expr(func));
                let arg_exprs = args.iter().map(|a| self.convert_local_to_expr(a)).collect();
                Some(AstExpression::Call {
                    func: func_expr,
                    args: arg_exprs,
                })
            }
            _ => None,
        }
    }

    fn convert_instruction_to_stmt(&mut self, instr: &SsaInstruction) -> Option<AstStatement> {
        match instr {
            SsaInstruction::LoadConst { dst, .. }
            | SsaInstruction::LoadBool { dst, .. }
            | SsaInstruction::LoadGlobal { dst, .. }
            | SsaInstruction::LoadLocal { dst, .. }
            | SsaInstruction::LoadAttr { dst, .. }
            | SsaInstruction::LoadSubscript { dst, .. }
            | SsaInstruction::BinaryOp { dst, .. }
            | SsaInstruction::UnaryOp { dst, .. }
            | SsaInstruction::BuildList { dst, .. }
            | SsaInstruction::Call { dst, .. } => {
                if let Some(expr) = self.convert_instruction_to_expr(instr) {
                    let var_name = self.get_var_name(dst);
                    Some(AstStatement::Assign {
                        target: AstLValue::Variable { name: var_name },
                        value: Box::new(expr),
                    })
                } else {
                    None
                }
            }
            SsaInstruction::StoreLocal { local, src } => {
                let var_name = self.get_var_name(local);
                let expr = self.convert_local_to_expr(src);
                Some(AstStatement::Assign {
                    target: AstLValue::Variable { name: var_name },
                    value: Box::new(expr),
                })
            }
            SsaInstruction::StoreAttr { dst, src, name } => {
                let obj = self.convert_local_to_expr(dst);
                let value = self.convert_local_to_expr(src);
                Some(AstStatement::Assign {
                    target: AstLValue::Attr {
                        obj: Box::new(obj),
                        name: name.clone(),
                    },
                    value: Box::new(value),
                })
            }
            SsaInstruction::StoreSubscript { dst, idx, src } => {
                let obj = self.convert_local_to_expr(dst);
                let index = self.convert_local_to_expr(idx);
                let value = self.convert_local_to_expr(src);
                Some(AstStatement::Assign {
                    target: AstLValue::Subscript {
                        obj: Box::new(obj),
                        index: Box::new(index),
                    },
                    value: Box::new(value),
                })
            }
            SsaInstruction::Return { value } => {
                let ret_value = value.as_ref().map(|v| Box::new(self.convert_local_to_expr(v)));
                Some(AstStatement::Return { value: ret_value })
            }
            SsaInstruction::Assert { cond } => {
                let cond_expr = self.convert_local_to_expr(cond);
                Some(AstStatement::Assert {
                    cond: Box::new(cond_expr),
                })
            }
            SsaInstruction::Print { new_line, values } => {
                let value_exprs = values
                    .iter()
                    .map(|v| self.convert_local_to_expr(v))
                    .collect();
                Some(AstStatement::Print {
                    new_line: *new_line,
                    values: value_exprs,
                })
            }
            SsaInstruction::Pop { .. } => None, // Ignore pop instructions
            SsaInstruction::Jump { .. } | SsaInstruction::JumpIfFalse { .. } => {
                None // Control flow handled separately
            }
        }
    }

    fn convert(&mut self, blocks: &[SsaBlock], cfg: &XvmControlFlowGraph) -> AstFunction {
        // Generate parameter names
        let mut params = Vec::new();
        for i in 0..self.arg_count {
            params.push(format!("arg{}", i));
        }

        // Convert blocks to statements (simple linear conversion for now)
        let mut body = Vec::new();
        let visited = self.convert_blocks_to_statements(blocks, cfg, 0, &mut std::collections::HashSet::new());
        body.extend(visited);

        AstFunction {
            name: self.function_name.clone(),
            params,
            body,
        }
    }

    fn convert_blocks_to_statements(
        &mut self,
        blocks: &[SsaBlock],
        cfg: &XvmControlFlowGraph,
        block_idx: u16,
        visited: &mut std::collections::HashSet<u16>,
    ) -> Vec<AstStatement> {
        if visited.contains(&block_idx) {
            return vec![];
        }
        visited.insert(block_idx);

        let block = &blocks[block_idx as usize];
        let mut statements = Vec::new();

        // Convert instructions to statements
        for instr in &block.instructions {
            // Check if this is a control flow instruction
            match instr {
                SsaInstruction::Jump { target, .. } => {
                    // Simple unconditional jump - just continue with the target
                    let target_stmts = self.convert_blocks_to_statements(blocks, cfg, *target, visited);
                    statements.extend(target_stmts);
                }
                SsaInstruction::JumpIfFalse { cond, target, .. } => {
                    // This is a conditional branch
                    let cond_expr = self.convert_local_to_expr(cond);

                    // Find the fall-through block (next block in sequence)
                    let fall_through = block_idx + 1;

                    // Convert both branches
                    let else_stmts = self.convert_blocks_to_statements(blocks, cfg, *target, visited);
                    let then_stmts = if (fall_through as usize) < blocks.len() {
                        self.convert_blocks_to_statements(blocks, cfg, fall_through, visited)
                    } else {
                        vec![]
                    };

                    statements.push(AstStatement::If {
                        cond: Box::new(cond_expr),
                        then_block: then_stmts,
                        else_block: if else_stmts.is_empty() {
                            None
                        } else {
                            Some(else_stmts)
                        },
                    });
                }
                _ => {
                    if let Some(stmt) = self.convert_instruction_to_stmt(instr) {
                        statements.push(stmt);
                    }
                }
            }
        }

        statements
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !args.file.is_file() {
        bail!("{:?} is not a file", args.file);
    }

    let extension = args
        .file
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .context("Failed to determine file extension")?;

    // Open the file
    let file = std::fs::File::open(args.file.clone()).context("Failed to open file")?;
    let mut reader = std::io::BufReader::new(file);

    match extension {
        "xvmc" => {
            // Parse the ADF
            let xvm = AdfFile::read_le(&mut reader).context("Failed to parse ADF")?;

            // Find associated ADF instances
            let module = xvm
                .get_instance_by_info::<XvmFormatModule>("module")
                .context("failed to find `module` instance")?
                .read::<XvmFormatModule>()?;
            let _debug_info = xvm
                .get_instance_by_info::<XvmFunctionDebugArray>("debug_info")
                .context("failed to find `debug_info` instance")?
                .read::<XvmFunctionDebugArray>()?;
            let debug_strings = xvm
                .get_instance_by_info::<XvmFormatDebugStrings>("debug_strings")
                .context("failed to find `debug_strings` instance")?
                .read::<XvmFormatDebugStrings>()?;

            // Convert instances to objects
            let constants: Vec<(XvmObject, u64)> = module
                .constants
                .iter()
                .map(|constant| (XvmObject::from(constant.flags), constant.value))
                .collect();

            // Disassemble
            for function in module.functions.iter() {
                // TODO: remove debug code
                // if !function.name.ends_with(b"IsVehicleIncapacitaded\0") {
                //     continue;
                // }
                // Decode instructions into operations
                let operations: Vec<(XvmOperation, u16)> = function
                    .instructions
                    .iter()
                    .map(XvmInstruction::from)
                    .map(|instruction| (instruction.operation(), instruction.operand()))
                    .collect();

                // Create control flow graph
                let cfg = XvmControlFlowGraph::new(&operations);

                // Calculate block stack depths using fixed-point iteration
                let block_stack_info = calculate_block_stack_depths(function, &operations, &cfg);

                // Convert to SSA with block arguments
                let blocks = convert_to_ssa_blocks(
                    &module,
                    &constants,
                    &debug_strings,
                    function,
                    &operations,
                    &cfg,
                    &block_stack_info,
                );

                // Debug: print CFG
                // println!("{:?}", cfg);

                // Debug: print SSA
                // debug_print(function, &blocks);

                // Convert SSA to AST
                let ast = convert_ssa_to_ast(function, &blocks, &cfg);

                // Print AST
                println!("{}\n", ast);

                // TODO:
                // - is it possible to overwrite args...? seem likely?
                // - insert load argument instructions... think not :)
                // - insert Phi blocks during control flow analysis... block arguments > phi
                // - replace load/store local during mem2reg pass
                // - how do we model lists in SSA, and what about properties...?
                // - identify higher level constructs (for, while, do while, if, else)
                // - emit reasonable AST representation
            }
        }
        extension => {
            bail!("This tool does not support the '{extension}' extension");
        }
    }

    Ok(())
}

#[derive(Parser)]
struct Args {
    #[arg()]
    file: std::path::PathBuf,
}

trait RangeSlice {
    type Type;

    fn slice(&self, range: &Range<u16>) -> &[Self::Type];
}

impl<T> RangeSlice for Vec<T> {
    type Type = T;

    fn slice(&self, range: &Range<u16>) -> &[Self::Type] {
        &self[range.start as usize..range.end as usize]
    }
}

struct SsaContext<'a> {
    module: &'a XvmFormatModule,
    constants: &'a [(XvmObject, u64)],
    debug_strings: &'a XvmFormatDebugStrings,
    args: Range<u16>,
    locals: Range<u16>,
    targets: &'a HashMap<u16, u16>,
    stack: Vec<SsaLocal>,
    stack_count: u16,
}

impl<'a> SsaContext<'a> {
    fn new(
        module: &'a XvmFormatModule,
        constants: &'a [(XvmObject, u64)],
        debug_strings: &'a XvmFormatDebugStrings,
        function: &XvmFormatFunction,
        targets: &'a HashMap<u16, u16>,
        block_arg_count: u16,
    ) -> Self {
        let mut stack = Vec::with_capacity(function.max_stack_depth as usize);

        // Initialize stack with block arguments
        for i in 0..block_arg_count {
            stack.push(SsaLocal::Argument(i));
        }

        Self {
            module,
            constants,
            debug_strings,
            args: 0..function.arg_count,
            locals: function.arg_count..(function.arg_count + function.locals_count),
            targets,
            stack,
            stack_count: block_arg_count,
        }
    }

    fn pop(&mut self) -> SsaLocal {
        self.stack.pop().unwrap_or(SsaLocal::Argument(0))
    }

    fn push(&mut self) -> SsaLocal {
        let result = SsaLocal::Local(self.stack_count);
        self.stack.push(result);
        self.stack_count += 1;
        result
    }

    fn list(&mut self, count: u16) -> Vec<SsaLocal> {
        let mut result: Vec<SsaLocal> = (0..count).map(|_| self.pop()).collect();
        result.reverse();
        result
    }

    fn local(&self, local: u16) -> SsaLocal {
        if self.args.contains(&local) {
            SsaLocal::Argument(local)
        } else if self.locals.contains(&local) {
            SsaLocal::Local(local - self.args.end)
        } else {
            unreachable!("invalid operand");
        }
    }

    fn target(&self, target: u16) -> u16 {
        *self.targets.get(&target).expect("invalid operand")
    }

    fn name(&self, constant: u16) -> String {
        let (object, value) = self
            .constants
            .get(constant as usize)
            .expect("Invalid operand");
        match object.object_type() {
            XvmObjectType::String => {
                let start = usize::from(u16::from_le_bytes([
                    self.module.string_buffer[*value as usize - 1],
                    self.module.string_buffer[*value as usize - 2],
                ]));
                let mut end = start;
                while self.debug_strings.string_buffer_debug[end] != 0 {
                    end += 1;
                }
                let buffer = &self.debug_strings.string_buffer_debug[start..end];
                String::from_utf8_lossy(buffer).into()
            }
            _ => unreachable!(),
        }
    }

    fn constant(&self, constant: u16) -> SsaConstant {
        let (object, value) = self
            .constants
            .get(constant as usize)
            .expect("Invalid operand");
        match object.object_type() {
            XvmObjectType::None => SsaConstant::None,
            XvmObjectType::Float => SsaConstant::Float(f32::from_bits(*value as u32)),
            XvmObjectType::String => {
                let start = *value as usize;
                let end = start + object.size() as usize;
                let buffer = &self.module.string_buffer[start..end];
                // Some hashes may actually be valid as ASCII. Unfortunately there's no way to know for sure.
                let is_valid_ascii = |c: &u8| c.is_ascii_alphanumeric() || *c == 20;
                if buffer.len() != 4 || buffer.iter().all(is_valid_ascii) {
                    SsaConstant::String(String::from_utf8_lossy(buffer).into())
                } else {
                    SsaConstant::StringHash(u32::from_le_bytes([
                        buffer[0], buffer[1], buffer[2], buffer[3],
                    ]))
                }
            }
            _ => unreachable!(),
        }
    }

    fn get_jump_args(&self) -> Vec<SsaLocal> {
        self.stack.clone()
    }

    fn instruction(&mut self, operation: XvmOperation, operand: u16, is_last: bool) -> SsaInstruction {
        match operation {
            XvmOperation::Assert => SsaInstruction::Assert { cond: self.pop() },
            XvmOperation::BinaryAnd
            | XvmOperation::BinaryOr
            | XvmOperation::BinaryAddition
            | XvmOperation::BinaryDivision
            | XvmOperation::BinaryModulo
            | XvmOperation::BinaryMultiply
            | XvmOperation::BinarySubtract
            | XvmOperation::CompareEqual
            | XvmOperation::CompareGreaterThanEqual
            | XvmOperation::CompareGreaterThan
            | XvmOperation::CompareNotEqual => SsaInstruction::BinaryOp {
                op: to_binary(operation),
                rhs: self.pop(),
                lhs: self.pop(),
                dst: self.push(),
            },
            XvmOperation::BuildList => SsaInstruction::BuildList {
                elements: self.list(operand),
                dst: self.push(),
            },
            XvmOperation::Call => SsaInstruction::Call {
                func: self.pop(),
                args: self.list(operand),
                dst: self.push(),
            },
            XvmOperation::Jump => {
                let args = if is_last { self.get_jump_args() } else { vec![] };
                SsaInstruction::Jump {
                    target: self.target(operand),
                    args,
                }
            }
            XvmOperation::JumpIfFalse => {
                let cond = self.pop();
                let args = if is_last { self.get_jump_args() } else { vec![] };
                SsaInstruction::JumpIfFalse {
                    cond,
                    target: self.target(operand),
                    args,
                }
            }
            XvmOperation::LoadAttr => SsaInstruction::LoadAttr {
                src: self.pop(),
                dst: self.push(),
                name: self.name(operand),
            },
            XvmOperation::LoadConst => SsaInstruction::LoadConst {
                dst: self.push(),
                value: self.constant(operand),
            },
            XvmOperation::LoadBool => SsaInstruction::LoadBool {
                dst: self.push(),
                value: operand == 1,
            },
            XvmOperation::LoadGlobal => SsaInstruction::LoadGlobal {
                dst: self.push(),
                name: self.name(operand),
            },
            XvmOperation::LoadLocal => SsaInstruction::LoadLocal {
                dst: self.push(),
                local: self.local(operand),
            },
            XvmOperation::LoadSubscript => SsaInstruction::LoadSubscript {
                idx: self.pop(),
                src: self.pop(),
                dst: self.push(),
            },
            XvmOperation::Pop => SsaInstruction::Pop { src: self.pop() },
            XvmOperation::Print => SsaInstruction::Print {
                new_line: (operand & 0b10000000000) != 0,
                values: self.list(operand & 0b01111111111),
            },
            XvmOperation::Return => SsaInstruction::Return {
                value: (operand == 1).then(|| self.pop()),
            },
            XvmOperation::StoreAttr => SsaInstruction::StoreAttr {
                dst: self.pop(),
                src: self.pop(),
                name: self.name(operand),
            },
            XvmOperation::StoreLocal => SsaInstruction::StoreLocal {
                local: self.local(operand),
                src: self.pop(),
            },
            XvmOperation::StoreSubscript => SsaInstruction::StoreSubscript {
                src: self.pop(),
                idx: self.pop(),
                dst: self.pop(),
            },
            XvmOperation::UnaryNot | XvmOperation::UnaryNegate => SsaInstruction::UnaryOp {
                op: to_unary(operation),
                src: self.pop(),
                dst: self.push(),
            },
        }
    }
}

fn to_binary(operation: XvmOperation) -> SsaBinaryOperation {
    match operation {
        XvmOperation::BinaryAnd => SsaBinaryOperation::And,
        XvmOperation::BinaryOr => SsaBinaryOperation::Or,
        XvmOperation::BinaryAddition => SsaBinaryOperation::Addition,
        XvmOperation::BinaryDivision => SsaBinaryOperation::Division,
        XvmOperation::BinaryModulo => SsaBinaryOperation::Modulo,
        XvmOperation::BinaryMultiply => SsaBinaryOperation::Multiply,
        XvmOperation::BinarySubtract => SsaBinaryOperation::Subtract,
        XvmOperation::CompareEqual => SsaBinaryOperation::CompareEqual,
        XvmOperation::CompareGreaterThanEqual => SsaBinaryOperation::CompareGreaterThanEqual,
        XvmOperation::CompareGreaterThan => SsaBinaryOperation::CompareGreaterThan,
        XvmOperation::CompareNotEqual => SsaBinaryOperation::CompareNotEqual,
        _ => unreachable!(),
    }
}

fn to_unary(operation: XvmOperation) -> SsaUnaryOperation {
    match operation {
        XvmOperation::UnaryNot => SsaUnaryOperation::Not,
        XvmOperation::UnaryNegate => SsaUnaryOperation::Negate,
        _ => unreachable!(),
    }
}

fn debug_print(function: &XvmFormatFunction, blocks: &[SsaBlock]) {
    // Signature
    print!(
        "{}(",
        String::from_utf8_lossy(&function.name[0..function.name.len() - 1])
    );
    for i in (0..function.arg_count).rev() {
        print!("ARG{}", i);
        if i > 0 {
            print!(", ");
        }
    }
    println!("):");

    for (index, block) in blocks.iter().enumerate() {
        // Labels
        if !block.predecessors.is_empty() {
            // First block has any predecessors
            if index == 0 {
                println!("LABEL{index}:");
            }
            // More than one predecessor
            else if block.predecessors.len() > 1 {
                println!("LABEL{index}:");
            }
            // Single predecessors that is not fall through
            else if block.predecessors[0] != (index - 1) as u16 {
                println!("LABEL{index}:");
            }
        }

        // Disassembly
        for instruction in &block.instructions {
            println!("  {instruction}");
        }
    }
}
