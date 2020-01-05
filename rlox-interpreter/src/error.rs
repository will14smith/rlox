use rlox_scanner::SourceToken;
use crate::Value;

#[derive(Debug, PartialEq)]
pub struct RuntimeError {
    pub token: SourceToken,
    pub description: RuntimeErrorDescription,
}

impl RuntimeError {
    pub fn new(token: SourceToken, description: RuntimeErrorDescription) -> RuntimeError {
        RuntimeError { token, description }
    }
}

#[derive(Debug, PartialEq)]
pub enum RuntimeErrorDescription {
    ExpectedNumber,
    InvalidAdditionArguments(Value, Value),
    DivideByZero,
    UndefinedVariable,
}