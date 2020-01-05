#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Value {
    pub fn as_number(&self) -> f64 {
        match self {
            Value::Number(value) => *value,

            Value::Boolean(_) | Value::String(_) | Value::Nil => unimplemented!("handle runtime error"),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(value) => *value,
            Value::Number(_) => true,
            Value::String(_) => true,
        }
    }
}