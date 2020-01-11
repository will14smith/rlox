use std::io::Write;
use crate::chunk::Chunk;
use crate::op::DecodeError;
use crate::OpCode;

pub fn disasm(w: &mut dyn Write, chunk: &Chunk, name: &str) -> std::io::Result<()> {
    writeln!(w, "== {} ==", name)?;

    let mut offset = 0;
    loop {
        if let Some(next_offset) = disassemble_instruction(w, &chunk, offset)? {
            offset = next_offset;
        } else {
            break;
        }
    }

    Ok(())
}

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