use std::time::SystemTime;
use rlox_scanner::SourceToken;
use crate::{ RuntimeError, RuntimeErrorDescription, value::{Callable, Value} };
use std::fmt::{Display, Formatter, Error};

#[derive(Clone, Debug)]
pub struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let time = SystemTime::now();
        let x = time.duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| RuntimeError::new(SourceToken::default(), RuntimeErrorDescription::Message(format!("Error getting current time: {:?}", e))))?;

        Ok(Value::Number(x.as_secs_f64()))
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<native fn>")
    }
}