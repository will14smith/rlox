pub const OP_CONSTANT: u8 = 0x00;
pub const OP_ADD: u8 = 0x01;
pub const OP_SUBTRACT: u8 = 0x02;
pub const OP_MULTIPLY: u8 = 0x03;
pub const OP_DIVIDE: u8 = 0x04;
pub const OP_NEGATE: u8 = 0x05;
pub const OP_RETURN: u8 = 0x06;

pub enum OpCode {
    Constant(u8),
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