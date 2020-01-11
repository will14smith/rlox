use std::io::Write;
use crate::{
    chunk::Chunk,
    op,
};

pub fn disasm(w: &mut dyn Write, chunk: &Chunk, name: &str) -> std::io::Result<()> {
    writeln!(w, "== {} ==", name)?;

    let mut iter = chunk.iter().enumerate();
    loop {
        let is_done = disasm_instrument(w, &chunk, &mut iter)?;

        if is_done {
            break;
        }
    }

    Ok(())
}

pub fn disasm_instrument(w: &mut dyn Write, chunk: &Chunk, iter: &mut dyn std::iter::Iterator<Item=(usize, &u8)>) -> std::io::Result<bool> {
    let op = iter.next();

    match op {
        Some((offset, op)) => {
            write!(w, "{:#06x} ", offset)?;
            let line = chunk.line(offset);
            let previous_line = if offset > 0 { Some(chunk.line(offset - 1)) } else { None };

            if Some(line) == previous_line {
                write!(w, "   | ")?;
            } else {
                write!(w, "{:4} ", line)?;
            }

            match *op {
                op::OP_CONSTANT => {
                    let index = iter.next();

                    match index {
                        Some((_, index)) => {
                            let value = chunk.constant(*index);
                            match value {
                                Ok(value) => writeln!(w, "OP_CONSTANT         {} '{}'", index, value)?,
                                Err(err) =>  writeln!(w, "OP_CONSTANT         {} '{}'", index, err)?,
                            }
                        },
                        None => {
                            writeln!(w, "OP_CONSTANT <invalid EOF>")?;
                            return Ok(true)
                        }
                    }
                },

                op::OP_RETURN => {
                    writeln!(w, "OP_RETURN")?
                },

                _ => writeln!(w, "Unknown opcode {}", op)?,
            }

            Ok(false)
        },
        None => {
            Ok(true)
        }
    }
}