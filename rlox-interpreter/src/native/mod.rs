use std::rc::Rc;
use crate::{
    Value,
    interpreter::Environment,
};

mod clock;

pub fn define_functions(environment: &mut Environment){
    environment.define(String::from("clock"), Value::Function(Rc::new(clock::Clock)));
}