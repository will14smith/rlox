pub const OP_CONSTANT: u8 = 0;
pub const OP_TRUE: u8 = OP_CONSTANT + 1;
pub const OP_FALSE: u8 = OP_TRUE + 1;
pub const OP_NIL: u8 = OP_FALSE + 1;

pub const OP_ADD: u8 = OP_NIL + 1;
pub const OP_SUBTRACT: u8 = OP_ADD + 1;
pub const OP_MULTIPLY: u8 = OP_SUBTRACT + 1;
pub const OP_DIVIDE: u8 = OP_MULTIPLY + 1;
pub const OP_NEGATE: u8 = OP_DIVIDE + 1;

pub const OP_RETURN: u8 = OP_NEGATE + 1;

pub enum OpCode {
    Constant(u8),
    True,
    False,
    Nil,

    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,

    Return,

    Unknown(u8),
}

// length
impl OpCode {
    // todo self.Encode().len() ??
    pub fn byte_length(&self) -> usize {
        match self {
            OpCode::Constant(_) => 2,
            OpCode::True => 1,
            OpCode::False => 1,
            OpCode::Nil => 1,

            OpCode::Add => 1,
            OpCode::Subtract => 1,
            OpCode::Multiply => 1,
            OpCode::Divide => 1,
            OpCode::Negate => 1,

            OpCode::Return => 1,

            OpCode::Unknown(_) => 1,
        }
    }
}

#[derive(Debug)]
pub enum DecodeError {
    EOF,
    UnexpectedEOF(usize, String),
}

// decode/encode
impl OpCode {
    pub fn decode(bytes: &[u8]) -> Result<(OpCode, usize), DecodeError> {
        if bytes.len() == 0 {
            return Err(DecodeError::EOF);
        }

        match bytes[0] {
            OP_CONSTANT => {
                if bytes.len() < 2 {
                    Err(DecodeError::UnexpectedEOF(1, "Missing constant index".into()))
                } else {
                    Ok((OpCode::Constant(bytes[1]), 2))
                }
            }
            OP_TRUE => Ok((OpCode::True, 1)),
            OP_FALSE => Ok((OpCode::False, 1)),
            OP_NIL => Ok((OpCode::Nil, 1)),

            OP_ADD => Ok((OpCode::Add, 1)),
            OP_SUBTRACT => Ok((OpCode::Subtract, 1)),
            OP_MULTIPLY => Ok((OpCode::Multiply, 1)),
            OP_DIVIDE => Ok((OpCode::Divide, 1)),
            OP_NEGATE => Ok((OpCode::Negate, 1)),

            OP_RETURN => Ok((OpCode::Return, 1)),

            _ => {
                Ok((OpCode::Unknown(bytes[0]), 1))
            }
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            OpCode::Constant(index) => vec![OP_CONSTANT, *index],
            OpCode::True => vec![OP_TRUE],
            OpCode::False => vec![OP_FALSE],
            OpCode::Nil => vec![OP_NIL],

            OpCode::Add => vec![OP_ADD],
            OpCode::Subtract => vec![OP_SUBTRACT],
            OpCode::Multiply => vec![OP_MULTIPLY],
            OpCode::Divide => vec![OP_DIVIDE],
            OpCode::Negate => vec![OP_NEGATE],

            OpCode::Return => vec![OP_RETURN],

            OpCode::Unknown(val) => vec![*val],
        }
    }
}