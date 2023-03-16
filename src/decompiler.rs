use crate::errors::DecompilerError;
use color_print::cprintln;
use map_macro::map;
use std::fs;

pub fn decompile(file_name: String, silent: bool) -> Result<String, DecompilerError> {
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
        return Err(DecompilerError::FileFormat);
    }
    let mut data_iterator = data.iter();
    let first_element = *data_iterator.next().unwrap() as u16;
    let second_element = *data_iterator.next().unwrap() as u16;
    let size = ((first_element << 8) + second_element) as u16;
    let mut result = String::new();
    if data.len() as u16 != size {
        return Err(DecompilerError::FileFormat);
    }
    while let (Some(instruction), Some(argument)) = (data_iterator.next(), data_iterator.next()) {
        let Some(set_instruction) = set_instructions.get(instruction) else {
            return Err(DecompilerError::UnknownInstruction(*instruction))
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
