use std::convert::TryInto;

pub const OP_CONSTANT: u8 = 0;
pub const OP_TRUE: u8 = OP_CONSTANT + 1;
pub const OP_FALSE: u8 = OP_TRUE + 1;
pub const OP_NIL: u8 = OP_FALSE + 1;
pub const OP_POP: u8 = OP_NIL + 1;

pub const OP_GET_LOCAL: u8 = OP_POP + 1;
pub const OP_SET_LOCAL: u8 = OP_GET_LOCAL + 1;
pub const OP_GET_GLOBAL: u8 = OP_SET_LOCAL + 1;
pub const OP_DEFINE_GLOBAL: u8 = OP_GET_GLOBAL + 1;
pub const OP_SET_GLOBAL: u8 = OP_DEFINE_GLOBAL + 1;

pub const OP_EQUAL: u8 = OP_SET_GLOBAL + 1;
pub const OP_GREATER: u8 = OP_EQUAL + 1;
pub const OP_LESS: u8 = OP_GREATER + 1;
pub const OP_ADD: u8 = OP_LESS + 1;
pub const OP_SUBTRACT: u8 = OP_ADD + 1;
pub const OP_MULTIPLY: u8 = OP_SUBTRACT + 1;
pub const OP_DIVIDE: u8 = OP_MULTIPLY + 1;
pub const OP_NOT: u8 = OP_DIVIDE + 1;
pub const OP_NEGATE: u8 = OP_NOT + 1;

pub const OP_PRINT: u8 = OP_NEGATE + 1;
pub const OP_JUMP: u8 = OP_PRINT + 1;
pub const OP_JUMP_IF_FALSE: u8 = OP_JUMP + 1;
pub const OP_RETURN: u8 = OP_JUMP_IF_FALSE + 1;

pub enum OpCode {
    Constant(u8),
    True,
    False,
    Nil,
    Pop,

    GetLocal(u8),
    SetLocal(u8),
    GetGlobal(u8),
    DefineGlobal(u8),
    SetGlobal(u8),

    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,

    Print,
    Jump(i16),
    JumpIfFalse(i16),
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
            OpCode::Pop => 1,

            OpCode::GetLocal(_) => 2,
            OpCode::SetLocal(_) => 2,
            OpCode::GetGlobal(_) => 2,
            OpCode::DefineGlobal(_) => 2,
            OpCode::SetGlobal(_) => 2,

            OpCode::Equal => 1,
            OpCode::Greater => 1,
            OpCode::Less => 1,
            OpCode::Add => 1,
            OpCode::Subtract => 1,
            OpCode::Multiply => 1,
            OpCode::Divide => 1,
            OpCode::Not => 1,
            OpCode::Negate => 1,

            OpCode::Print => 1,
            OpCode::Jump(_) => 3,
            OpCode::JumpIfFalse(_) => 3,
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

macro_rules! constant_op {
    ($type:path, $bytes:expr) => {
        {
            if $bytes.len() < 2 {
                Err(DecodeError::UnexpectedEOF(1, "Missing constant index".into()))
            } else {
                Ok(($type($bytes[1]), 2))
            }
        }
    };
}
macro_rules! jump_op {
    ($type:path, $bytes:ident) => {
        {
            if $bytes.len() < 3 {
                Err(DecodeError::UnexpectedEOF(1, "Missing jump offset".into()))
            } else {
                let offset = i16::from_be_bytes((&$bytes[1..3]).try_into().unwrap());
                Ok(($type(offset), 3))
            }
        }
    };
}

// decode/encode
impl OpCode {
    pub fn decode(bytes: &[u8]) -> Result<(OpCode, usize), DecodeError> {
        if bytes.len() == 0 {
            return Err(DecodeError::EOF);
        }

        match bytes[0] {
            OP_CONSTANT => constant_op!(OpCode::Constant, bytes),
            OP_TRUE => Ok((OpCode::True, 1)),
            OP_FALSE => Ok((OpCode::False, 1)),
            OP_NIL => Ok((OpCode::Nil, 1)),
            OP_POP => Ok((OpCode::Pop, 1)),

            OP_GET_LOCAL => constant_op!(OpCode::GetLocal, bytes),
            OP_SET_LOCAL => constant_op!(OpCode::SetLocal, bytes),
            OP_GET_GLOBAL => constant_op!(OpCode::GetGlobal, bytes),
            OP_DEFINE_GLOBAL => constant_op!(OpCode::DefineGlobal, bytes),
            OP_SET_GLOBAL => constant_op!(OpCode::SetGlobal, bytes),

            OP_EQUAL => Ok((OpCode::Equal, 1)),
            OP_GREATER => Ok((OpCode::Greater, 1)),
            OP_LESS => Ok((OpCode::Less, 1)),
            OP_ADD => Ok((OpCode::Add, 1)),
            OP_SUBTRACT => Ok((OpCode::Subtract, 1)),
            OP_MULTIPLY => Ok((OpCode::Multiply, 1)),
            OP_DIVIDE => Ok((OpCode::Divide, 1)),
            OP_NOT => Ok((OpCode::Not, 1)),
            OP_NEGATE => Ok((OpCode::Negate, 1)),

            OP_PRINT => Ok((OpCode::Print, 1)),
            OP_JUMP => jump_op!(OpCode::Jump, bytes),
            OP_JUMP_IF_FALSE => jump_op!(OpCode::JumpIfFalse, bytes),
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
            OpCode::Pop => vec![OP_POP],

            OpCode::GetLocal(index) => vec![OP_GET_LOCAL, *index],
            OpCode::SetLocal(index) => vec![OP_SET_LOCAL, *index],
            OpCode::GetGlobal(index) => vec![OP_GET_GLOBAL, *index],
            OpCode::DefineGlobal(index) => vec![OP_DEFINE_GLOBAL, *index],
            OpCode::SetGlobal(index) => vec![OP_SET_GLOBAL, *index],

            OpCode::Equal => vec![OP_EQUAL],
            OpCode::Greater => vec![OP_GREATER],
            OpCode::Less => vec![OP_LESS],
            OpCode::Add => vec![OP_ADD],
            OpCode::Subtract => vec![OP_SUBTRACT],
            OpCode::Multiply => vec![OP_MULTIPLY],
            OpCode::Divide => vec![OP_DIVIDE],
            OpCode::Not => vec![OP_NOT],
            OpCode::Negate => vec![OP_NEGATE],

            OpCode::Print => vec![OP_PRINT],
            OpCode::Jump(offset) => { let mut b = vec![OP_JUMP]; b.extend_from_slice(&offset.to_be_bytes()[..]); b },
            OpCode::JumpIfFalse(offset) => { let mut b = vec![OP_JUMP_IF_FALSE]; b.extend_from_slice(&offset.to_be_bytes()[..]); b },
            OpCode::Return => vec![OP_RETURN],

            OpCode::Unknown(val) => vec![*val],
        }
    }
}