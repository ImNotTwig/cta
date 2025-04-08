#[derive(Clone)]
pub enum ArgumentWithData {
    String(String),
    UInt(u32),
    Int(i32),
    Bool(bool),
}
impl ArgumentWithData {
    pub fn new(arg: &Argument, val: String) -> anyhow::Result<Self> {
        Ok(match arg {
            Argument::String(_) => Self::String(val),
            Argument::UInt(_) => Self::UInt(val.parse()?),
            Argument::Int(_) => Self::Int(val.parse()?),
            Argument::Bool(_) => Self::Bool(val.parse()?),
        })
    }

    pub fn string(&self) -> Option<String> {
        match self {
            Self::String(d) => Some(d.clone()),
            _ => None,
        }
    }
    pub fn uint(&self) -> Option<u32> {
        match self {
            Self::UInt(d) => Some(*d),
            _ => None,
        }
    }
    pub fn int(&self) -> Option<i32> {
        match self {
            Self::Int(d) => Some(*d),
            _ => None,
        }
    }
    pub fn bool(&self) -> Option<bool> {
        match self {
            Self::Bool(d) => Some(*d),
            _ => None,
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
            Argument::String(d) | Argument::UInt(d) | Argument::Int(d) | Argument::Bool(d) => {
                d.label
            }
        }
    }
    pub fn size(self: Argument) -> u32 {
        match self {
            Argument::String(d) | Argument::UInt(d) | Argument::Int(d) | Argument::Bool(d) => {
                d.size
            }
        }
    }
}
