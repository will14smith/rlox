pub const OP_CONSTANT: u8 = 0x00;
pub const OP_RETURN: u8 = 0x01;

pub enum OpCode {
    Constant(u8),
    Return,

    Unknown(u8),
}

impl OpCode {
    pub fn byte_length(&self) -> usize {
        match self {
            OpCode::Constant(_) => 2,
            OpCode::Return => 1,

            OpCode::Unknown(_) => 1,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            OpCode::Constant(index) => vec![OP_CONSTANT, *index],
            OpCode::Return => vec![OP_RETURN],

            OpCode::Unknown(val) => vec![*val],
        }
    }
}