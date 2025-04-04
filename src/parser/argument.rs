#[derive(Clone)]
pub enum ArgumentWithData {
    String(String),
    UInt(u32),
    Int(i32),
    Bool(bool),
}
impl ArgumentWithData {
    pub fn string(self) -> String {
        match self {
            Self::String(d) => d,
            _ => unreachable!(),
        }
    }
    pub fn uint(self) -> u32 {
        match self {
            Self::UInt(d) => d,
            _ => unreachable!(),
        }
    }
    pub fn int(self) -> i32 {
        match self {
            Self::Int(d) => d,
            _ => unreachable!(),
        }
    }
    pub fn bool(self) -> bool {
        match self {
            Self::Bool(d) => d,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct ArgumentMetadata {
    pub label: String,
    pub size: u32,
}

#[derive(Clone)]
pub enum Argument {
    String(ArgumentMetadata),
    UInt(ArgumentMetadata),
    Int(ArgumentMetadata),
    Bool(ArgumentMetadata),
}

impl Argument {
    pub fn label(self: Argument) -> String {
        match self {
            Argument::String(d) => d.label,
            Argument::UInt(d) => d.label,
            Argument::Int(d) => d.label,
            Argument::Bool(d) => d.label,
        }
    }
    pub fn size(self: Argument) -> u32 {
        match self {
            Argument::String(d) => d.size,
            Argument::UInt(d) => d.size,
            Argument::Int(d) => d.size,
            Argument::Bool(d) => d.size,
        }
    }
}
