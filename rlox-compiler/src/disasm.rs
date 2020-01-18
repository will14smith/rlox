use std::io::Write;
use crate::chunk::Chunk;
use crate::op::DecodeError;
use crate::OpCode;

fn write_instruction_header(w: &mut dyn Write, chunk: &Chunk, offset: usize) -> std::io::Result<()> {
    write!(w, "{:#06x} ", offset)?;
    let line = chunk.line(offset);
    let previous_line = if offset > 0 { Some(chunk.line(offset - 1)) } else { None };

    if Some(line) == previous_line {
        write!(w, "   | ")
    } else {
        write!(w, "{:4} ", line)
    }
}

macro_rules! write_constant_op {
    ($w:ident, $op:expr, $chunk:ident, $index:ident) => {
        {
            let value = $chunk.constant($index);
            match value {
                Ok(value) => writeln!($w, "{:16} {} '{}'", $op, $index, value)?,
                Err(err) =>  writeln!($w, "{:16} {} '{}'", $op, $index, err)?,
            }
        }
    };
}

pub fn disassemble_instruction(w: &mut dyn Write, chunk: &Chunk, offset: usize) -> std::io::Result<Option<usize>> {
    match chunk.decode(offset) {
        Ok((op, next_offset)) => {
            write_instruction_header(w, chunk, offset)?;

            match op {
                OpCode::Constant(index) => write_constant_op!(w, "OP_CONSTANT", chunk, index),

                OpCode::True => writeln!(w, "OP_TRUE")?,
                OpCode::False => writeln!(w, "OP_FALSE")?,
                OpCode::Nil => writeln!(w, "OP_NIL")?,
                OpCode::Pop => writeln!(w, "OP_POP")?,

                OpCode::GetLocal(index) => write_constant_op!(w, "OP_GET_LOCAL", chunk, index),
                OpCode::SetLocal(index) => write_constant_op!(w, "OP_SET_LOCAL", chunk, index),
                OpCode::GetGlobal(index) => write_constant_op!(w, "OP_GET_GLOBAL", chunk, index),
                OpCode::DefineGlobal(index) => write_constant_op!(w, "OP_DEFINE_GLOBAL", chunk, index),
                OpCode::SetGlobal(index) => write_constant_op!(w, "OP_SET_GLOBAL", chunk, index),

                OpCode::Equal => writeln!(w, "OP_EQUAL")?,
                OpCode::Greater => writeln!(w, "OP_GREATER")?,
                OpCode::Less => writeln!(w, "OP_LESS")?,
                OpCode::Add => writeln!(w, "OP_ADD")?,
                OpCode::Subtract => writeln!(w, "OP_SUBTRACT")?,
                OpCode::Multiply => writeln!(w, "OP_MULTIPLY")?,
                OpCode::Divide => writeln!(w, "OP_DIVIDE")?,
                OpCode::Not => writeln!(w, "OP_NOT")?,
                OpCode::Negate => writeln!(w, "OP_NEGATE")?,

                OpCode::Print => writeln!(w, "OP_PRINT")?,
                OpCode::Return => writeln!(w, "OP_RETURN")?,

                OpCode::Unknown(val) => writeln!(w, "Unknown opcode {}", val)?,
            }

            Ok(Some(next_offset))
        },

        Err(DecodeError::EOF) => Ok(None),
        Err(err) => {
            write_instruction_header(w, chunk, offset)?;
            writeln!(w, "Error decoding instruction {:?}", err)?;

            // TODO?
            Ok(Some(offset + 1))
        }
    }
}