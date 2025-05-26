use color_print::cprintln;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::errors::CompilerError;

fn instruction_data(op: &str) -> Option<(u8, bool)> {
    match op {
        "dbt" => Some((0b0000_0001, false)),
        "att" => Some((0b0000_0010, true)),
        "dal" => Some((0b0100_0100, true)),
        "det" => Some((0b0100_0101, false)),
        "sgo" => Some((0b0100_1000, true)),
        "sar" => Some((0b0000_1001, false)),
        "mar" => Some((0b0110_0000, false)),
        "mav" => Some((0b0110_0010, true)),
        "mre" => Some((0b0110_0011, true)),
        "trd" => Some((0b0110_0100, false)),
        "trg" => Some((0b0110_0101, false)),
        "dbc" => Some((0b1100_0000, true)),
        "fbc" => Some((0b1100_0001, false)),
        "fin" => Some((0b1111_1111, false)),
        _ => None,
    }
}

pub fn compile(file_name: String, silent: bool, optimize: bool) -> Result<Vec<u8>, CompilerError> {
    let file = File::open(file_name).expect("Unable to open file");
    let reader = BufReader::new(file);
    // define result as a vector of u8
    let mut result = vec![0, 0];
    let comments = Regex::new(r"(//|#|%).*").unwrap();
    let remove_multiple_space = Regex::new(r"\s+").unwrap();
    let mut start = false;
    let mut end = false;
    let mut start_loop = false;
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
            if instruction.is_empty() {
                continue;
            }
            let instruction = remove_multiple_space.replace_all(instruction, " ");
            let mut iterator = instruction.split(' ');
            let operation = iterator.next().unwrap();
            let argument = iterator.next();
            if !(operation.to_lowercase() == operation || operation.to_uppercase() == operation) {
                return Err(CompilerError::CaseError(i + 1, operation.to_string()));
            }
            let operation = operation.to_lowercase();
            let Some((opcode, needs_arg)) = instruction_data(&operation) else {
                return Err(CompilerError::UnknownInstruction(
                    i + 1,
                    instruction.to_string(),
                ));
            };

            if optimize && (!start || end) && operation != "dbt" {
                continue;
            }

            if operation == "dbt" {
                if start {
                    if optimize {
                        continue;
                    }
                } else {
                    start = true;
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

            can_print = !optimize || start;

            if needs_arg {
                if argument.is_none() {
                    return Err(CompilerError::MissingOperand(i + 1, operation));
                }
                let Ok(argument) = argument.unwrap().parse::<u8>() else {
                    return Err(CompilerError::InvalidOperand(
                        i + 1,
                        operation,
                        argument.unwrap().to_string(),
                    ));
                };
                result.push(opcode);
                result.push(argument);
                if !silent && can_print {
                    cprintln!(
                        "<blue>{}</> <yellow>{}</>\t-> <blue>{:02x}</> <yellow>{:02x}</>",
                        operation,
                        argument,
                        opcode,
                        argument
                    )
                }
            } else {
                if argument.is_some() {
                    warnings.push(format!(
                        "Warning line {}: Instruction '{}' does not need an operand",
                        i + 1,
                        operation
                    ));
                }
                result.push(opcode);
                result.push(0);
                if !silent && can_print {
                    cprintln!("<blue>{}</>\t-> <blue>{:02x}</> 00", operation, opcode)
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
    result[0] = (result.len() >> 8) as u8;
    result[1] = (result.len() & 0xff) as u8;
    for warning in warnings {
        cprintln!("<yellow>{}</>", warning);
    }
    println!("----------------\nsize: {}", result.len());
    Ok(result)
}
