use cddl_cat::parser::ParseError;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ValidateError {
    #[error("Parser {0}")]
    Parse(#[from] ParseError),
    #[error("Enum type with 0 members unsupported")]
    InvalidEnum0,
    #[error("Types must be constrained")]
    InvalidUnconstrainedPrimative,
    #[error("Invalid literal type")]
    InvalidLiteral,
    #[error("Size control only supported on primative types")]
    InvalidControl,
    #[error("Only Integers supported for control Args")]
    InvalidControlArg,
    #[error("All group memembers must have a key")]
    InvalidGroupMissingKey,
    #[error("Invalid type")]
    InvalidType,
    #[error("Invalid array")]
    InvalidArray,
    #[error("Invalid array size")]
    InvalidArraySize,
    #[error("TODO - enums not supported yet")]
    TodoEnums,
    #[error("CDDL not supported {0}")]
    UnsupportedCddl(String),
    #[error("Foreign key {0} not defined")]
    ForeignKey(String),
    #[error("infallible")]
    Infallible,
}
