use super::program::{CoordinateSystemType, Instruction, ProgramLine, UnitSystem, Winding};
use itertools::Itertools;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unknown instruction")]
    UnknownInstruction,
    #[error("unsupported unit system")]
    UnsupportedUnitSystem,
    #[error("unsupported winding")]
    UnsupportedWinding,
    #[error("invalid number")]
    InvalidLocation,
    #[error("invalid line number")]
    InvalidLineNumber,
    #[error("invalid movement speed")]
    InvalidMovementSpeed,
    #[error("invalid rotation speed")]
    InvalidRotationSpeed,
    #[error("instruction syntax error")]
    InstructionSyntaxError,
}

#[derive(Debug)]
pub struct LineParseError {
    line_number: usize,
    error: ParseError,
}

type ParseOptionResultLine = Option<Result<ProgramLine, ParseError>>;
type ParseOptionResult = Option<Result<Instruction, ParseError>>;

pub fn parse_source(source: &str) -> Result<Vec<ProgramLine>, LineParseError> {
    Ok(source
        .lines()
        .enumerate()
        .map(|(line_number, line)| {
            parse_instruction(line.trim()).map_err(|error| LineParseError { line_number, error })
        })
        .try_collect()?)
}

fn parse_instruction(source: &str) -> Result<ProgramLine, ParseError> {
    parse_unit_system(source)
        .or_else(|| parse_numbered_instruction(source))
        .unwrap_or(Err(ParseError::UnknownInstruction))
}

fn parse_unit_system(source: &str) -> ParseOptionResultLine {
    source.strip_prefix("%G").map(|label| {
        if label == "71" {
            Ok(ProgramLine::UnitSystem(UnitSystem::Metric))
        } else {
            Err(ParseError::UnsupportedUnitSystem)
        }
    })
}

fn parse_numbered_instruction(source: &str) -> ParseOptionResultLine {
    let source = source.strip_prefix('N')?;

    Some(
        if let Some((number, digit_count)) = parse_prefix_number(source) {
            if let Ok(data) = parse_clean_numbered_instruction(&source[0..digit_count as usize]) {
                Ok(ProgramLine::Instruction {
                    number,
                    instruction: data,
                })
            } else {
                Err(ParseError::UnknownInstruction)
            }
        } else {
            Err(ParseError::InvalidLineNumber)
        },
    )
}

fn parse_prefix_number(source: &str) -> Option<(u32, u8)> {
    let mut digit_count: u8 = 0;
    for c in source.chars() {
        if c.is_ascii_digit() {
            digit_count = digit_count.checked_add(1)?;
        } else {
            break;
        }
    }

    source[0..digit_count as usize]
        .parse::<u32>()
        .ok()
        .map(|number| (number, digit_count))
}

// Instruction with its number stripped
fn parse_clean_numbered_instruction(source: &str) -> Result<Instruction, ParseError> {
    parse_coordinate_system_type(source)
        .or_else(|| parse_movement_speed(source))
        .or_else(|| parse_rotation_speed_and_optional_winding(source))
        .or_else(|| parse_winding(source))
        .or_else(|| parse_move_fast(source))
        .or_else(|| parse_move_slow(source))
        .or_else(|| parse_turn_off(source))
        .or_else(|| parse_end(source))
        .unwrap_or(Err(ParseError::UnknownInstruction))
}

fn parse_coordinate_system_type(source: &str) -> ParseOptionResult {
    (source == "G40G90").then_some(Ok(Instruction::CoordinateSystemType(
        CoordinateSystemType::Global,
    )))
}

fn parse_movement_speed(source: &str) -> ParseOptionResult {
    let Ok(speed) = source.strip_prefix('F')?.parse::<u32>() else {
        return Some(Err(ParseError::InvalidMovementSpeed));
    };

    Some(Ok(Instruction::MovementSpeed(speed)))
}

fn parse_rotation_speed_and_optional_winding(source: &str) -> ParseOptionResult {
    let source = source.strip_prefix('S')?;
    let Some((rotation_speed, speed_digits)) = parse_prefix_number(source) else {
        return Some(Err(ParseError::InvalidRotationSpeed));
    };

    if source.len() == speed_digits as usize {
        return Some(Ok(Instruction::RotationSpeed(rotation_speed)));
    }

    let Some(winding) = parse_winding(&source[0..speed_digits as usize]) else {
        return Some(Err(ParseError::InstructionSyntaxError));
    };

    Some(if let Ok(Instruction::Winding(winding)) = winding {
        Ok(Instruction::RotationSpeedAndWinding {
            rotation_speed,
            winding,
        })
    } else {
        // parse_winding should not return anything other than
        // NumberedInstruction::Winding
        assert!(winding.is_err());
        winding
    })
}

fn parse_winding(source: &str) -> ParseOptionResult {
    (source == "M03").then_some(Ok(Instruction::Winding(Winding::CW)))
}

fn parse_move_fast(source: &str) -> ParseOptionResult {
    Some(if let Ok(location) = source.strip_prefix("G00")?.parse() {
        Ok(Instruction::MoveFast(location))
    } else {
        Err(ParseError::InvalidLocation)
    })
}

fn parse_move_slow(source: &str) -> ParseOptionResult {
    Some(if let Ok(location) = source.strip_prefix("G01")?.parse() {
        Ok(Instruction::MoveSlow(location))
    } else {
        Err(ParseError::InvalidLocation)
    })
}

fn parse_turn_off(source: &str) -> ParseOptionResult {
    (source == "M05").then_some(Ok(Instruction::TurnOff))
}

fn parse_end(source: &str) -> ParseOptionResult {
    (source == "M30").then_some(Ok(Instruction::End))
}
