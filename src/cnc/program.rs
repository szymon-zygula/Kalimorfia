use crate::cnc::{
    location::Location,
    mill::MillInstruction,
    parser::{self, LineParseError},
};
use thiserror::Error;

pub enum UnitSystem {
    Metric,
}

pub enum CoordinateSystemType {
    Global,
}

pub enum Winding {
    CW,
}

pub enum Instruction {
    CoordinateSystemType(CoordinateSystemType),
    RotationSpeed(u32),
    Winding(Winding),
    RotationSpeedAndWinding {
        rotation_speed: u32,
        winding: Winding,
    },
    MovementSpeed(u32),
    MoveFast(Location),
    MoveSlow(Location),
    TurnOff,
    End,
}

pub enum ProgramLine {
    UnitSystem(UnitSystem),
    Instruction {
        number: u32,
        instruction: Instruction,
    },
}

pub struct Program(Vec<MillInstruction>);

#[derive(Error, Debug)]
pub enum ProgramLoadError {
    #[error("IO error")]
    Io(std::io::Error),
    #[error("parse error")]
    ParseError(LineParseError),
    #[error("a not numbered line inbetween other lines")]
    StrayLine,
    #[error("non-sequential line numbering: line numbered as {number} is actually {actual}")]
    LineSequence { number: u32, actual: u32 },
    #[error("unit system is not set before the first move")]
    UnitsNotSet,
    #[error("coordinate system is not set before the first move")]
    CoordinateSystemNotSet,
    #[error("no end instruction at the end of the program")]
    NoEndInstruction,
    #[error("mill not turned off befor the end of the program")]
    NoTurnOff,
    #[error("winding not set before rotation speed set")]
    NoWinding,
}

impl Program {
    pub fn from_file(path: &std::path::Path) -> Result<Self, ProgramLoadError> {
        let source = std::fs::read_to_string(path).map_err(ProgramLoadError::Io)?;
        let lines = parser::parse_source(&source).map_err(ProgramLoadError::ParseError)?;
        Self::from_lines(lines)
    }

    pub fn from_lines(lines: Vec<ProgramLine>) -> Result<Self, ProgramLoadError> {
        let lines = lines;
        Self::validate_lines(&lines)?;
        Ok(Self(Self::lines_to_mill_instructions(&lines)))
    }

    fn lines_to_mill_instructions(lines: &[ProgramLine]) -> Vec<MillInstruction> {
        lines
            .iter()
            .flat_map(Self::line_to_mill_instruction)
            .collect()
    }

    fn line_to_mill_instruction(line: &ProgramLine) -> Vec<MillInstruction> {
        match line {
            ProgramLine::UnitSystem(_) => Vec::new(),
            ProgramLine::Instruction { instruction, .. } => match instruction {
                Instruction::Winding(_) => Vec::new(),
                Instruction::RotationSpeed(speed) => {
                    vec![MillInstruction::RotationSpeed(*speed as f32 / 1000.0)]
                }
                Instruction::RotationSpeedAndWinding {
                    rotation_speed,
                    winding,
                } => todo!(),
                Instruction::MovementSpeed(speed) => {
                    vec![MillInstruction::MovementSpeed(*speed as f32 / 1000.0)]
                }
                Instruction::MoveFast(location) => {
                    vec![MillInstruction::MoveFast(location.clone())]
                }
                Instruction::MoveSlow(location) => {
                    vec![MillInstruction::MoveSlow(location.clone())]
                }
                Instruction::TurnOff => Vec::new(),
                Instruction::End => Vec::new(),
                Instruction::CoordinateSystemType(_) => Vec::new(),
            },
        }
    }

    fn validate_lines(lines: &[ProgramLine]) -> Result<(), ProgramLoadError> {
        let lines = Self::validate_units(lines)?;
        Self::validate_line_sequenciality(lines)?;
        let lines = Self::validate_coordinate_system(lines)?;

        Ok(())
    }

    fn validate_units(lines: &[ProgramLine]) -> Result<&[ProgramLine], ProgramLoadError> {
        if !matches!(lines.first(), Some(ProgramLine::UnitSystem(_))) {
            return Err(ProgramLoadError::UnitsNotSet);
        }

        Ok(&lines[1..])
    }

    fn validate_coordinate_system(
        lines: &[ProgramLine],
    ) -> Result<&[ProgramLine], ProgramLoadError> {
        if let Some(ProgramLine::Instruction { instruction, .. }) = lines.first() {
            if let Instruction::CoordinateSystemType(_) = instruction {
                return Ok(&lines[1..]);
            }
        }

        Err(ProgramLoadError::CoordinateSystemNotSet)
    }

    fn validate_line_sequenciality(lines: &[ProgramLine]) -> Result<(), ProgramLoadError> {
        for (actual, line) in lines.iter().enumerate() {
            let ProgramLine::Instruction { number, .. } = line else {
                return Err(ProgramLoadError::StrayLine);
            };

            return Err(ProgramLoadError::LineSequence {
                actual: actual as u32,
                number: *number,
            });
        }

        Ok(())
    }

    fn validate_gracefull_exit(lines: &[ProgramLine]) -> Result<(), ProgramLoadError> {
        let len = lines.len();

        if len == 0
            || !matches!(
                lines[len - 1],
                ProgramLine::Instruction {
                    instruction: Instruction::End,
                    ..
                }
            )
        {
            return Err(ProgramLoadError::NoEndInstruction);
        }

        if len == 1
            || !matches!(
                lines[len - 2],
                ProgramLine::Instruction {
                    instruction: Instruction::TurnOff,
                    ..
                }
            )
        {
            return Err(ProgramLoadError::NoTurnOff);
        }

        Ok(())
    }

    fn validate_winding(lines: &[ProgramLine]) -> Result<(), ProgramLoadError> {
        for line in lines {
            let ProgramLine::Instruction { instruction, .. } = line else {
                continue;
            };

            if matches!(instruction, Instruction::Winding(_))
                || matches!(instruction, Instruction::RotationSpeedAndWinding { .. })
            {
                return Ok(());
            }

            if matches!(instruction, Instruction::RotationSpeed(_)) {
                return Err(ProgramLoadError::NoWinding);
            }
        }

        Ok(())
    }
}
