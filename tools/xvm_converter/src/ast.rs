use std::fmt::Display;

use crate::ssa::{SsaBinaryOperation, SsaUnaryOperation};

/// AST representation of the scripting language
#[derive(Debug, Clone)]
pub struct AstFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<AstStatement>,
}

#[derive(Debug, Clone)]
pub enum AstStatement {
    // Variable declaration/assignment
    Assign {
        target: AstLValue,
        value: Box<AstExpression>,
    },

    // Control flow
    If {
        cond: Box<AstExpression>,
        then_block: Vec<AstStatement>,
        else_block: Option<Vec<AstStatement>>,
    },
    #[allow(dead_code)]
    While {
        cond: Box<AstExpression>,
        body: Vec<AstStatement>,
    },
    Return {
        value: Option<Box<AstExpression>>,
    },

    // Expression statement (function call, etc.)
    #[allow(dead_code)]
    Expression {
        expr: Box<AstExpression>,
    },

    // Miscellaneous
    Assert {
        cond: Box<AstExpression>,
    },
    Print {
        new_line: bool,
        values: Vec<AstExpression>,
    },
}

#[derive(Debug, Clone)]
pub enum AstLValue {
    Variable { name: String },
    Attr { obj: Box<AstExpression>, name: String },
    Subscript { obj: Box<AstExpression>, index: Box<AstExpression> },
}

#[derive(Debug, Clone)]
pub enum AstExpression {
    // Literals
    None,
    Bool(bool),
    Float(f32),
    String(String),
    StringHash(u32),
    List(Vec<AstExpression>),

    // Variables
    Variable { name: String },
    Global { name: String },

    // Operations
    BinaryOp {
        op: SsaBinaryOperation,
        lhs: Box<AstExpression>,
        rhs: Box<AstExpression>,
    },
    UnaryOp {
        op: SsaUnaryOperation,
        src: Box<AstExpression>,
    },

    // Access
    Attr {
        obj: Box<AstExpression>,
        name: String,
    },
    Subscript {
        obj: Box<AstExpression>,
        index: Box<AstExpression>,
    },

    // Function call
    Call {
        func: Box<AstExpression>,
        args: Vec<AstExpression>,
    },
}

impl Display for AstFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("fn {}(", self.name))?;
        for (i, param) in self.params.iter().enumerate() {
            f.write_str(param)?;
            if i < self.params.len() - 1 {
                f.write_str(", ")?;
            }
        }
        f.write_str(") {\n")?;

        for stmt in &self.body {
            f.write_fmt(format_args!("  {}\n", stmt))?;
        }

        f.write_str("}")
    }
}

impl Display for AstStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstStatement::Assign { target, value } => {
                f.write_fmt(format_args!("{} = {};", target, value))
            }
            AstStatement::If { cond, then_block, else_block } => {
                f.write_fmt(format_args!("if ({}) {{\n", cond))?;
                for stmt in then_block {
                    f.write_fmt(format_args!("    {}\n", stmt))?;
                }
                if let Some(else_stmts) = else_block {
                    f.write_str("  } else {\n")?;
                    for stmt in else_stmts {
                        f.write_fmt(format_args!("    {}\n", stmt))?;
                    }
                }
                f.write_str("  }")
            }
            AstStatement::While { cond, body } => {
                f.write_fmt(format_args!("while ({}) {{\n", cond))?;
                for stmt in body {
                    f.write_fmt(format_args!("    {}\n", stmt))?;
                }
                f.write_str("  }")
            }
            AstStatement::Return { value } => match value {
                Some(v) => f.write_fmt(format_args!("return {};", v)),
                None => f.write_str("return;"),
            },
            AstStatement::Expression { expr } => f.write_fmt(format_args!("{};", expr)),
            AstStatement::Assert { cond } => f.write_fmt(format_args!("assert({});", cond)),
            AstStatement::Print { new_line, values } => {
                f.write_str("print(")?;
                for (i, val) in values.iter().enumerate() {
                    f.write_fmt(format_args!("{}", val))?;
                    if i < values.len() - 1 {
                        f.write_str(", ")?;
                    }
                }
                if *new_line {
                    f.write_str(", \"\\n\"")?;
                }
                f.write_str(");")
            }
        }
    }
}

impl Display for AstLValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstLValue::Variable { name } => f.write_str(name),
            AstLValue::Attr { obj, name } => f.write_fmt(format_args!("{}.{}", obj, name)),
            AstLValue::Subscript { obj, index } => {
                f.write_fmt(format_args!("{}[{}]", obj, index))
            }
        }
    }
}

impl Display for AstExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstExpression::None => f.write_str("None"),
            AstExpression::Bool(b) => f.write_fmt(format_args!("{}", b)),
            AstExpression::Float(fl) => f.write_fmt(format_args!("{}", fl)),
            AstExpression::String(s) => f.write_fmt(format_args!("\"{}\"", s)),
            AstExpression::StringHash(h) => f.write_fmt(format_args!("0x{:08X}", h)),
            AstExpression::List(items) => {
                f.write_str("[")?;
                for (i, item) in items.iter().enumerate() {
                    f.write_fmt(format_args!("{}", item))?;
                    if i < items.len() - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_str("]")
            }
            AstExpression::Variable { name } => f.write_str(name),
            AstExpression::Global { name } => f.write_str(name),
            AstExpression::BinaryOp { op, lhs, rhs } => {
                let op_str = match op {
                    SsaBinaryOperation::And => "&&",
                    SsaBinaryOperation::Or => "||",
                    SsaBinaryOperation::Addition => "+",
                    SsaBinaryOperation::Division => "/",
                    SsaBinaryOperation::Modulo => "%",
                    SsaBinaryOperation::Multiply => "*",
                    SsaBinaryOperation::Subtract => "-",
                    SsaBinaryOperation::CompareEqual => "==",
                    SsaBinaryOperation::CompareGreaterThanEqual => ">=",
                    SsaBinaryOperation::CompareGreaterThan => ">",
                    SsaBinaryOperation::CompareNotEqual => "!=",
                };
                f.write_fmt(format_args!("({} {} {})", lhs, op_str, rhs))
            }
            AstExpression::UnaryOp { op, src } => {
                let op_str = match op {
                    SsaUnaryOperation::Not => "!",
                    SsaUnaryOperation::Negate => "-",
                };
                f.write_fmt(format_args!("({}{})", op_str, src))
            }
            AstExpression::Attr { obj, name } => f.write_fmt(format_args!("{}.{}", obj, name)),
            AstExpression::Subscript { obj, index } => {
                f.write_fmt(format_args!("{}[{}]", obj, index))
            }
            AstExpression::Call { func, args } => {
                f.write_fmt(format_args!("{}(", func))?;
                for (i, arg) in args.iter().enumerate() {
                    f.write_fmt(format_args!("{}", arg))?;
                    if i < args.len() - 1 {
                        f.write_str(", ")?;
                    }
                }
                f.write_str(")")
            }
        }
    }
}
