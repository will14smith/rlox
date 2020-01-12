use std::fmt::{Display, Formatter, Error};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    Object(Rc<Object>)
}

#[derive(Debug)]
pub enum Object {
    String(String),
}

impl Value {
    pub fn new_string(s: String) -> Value {
        Value::Object(Rc::new(Object::String(s)))
    }

    pub fn as_number(&self) -> Result<f64, ()> {
        use Value::*;

        match self {
            Number(value) => Ok(*value),

            _ => Err(()),
        }
    }

    pub fn is_truthy(&self) -> bool {
        use Value::*;

        match self {
            Nil => false,
            Boolean(value) => *value,

            _ => true,
        }
    }

    pub fn is_equal(&self, other: &Value) -> bool {
        use Value::*;

        match (self, other) {
            (Nil, Nil) => true,
            (Boolean(left), Boolean(right)) => *left == *right,
            (Number(left), Number(right)) => *left == *right,
            (Object(left), Object(right)) => left.is_equal(right),

            _ => false
        }
    }
}

impl Object {
    pub fn is_equal(&self, other: &Object) -> bool {
        use Object::*;

        match (self, other) {
            (String(left), String(right)) => *left == *right,

            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use Value::*;

        match self {
            Nil => f.write_str("nil"),
            Boolean(value) => if *value { f.write_str("true") } else { f.write_str("false") },
            Number(val) => write!(f, "{}", val),
            Object(val) => write!(f, "{}", val),
        }
    }
}
impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use Object::*;

        match self {
            String(val) => write!(f, "{}", val),
        }
    }
}
