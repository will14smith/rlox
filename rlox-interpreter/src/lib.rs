mod error;
mod expression;
mod interpreter;
mod stmt;
mod value;

pub use error::{ RuntimeError, RuntimeErrorDescription };
pub use interpreter::Interpreter;
pub use value::Value;

pub type EvaluateResult<T> = Result<T, RuntimeError>;
