use std::fmt::{Display, Formatter, Error};

pub enum Value {
    Number(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Value::Number(val) => write!(f, "{}", val),
        }
    }
}