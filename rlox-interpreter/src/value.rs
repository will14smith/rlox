#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Value {
    pub fn as_number(&self) -> Result<f64, ()> {
        use Value::*;

        match self {
            Number(value) => Ok(*value),

            Boolean(_) | String(_) | Nil => Err(()),
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

impl ::std::fmt::Display for Value {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
        match self {
            Value::Nil => f.write_str("nil"),
            Value::Boolean(value) => if *value { f.write_str("true") } else { f.write_str("false") },
            Value::Number(value) => write!(f, "{}", value),
            Value::String(value) => f.write_str(value),
        }
    }
}