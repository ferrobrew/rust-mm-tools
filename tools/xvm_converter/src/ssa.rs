use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct SsaBlock {
    pub instructions: Vec<SsaInstruction>,
    pub predecessors: Vec<u16>,
    pub successors: Vec<u16>,
    pub arguments: Vec<SsaLocal>,
}

// NOTE: local: == SsaLocal, all other fields *used to be* SsaTemporary
#[derive(Debug, Clone, PartialEq)]
pub enum SsaInstruction {
    // Load Operations
    LoadConst {
        dst: SsaLocal,
        value: SsaConstant,
    },
    LoadBool {
        dst: SsaLocal,
        value: bool,
    },
    LoadGlobal {
        dst: SsaLocal,
        name: String,
    },
    LoadLocal {
        // dst = local
        dst: SsaLocal,
        local: SsaLocal,
    },
    LoadAttr {
        // dst = src.name
        src: SsaLocal,
        dst: SsaLocal,
        name: String,
    },
    LoadSubscript {
        // dst = src[idx]
        idx: SsaLocal,
        src: SsaLocal,
        dst: SsaLocal,
    },

    // Store operations
    StoreLocal {
        // local = src
        local: SsaLocal,
        src: SsaLocal,
    },
    StoreAttr {
        // dst.name = src
        dst: SsaLocal,
        src: SsaLocal,
        name: String,
    },
    StoreSubscript {
        // dst[idx] = src
        src: SsaLocal,
        idx: SsaLocal,
        dst: SsaLocal,
    },

    // Arithmetic/Logical Operations
    BinaryOp {
        op: SsaBinaryOperation,
        rhs: SsaLocal,
        lhs: SsaLocal,
        dst: SsaLocal,
    },
    UnaryOp {
        op: SsaUnaryOperation,
        src: SsaLocal,
        dst: SsaLocal,
    },

    // List/Table Operations
    BuildList {
        elements: Vec<SsaLocal>,
        dst: SsaLocal,
    },

    // Function Call & Return
    Call {
        func: SsaLocal,
        args: Vec<SsaLocal>,
        dst: SsaLocal,
    },
    Return {
        value: Option<SsaLocal>,
    },

    // Control Flow
    Jump {
        target: u16,
    },
    JumpIfFalse {
        cond: SsaLocal,
        target: u16,
    },

    // Miscellaneous
    Assert {
        cond: SsaLocal,
    },
    Print {
        new_line: bool,
        values: Vec<SsaLocal>,
    },
    Pop {
        src: SsaLocal,
    },
}

