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

pub fn disassemble_instruction(w: &mut dyn Write, chunk: &Chunk, offset: usize) -> std::io::Result<Option<usize>> {
    match chunk.decode(offset) {
        Ok((op, next_offset)) => {
            write_instruction_header(w, chunk, offset)?;

            match op {
                OpCode::Constant(index) => {
                    let value = chunk.constant(index);
                    match value {
                        Ok(value) => writeln!(w, "OP_CONSTANT         {} '{}'", index, value)?,
                        Err(err) =>  writeln!(w, "OP_CONSTANT         {} '{}'", index, err)?,
                    }
                },
                OpCode::True => writeln!(w, "OP_TRUE")?,
                OpCode::False => writeln!(w, "OP_FALSE")?,
                OpCode::Nil => writeln!(w, "OP_NIL")?,

                OpCode::Add => writeln!(w, "OP_ADD")?,
                OpCode::Subtract => writeln!(w, "OP_SUBTRACT")?,
                OpCode::Multiply => writeln!(w, "OP_MULTIPLY")?,
                OpCode::Divide => writeln!(w, "OP_DIVIDE")?,
                OpCode::Negate => writeln!(w, "OP_NEGATE")?,

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