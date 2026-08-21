use iris_syntax::{Expression, ProgramEntry, Statement};

use super::lowering::Lowering;
use super::{CompileError, Instruction, Register};

pub(super) fn ordinary_receiver_decline(
    receiver: &Expression,
    is_bound: impl FnOnce(&str) -> bool,
) -> Option<&'static str> {
    match receiver {
        Expression::Name(name) if !is_bound(name) => Some("call unbound receiver"),
        _ => None,
    }
}

impl Lowering<'_, '_> {
    pub(super) fn interpolated_text(&mut self, text: &str) -> Result<Register, CompileError> {
        let mut rest = text;
        let mut built = self.load_text("")?;
        while let Some(open) = rest.find("${") {
            let literal = self.load_text(&rest[..open])?;
            built = self.concat_text(built, literal)?;
            let after = &rest[open + 2..];
            let Some(end) = after.find('}') else {
                return Err(CompileError::new("bad interpolation"));
            };
            let parsed = iris_parser::parse(&after[..end]);
            let [ProgramEntry::Statement(Statement::Expression(expression))] =
                parsed.program.entries.as_slice()
            else {
                return Err(CompileError::new("bad interpolation"));
            };
            if !parsed.program_accepted {
                return Err(CompileError::new("bad interpolation"));
            }
            let value = self.expression(expression)?;
            built = self.concat_text(built, value)?;
            rest = &after[end + 1..];
        }
        let literal = self.load_text(rest)?;
        self.concat_text(built, literal)
    }

    fn load_text(&mut self, text: &str) -> Result<Register, CompileError> {
        let destination = self.allocate()?;
        self.instructions.push(Instruction::LoadText {
            destination,
            text: text.to_owned(),
        });
        Ok(destination)
    }

    fn concat_text(&mut self, left: Register, right: Register) -> Result<Register, CompileError> {
        let destination = self.allocate()?;
        self.instructions.push(Instruction::Binary {
            destination,
            selector: "+",
            left,
            right,
        });
        Ok(destination)
    }
}
