mod error;
mod expression;
mod interpreter;
mod value;

mod native;

pub use error::{ RuntimeError, RuntimeErrorDescription };
pub use interpreter::Interpreter;
pub use value::Value;

pub type EvaluateResult<T> = Result<T, RuntimeError>;
