use std::{collections::HashMap, ops::Range};

use anyhow::{bail, Context};
use binrw::BinRead;
use clap::Parser;

use mm_file_formats::adf::AdfFile;

mod adf;
use adf::{XvmFormatDebugStrings, XvmFormatFunction, XvmFormatModule, XvmFunctionDebugArray};

mod ssa;
use ssa::{SsaBinaryOperation, SsaBlock, SsaConstant, SsaInstruction, SsaLocal, SsaUnaryOperation};

mod xvm;
use xvm::{XvmControlFlowGraph, XvmInstruction, XvmObject, XvmObjectType, XvmOperation};

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
            let debug_info = xvm
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
                let mut cfg = XvmControlFlowGraph::new(&operations);

                // Create initial block infos
                let mut block_infos: Vec<Option<XvmBlockInfo>> = vec![None; cfg.count];
                let mut stack_frame = XvmStackFrame::new(function);
                // TODO: this isn't quite right, we want to use RPO, and iterate until converged...
                loop {
                    let changed = false;
                    for block in cfg.post_order.iter().rev().skip(1).map(|&x| x as usize) {
                        let block_info =
                            XvmBlockInfo::new(stack_frame, operations.slice(&cfg.ranges[block]));
                        stack_frame = block_info.exit_stack_frame.clone();
                        block_infos[block] = Some(block_info);
                    }
                    if !changed {
                        break;
                    }
                }
                println!("{block_infos:?}");

                // Create a temporary SSA context
                let mut context =
                    SsaContext::new(&module, &constants, &debug_strings, &function, &cfg.targets);

                /*
                 * TODO: We need to figure out which temporaries are actually block arguments
                 *  - We must determine which temporaries are declared in a block, and which come from outside a block
                 *  - If we create an ssa context per block, underflow == argument
                 *  - Problem: we to hot swap temporaries if we do this, not ideal!
                 *  - Might be okay though? So long as we validate stack never drops below local_count + arg_count?
                 *  - This means arg numbers would be inverted compared to stack, but this is... maybe fine?
                 */

                /*
                 * TODO: calculate stack depth per block, and validate program
                 * - Do a pass that simulates push + pop counts
                 * - Main issue: need to know the local stack depth to calculate the outgoing stack depth correctly (as underflowing the stack frame == reading func args)
                 * - We must do this iteratively some how, erroring if the local stack depth is indeterminant
                 */

                // Convert ranges of operations into blocks using control flow graph
                let mut blocks = Vec::with_capacity(cfg.count);
                for block in 0..cfg.count {
                    // Build up instructions
                    let instructions = operations
                        .slice(&cfg.ranges[block])
                        .iter()
                        .map(|&(operation, operand)| context.instruction(operation, operand))
                        .collect();

                    blocks.push(SsaBlock {
                        instructions,
                        predecessors: std::mem::take(&mut cfg.predecessors[block]),
                        successors: std::mem::take(&mut cfg.successors[block]),
                        arguments: vec![], // TODO: this needs implemented lol
                    });
                }

                println!("{:?}", cfg);

                // Debugging
                debug_print(function, &blocks);

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

#[derive(Clone, Debug)]
struct XvmBlockInfo {
    entry_stack_frame: XvmStackFrame,
    exit_stack_frame: XvmStackFrame,
    arg_count: u16,
}

impl XvmBlockInfo {
    pub fn new(stack_frame: XvmStackFrame, operations: &[(XvmOperation, u16)]) -> Self {
        let mut entry_stack_frame = stack_frame.clone();
        let (exit_stack_frame, arg_count) = entry_stack_frame.simulate(operations);
        Self {
            entry_stack_frame,
            exit_stack_frame,
            arg_count,
        }
    }
}

#[derive(Clone, Debug)]
struct XvmStackFrame {
    offset: u16,
    size: u16,
}

impl XvmStackFrame {
    pub fn new(function: &XvmFormatFunction) -> XvmStackFrame {
        XvmStackFrame {
            offset: function.arg_count,
            size: 0,
        }
    }

    fn instruction(&mut self, operation: XvmOperation, operand: u16) -> u16 {
        let (pop_count, push_count) = match operation {
            XvmOperation::BuildList => (operand, 1),
            XvmOperation::Call => (operand + 1, 1),
            XvmOperation::Print => (operand & 0b01111111111, 0),
            XvmOperation::Return => ((operand == 1) as u16, 0),
            _ => (operation.pop_count(), operation.push_count()),
        };

        if pop_count <= self.size {
            self.size -= pop_count;
            self.size += push_count;
            0
        } else if pop_count - self.size <= self.offset {
            let used_args = pop_count - self.size;
            self.size = push_count;
            used_args
        } else {
            panic!("stack underflow");
        }
    }

    pub fn simulate(&mut self, operations: &[(XvmOperation, u16)]) -> (XvmStackFrame, u16) {
        let mut used_args = 0;
        for &(operation, operand) in operations {
            used_args = used_args.max(self.instruction(operation, operand))
        }
        (
            XvmStackFrame {
                offset: self.offset + self.size,
                size: 0,
            },
            used_args,
        )
    }
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
    ) -> Self {
        Self {
            module,
            constants,
            debug_strings,
            args: 0..function.arg_count,
            locals: function.arg_count..(function.arg_count + function.locals_count),
            targets,
            stack: Vec::with_capacity(function.max_stack_depth as usize),
            stack_count: 0,
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

    fn instruction(&mut self, operation: XvmOperation, operand: u16) -> SsaInstruction {
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
            XvmOperation::Jump => SsaInstruction::Jump {
                target: self.target(operand),
            },
            XvmOperation::JumpIfFalse => SsaInstruction::JumpIfFalse {
                cond: self.pop(),
                target: self.target(operand),
            },
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
