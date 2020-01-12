use std::fmt::{Display, Formatter, Error};

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
}

impl Value {
    pub fn as_number(&self) -> Result<f64, ()> {
        use Value::*;

        match self {
            Number(value) => Ok(*value),

            _ => Err(()),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Value::Nil => f.write_str("nil"),
            Value::Boolean(value) => if *value { f.write_str("true") } else { f.write_str("false") },
            Value::Number(val) => write!(f, "{}", val),
        }
    }
}