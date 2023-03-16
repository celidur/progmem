use color_print::cprintln;
use map_macro::map;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::errors::CompilerError;

pub fn compile(file_name: String, silent: bool, optimize: bool) -> Result<Vec<u8>, CompilerError> {
    let file = File::open(file_name).expect("Unable to open file");
    let reader = BufReader::new(file);
    let set_instructions = map! {
    "dbt" => (0b00000001, false),
    "att" => (0b00000010, true),
    "dal" => (0b01000100, true),
    "det" => (0b01000101, false),
    "sgo" => (0b01001000, true),
    "sar" => (0b00001001, false),
    "mar" => (0b01100000, false),
    "mav" => (0b01100010, true),
    "mre" => (0b01100011, true),
    "trd" => (0b01100100, false),
    "trg" => (0b01100101, false),
    "dbc" => (0b11000000, true),
    "fbc" => (0b11000001, false),
    "fin" => (0b11111111, false)};
    // define result as a vector of u8
    let mut result = vec![0, 0];
    let comments = Regex::new(r"(//|#|%).*").unwrap();
    let remove_multiple_space = Regex::new("  ").unwrap();
    let mut index_start = 0;
    let mut start = false;
    let mut end = false;
    let mut start_loop = false;
    let mut remove_instruction = vec![];
    let mut can_print;
    let mut warnings = vec![];
    for (i, line) in reader.lines().enumerate() {
        if end && optimize {
            break;
        }

        let Ok(line) = line else {
            return Err(CompilerError::ReadLine(i));
        };

        if let (Some(comment), false) = (comments.find(&line), silent) {
            cprintln!("<green>{}</>", comment.as_str());
        }

        let line = comments.replace_all(&line, "");

        if line.is_empty() {
            continue;
        }

        let instructions: Vec<&str> = line.split(';').map(|s| s.trim()).collect();

        if !instructions.last().unwrap().is_empty() {
            return Err(CompilerError::LackSemiColon(
                i + 1,
                instructions.last().unwrap().to_string(),
            ));
        }

        for instruction in instructions {
            if instruction.is_empty() || (end && optimize) {
                continue;
            }
            can_print = !optimize || start;
            let instruction = remove_multiple_space.replace_all(instruction, " ");
            let mut iterator = instruction.split(' ');
            let operation = iterator.next().unwrap();
            let argument = iterator.next();
            if !(operation.to_lowercase() == operation || operation.to_uppercase() == operation) {
                return Err(CompilerError::CaseError(i + 1, operation.to_string()));
            }
            let operation = operation.to_lowercase();
            let Some(set_instruction) = set_instructions.get(operation.as_str()) else {
                return Err(CompilerError::UnknownInstruction(i+1, instruction.to_string()))
            };
            if operation == "dbt" {
                if start && optimize {
                    remove_instruction.push(result.len());
                    can_print = false;
                } else {
                    index_start = result.len();
                    start = true;
                    can_print = true;
                }
            }
            if operation == "fin" && start {
                end = true;
            }
            if operation == "dbc" {
                if start_loop {
                    return Err(CompilerError::MultipleLoop(i + 1));
                }
                start_loop = true;
            }
            if operation == "fbc" {
                if !start_loop {
                    return Err(CompilerError::NoLoop(i + 1));
                }
                start_loop = false;
            }
            if set_instruction.1 {
                if argument.is_none() {
                    return Err(CompilerError::MissingOperand(i + 1, operation));
                }
                let Ok(argument) = argument.unwrap().parse::<u8>() else {
                    return Err(CompilerError::InvalidOperand(i+1, operation, argument.unwrap().to_string()))
                };
                result.push(set_instruction.0);
                result.push(argument);
                if !silent && can_print {
                    cprintln!(
                        "<blue>{}</> <yellow>{}</>\t-> <blue>{:02x}</> <yellow>{:02x}</>",
                        operation,
                        argument,
                        set_instruction.0,
                        argument
                    )
                }
            }
            if !set_instruction.1 {
                if argument.is_some() {
                    warnings.push(format!(
                        "Warning line {}: Instruction '{}' does not need an operand",
                        i + 1,
                        operation
                    ));
                }
                result.push(set_instruction.0);
                result.push(0);
                if !silent && can_print {
                    cprintln!(
                        "<blue>{}</>\t-> <blue>{:02x}</> 00",
                        operation,
                        set_instruction.0
                    )
                }
            }
        }
    }
    if !start {
        return Err(CompilerError::MissingStart);
    }
    if !end {
        return Err(CompilerError::MissingEnd);
    }
    if start_loop {
        return Err(CompilerError::MissingLoopEnd);
    }
    if optimize {
        for i in remove_instruction.iter().rev() {
            result.remove(*i);
            result.remove(*i);
        }
        result.drain(0..index_start - 2);
    }
    result[0] = (result.len() >> 8) as u8;
    result[1] = (result.len() & 0xff) as u8;
    for warning in warnings {
        cprintln!("<yellow>{}</>", warning);
    }
    println!("----------------\nsize: {}", result.len());
    Ok(result)
}
