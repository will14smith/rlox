mod error;
mod expression;
mod value;

pub use error::{ RuntimeError, RuntimeErrorDescription };
pub use expression::evaluate as evaluate_expression;
pub use value::Value;

pub type EvaluateResult<T> = Result<T, RuntimeError>;
