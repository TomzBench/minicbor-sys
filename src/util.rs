use cddl_cat::parser::ParseError;
use std::{error, fmt};

#[derive(Debug, PartialEq, Eq)]
pub enum ValidateError {
    Parse(ParseError),
    InvalidEnum0,
    InvalidUnconstrainedPrimative,
    InvalidLiteral,
    InvalidControl,
    InvalidControlArg,
    InvalidGroupMissingKey,
    InvalidType,
    InvalidArray,
    InvalidArraySize,
    TodoEnums,
    UnsupportedCddl(String),
    ForeignKey(String),
    Infallible,
}
impl fmt::Display for ValidateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ValidateError::*;
        match self {
            Parse(e) => e.fmt(f),
            InvalidEnum0 => write!(f, "Enum type with 0 members unsupported"),
            InvalidUnconstrainedPrimative => write!(f, "type must be constrained"),
            InvalidLiteral => write!(f, "invalid literal"),
            InvalidControl => write!(f, "size control only supported on primative types"),
            InvalidControlArg => write!(f, "only integers supported for control args"),
            InvalidGroupMissingKey => write!(f, "all group members must have a key"),
            InvalidType => write!(f, "invalid type"),
            InvalidArray => write!(f, "invalid array"),
            InvalidArraySize => write!(f, "invalid array size"),
            TodoEnums => write!(f, "enums not supported"),
            UnsupportedCddl(cddl) => write!(f, "CDDL not supported {}", cddl),
            ForeignKey(key) => write!(f, "foreign key not defined [{}]", key),
            Infallible => write!(f, "infallible"),
        }
    }
}

impl From<ParseError> for ValidateError {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl error::Error for ValidateError {}
