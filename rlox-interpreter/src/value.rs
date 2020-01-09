use std::fmt::{ Debug, Display };
use std::rc::Rc;
use crate::{ RuntimeError };

#[derive(Clone, Debug)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Rc<dyn Callable>),
}

pub trait Callable : Debug + Display {
    fn arity(&self) -> usize;
    fn call(&self, arguments: Vec<Value>) -> Result<Value, RuntimeError>;
}

impl Value {
    pub fn as_number(&self) -> Result<f64, ()> {
        use Value::*;

        match self {
            Number(value) => Ok(*value),

            _ => Err(()),
        }
    }

    pub fn as_callable(&self) -> Result<&dyn Callable, ()> {
        use Value::*;

        match self {
            Function(function) => Ok((*function).as_ref()),

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
            (String(left), String(right)) => *left == *right,
            (Function(left), Function(right)) => ::std::ptr::eq(left.as_ref(), right.as_ref()),

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
            Value::Function(function) => write!(f, "{}", function),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
