pub const OP_CONSTANT: u8 = 0;
pub const OP_TRUE: u8 = OP_CONSTANT + 1;
pub const OP_FALSE: u8 = OP_TRUE + 1;
pub const OP_NIL: u8 = OP_FALSE + 1;
pub const OP_POP: u8 = OP_NIL + 1;

pub const OP_GET_GLOBAL: u8 = OP_POP + 1;
pub const OP_DEFINE_GLOBAL: u8 = OP_GET_GLOBAL + 1;

pub const OP_EQUAL: u8 = OP_DEFINE_GLOBAL + 1;
pub const OP_GREATER: u8 = OP_EQUAL + 1;
pub const OP_LESS: u8 = OP_GREATER + 1;
pub const OP_ADD: u8 = OP_LESS + 1;
pub const OP_SUBTRACT: u8 = OP_ADD + 1;
pub const OP_MULTIPLY: u8 = OP_SUBTRACT + 1;
pub const OP_DIVIDE: u8 = OP_MULTIPLY + 1;
pub const OP_NOT: u8 = OP_DIVIDE + 1;
pub const OP_NEGATE: u8 = OP_NOT + 1;

pub const OP_PRINT: u8 = OP_NEGATE + 1;
pub const OP_RETURN: u8 = OP_PRINT + 1;

pub enum OpCode {
    Constant(u8),
    True,
    False,
    Nil,
    Pop,

    GetGlobal(u8),
    DefineGlobal(u8),

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

            OpCode::GetGlobal(_) => 2,
            OpCode::DefineGlobal(_) => 2,

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
            OP_POP => Ok((OpCode::Pop, 1)),

            OP_GET_GLOBAL => {
                if bytes.len() < 2 {
                    Err(DecodeError::UnexpectedEOF(1, "Missing constant index".into()))
                } else {
                    Ok((OpCode::GetGlobal(bytes[1]), 2))
                }
            }
            OP_DEFINE_GLOBAL => {
                if bytes.len() < 2 {
                    Err(DecodeError::UnexpectedEOF(1, "Missing constant index".into()))
                } else {
                    Ok((OpCode::DefineGlobal(bytes[1]), 2))
                }
            }

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

            OpCode::GetGlobal(index) => vec![OP_GET_GLOBAL, *index],
            OpCode::DefineGlobal(index) => vec![OP_DEFINE_GLOBAL, *index],

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
            OpCode::Return => vec![OP_RETURN],

            OpCode::Unknown(val) => vec![*val],
        }
    }
}