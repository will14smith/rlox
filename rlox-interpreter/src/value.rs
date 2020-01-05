#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Value {
    pub fn as_number(&self) -> f64 {
        use Value::*;

        match self {
            Number(value) => *value,

            Boolean(_) | String(_) | Nil => unimplemented!("handle runtime error"),
        }
    }

    pub fn is_truthy(&self) -> bool {
        use Value::*;

        match self {
            Nil => false,
            Boolean(value) => *value,
            Number(_) => true,
            String(_) => true,
        }
    }

    pub fn is_equal(&self, other: &Value) -> bool {
        use Value::*;

        match (self, other) {
            (Nil, Nil) => true,
            (Boolean(left), Boolean(right)) => *left == *right,
            (Number(left), Number(right)) => *left == *right,
            (String(left), String(right)) => *left == *right,

            _ => false
        }
    }
}