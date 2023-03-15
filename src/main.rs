use clap::Parser;
use color_print::cprintln;
use map_macro::map;
use regex::Regex;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Mode of the program
    #[arg(short, long, default_value_t = false)]
    decompile: bool,

    /// Output location
    #[arg(short, long, default_value_t = String::from("DEFAULT_OUTPUT"))]
    output: String,

    /// Verbosity of progmem
    #[arg(short, long, default_value_t = false)]
    silent: bool,

    /// Optimization
    #[arg(short, long, default_value_t = false)]
    clean: bool,

    /// Input
    input: String,
}

fn compile(file_name: String, silent: bool, optimize: bool) -> Result<Vec<u8>, String> {
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
    let mut result = vec![];
    result.push(0);
    result.push(0);
    let re = Regex::new(r"(//|#|%).*").unwrap();
    let mut index_start = 0;
    let mut start = false;
    let mut end = false;
    let mut start_loop = false;
    let mut remove_instruction = vec![];
    let mut can_print;
    for (i, line) in reader.lines().enumerate() {
        if end && optimize {
            break;
        }
        let Ok(line) = line else {return Err("Impossible de lire la ligne".to_string())};
        if let (Some(comment), true) = (re.find(&line), (!silent)) {
            cprintln!("<green>{}</>", comment.as_str());
        }

        let line = re.replace_all(&line, "");
        if line.is_empty() {
            continue;
        }
        let instructions: Vec<&str> = line.split(';').map(|s| s.trim()).collect();
        if !instructions.last().unwrap().is_empty() {
            return Err(format!(
                "Line {}: Instruction need to end with a semicolon: {}",
                i, line
            ));
        } // verify if the instruction as a semicolon at the end
        for instruction in instructions {
            if instruction.is_empty() || (end && optimize) {
                continue;
            }
            can_print = !optimize || start;
            let remove_multiple_space = Regex::new("  ").unwrap();
            let instruction = remove_multiple_space.replace_all(instruction, " ");
            let mut iterator = instruction.split(' ');
            let operation = iterator.next().unwrap();
            let argument = iterator.next();
            if !(operation.to_lowercase() == operation || operation.to_uppercase() == operation) {
                return Err(format!(
                    "Line {}: Instruction need to be in upper case or lower case: {}",
                    i, instruction
                ));
            }
            let operation = operation.to_lowercase();
            let Some(set_instruction) = set_instructions.get(operation.as_str()) else {
                return Err(format!("Line {}: Unknown instruction: {}", i, operation))
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
                    return Err(format!("Line {}: You can't embed loop", i));
                }
                start_loop = true;
            }
            if operation == "fbc" {
                if !start_loop {
                    return Err(format!("Line {}: There is not loop set", i));
                }
                start_loop = false;
            }
            if set_instruction.1 {
                if argument.is_none() {
                    return Err(format!(
                        "Line {}: Instruction {} need argument",
                        i, operation
                    ));
                }
                let Ok(argument) = argument.unwrap().parse::<u8>() else {
                    return Err(format!(
                        "Line {}: Instruction {} have an invalid argument {}",
                        i, operation, argument.unwrap()
                    ));
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
                    return Err(format!(
                        "Line {}: Instruction {} does not need argument",
                        i, operation
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
        return Err("The program doesn't start".to_string());
    }
    if !end {
        return Err("The program doesn't end".to_string());
    }
    if start_loop {
        return Err("The loop doesn't end".to_string());
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
    if !silent {
        println!("----------------\nsize: {}", result.len());
    }
    Ok(result)
}

fn decompile(file_name: String, silent: bool) -> Result<String, String> {
    let set_instructions = map! {
        0b00000001 => ("dbt", false),
        0b00000010 => ("dbt", true),
        0b01000100 => ("dal", true),
        0b01000101 => ("det", false),
        0b01001000 => ("sgo", true),
        0b00001001 => ("sar", false),
        0b01100000 => ("mar", false),
        0b01100001 => ("mar", false),
        0b01100010 => ("mav", true),
        0b01100011 => ("mre", true),
        0b01100100 => ("trd", false),
        0b01100101 => ("trg", false),
        0b11000000 => ("dbc", true),
        0b11000001 => ("fbc", false),
        0b11111111 => ("fin", false)
    };
    let data = fs::read(file_name).expect("Unable to open file");
    if data.len() < 2 || data.len() % 2 == 1 {
        return Err("Invalid file format".to_string());
    }
    let mut data_iterator = data.iter();
    let first_element = *data_iterator.next().unwrap() as u16;
    let second_element = *data_iterator.next().unwrap() as u16;
    let size = ((first_element << 8) + second_element) as u16;
    let mut result = String::new();
    if data.len() as u16 != size {
        return Err("Invalid file format".to_string());
    }
    while let (Some(instruction), Some(argument)) = (data_iterator.next(), data_iterator.next()) {
        let Some(set_instruction) = set_instructions.get(instruction) else {
            return Err(format!("Unknown instruction: {}", instruction))
        };
        if set_instruction.1 {
            if !silent {
                cprintln!(
                    "<blue>{:02x}</> <yellow>{:02x}</> -> <blue>{}</> <yellow>{}</>;",
                    instruction,
                    argument,
                    set_instruction.0,
                    argument,
                )
            }
            result.push_str(&format!("{} {};\n", set_instruction.0, argument));
        } else {
            if !silent {
                cprintln!(
                    "<blue>{:02x}</> 00 -> <blue>{}</>;",
                    instruction,
                    set_instruction.0,
                )
            }
            result.push_str(&format!("{};\n", set_instruction.0));
        }
    }
    Ok(result)
}
fn main() {
    let args = Args::parse();
    if args.decompile {
        let output;
        if args.output == "DEFAULT_OUTPUT".to_string() {
            output = "out.txt".to_string();
        } else {
            output = args.output;
        }
        match decompile(args.input, args.silent) {
            Ok(code) => {
                fs::write(&output, &code).expect("Unable to write data");
                println!("Build done in file : {}", output);
            }
            Err(erreur) => cprintln!("<red>{}</>", erreur),
        }
    } else {
        let output;
        if args.output == "DEFAULT_OUTPUT".to_string() {
            output = "out".to_string();
        } else {
            output = args.output;
        }
        match compile(args.input, args.silent, args.clean) {
            Ok(bytecode) => {
                fs::write(&output, &bytecode).expect("Unable to write data");
                println!("Build done in file : {}", output)
            }
            Err(erreur) => cprintln!("<red>{}</>", erreur),
        }
    }
}