impl Display for SsaInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SsaInstruction::LoadConst { dst, value } => {
                f.write_fmt(format_args!("mov {dst}, {value}"))
            }
            SsaInstruction::LoadBool { dst, value } => {
                f.write_fmt(format_args!("mov {dst}, {value}"))
            }
            SsaInstruction::LoadGlobal { dst, name } => {
                f.write_fmt(format_args!("mov {dst}, {name}"))
            }
            SsaInstruction::LoadLocal { dst, local } => {
                f.write_fmt(format_args!("mov {dst}, {local}"))
            }
            SsaInstruction::LoadAttr { src, dst, name } => {
                f.write_fmt(format_args!("mov {dst}, {src}.{name}"))
            }
            SsaInstruction::LoadSubscript { idx, src, dst } => {
                f.write_fmt(format_args!("mov {dst}, {src}[{idx}]"))
            }
            SsaInstruction::StoreLocal { local, src } => {
                f.write_fmt(format_args!("mov {local}, {src}"))
            }
            SsaInstruction::StoreAttr { dst, src, name } => {
                f.write_fmt(format_args!("mov {dst}.{name}, {src}"))
            }
            SsaInstruction::StoreSubscript { src, idx, dst } => {
                f.write_fmt(format_args!("mov {dst}[{idx}], {src}"))
            }
            SsaInstruction::BinaryOp { op, rhs, lhs, dst } => {
                match op {
                    SsaBinaryOperation::And => f.write_str("and"),
                    SsaBinaryOperation::Or => f.write_str("or"),
                    SsaBinaryOperation::Addition => f.write_str("add"),
                    SsaBinaryOperation::Division => f.write_str("div"),
                    SsaBinaryOperation::Modulo => f.write_str("mod"),
                    SsaBinaryOperation::Multiply => f.write_str("mul"),
                    SsaBinaryOperation::Subtract => f.write_str("sub"),
                    SsaBinaryOperation::CompareEqual => f.write_str("ce"),
                    SsaBinaryOperation::CompareGreaterThanEqual => f.write_str("cge"),
                    SsaBinaryOperation::CompareGreaterThan => f.write_str("cgt"),
                    SsaBinaryOperation::CompareNotEqual => f.write_str("cne"),
                }?;
                f.write_fmt(format_args!(" {dst}, {lhs}, {rhs}"))
            }
            SsaInstruction::UnaryOp { op, src, dst } => {
                match op {
                    SsaUnaryOperation::Not => f.write_str("not "),
                    SsaUnaryOperation::Negate => f.write_str("or "),
                }?;
                f.write_fmt(format_args!("{dst}, {src}"))
            }
            SsaInstruction::BuildList { elements, dst } => {
                f.write_fmt(format_args!("mov {dst}, ("))?;
                for (i, element) in elements.iter().enumerate() {
                    f.write_fmt(format_args!("{}", element))?;
                    if i < elements.len() - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_str(")")
            }
            SsaInstruction::Call { func, args, dst } => {
                f.write_fmt(format_args!("mov {dst}, {func}("))?;
                for (i, argument) in args.iter().enumerate() {
                    f.write_fmt(format_args!("{}", argument))?;
                    if i < args.len() - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_str(")")
            }
            SsaInstruction::Return { value } => match value {
                Some(value) => f.write_fmt(format_args!("ret {value}")),
                None => f.write_fmt(format_args!("ret")),
            },
            SsaInstruction::Jump { target } => f.write_fmt(format_args!("jmp LABEL{target}")),
            SsaInstruction::JumpIfFalse { cond, target } => {
                f.write_fmt(format_args!("jz {cond}, LABEL{target}"))
            }
            SsaInstruction::Assert { cond } => f.write_fmt(format_args!("assert {cond}")),
            SsaInstruction::Print { new_line, values } => {
                f.write_fmt(format_args!("print '"))?;
                for (i, value) in values.iter().enumerate() {
                    f.write_fmt(format_args!("{{{}}}", value))?;
                    if i < values.len() - 1 {
                        f.write_str(" ")?;
                    }
                }
                if *new_line {
                    f.write_str("\\n")?;
                }
                f.write_str("'")
            }
            SsaInstruction::Pop { src } => f.write_fmt(format_args!("pop {src}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SsaConstant {
    None,
    String(String),
    StringHash(u32),
    Float(f32),
}

impl Display for SsaConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SsaConstant::None => f.write_str("NONE"),
            SsaConstant::String(str) => f.write_fmt(format_args!("\"{str}\"")),
            SsaConstant::StringHash(str) => f.write_fmt(format_args!("0x{str:08X?}")),
            SsaConstant::Float(flt) => f.write_fmt(format_args!("{flt}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SsaLocal {
    Argument(u16),
    Local(u16),
}

impl Display for SsaLocal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SsaLocal::Argument(i) => f.write_fmt(format_args!("ARG{i}")),
            SsaLocal::Local(i) => f.write_fmt(format_args!("LOC{i}")),
        }
    }
}

// TODO: should we bring this back?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SsaTemporary(pub u16);

impl Display for SsaTemporary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("t{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsaBinaryOperation {
    And,
    Or,
    Addition,
    Division,
    Modulo,
    Multiply,
    Subtract,
    CompareEqual,
    CompareGreaterThanEqual,
    CompareGreaterThan,
    CompareNotEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsaUnaryOperation {
    Not,
    Negate,
}
