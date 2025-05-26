use crate::errors::DecompilerError;
use color_print::cprintln;
use std::fs;

fn instruction_name(opcode: u8) -> Option<(&'static str, bool)> {
    match opcode {
        0b0000_0001 => Some(("dbt", false)),
        0b0000_0010 => Some(("att", true)),
        0b0100_0100 => Some(("dal", true)),
        0b0100_0101 => Some(("det", false)),
        0b0100_1000 => Some(("sgo", true)),
        0b0000_1001 => Some(("sar", false)),
        0b0110_0000 => Some(("mar", false)),
        0b0110_0001 => Some(("mar", false)),
        0b0110_0010 => Some(("mav", true)),
        0b0110_0011 => Some(("mre", true)),
        0b0110_0100 => Some(("trd", false)),
        0b0110_0101 => Some(("trg", false)),
        0b1100_0000 => Some(("dbc", true)),
        0b1100_0001 => Some(("fbc", false)),
        0b1111_1111 => Some(("fin", false)),
        _ => None,
    }
}

pub fn decompile(file_name: String, silent: bool) -> Result<String, DecompilerError> {
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
        let Some((name, needs_arg)) = instruction_name(*instruction) else {
            return Err(DecompilerError::UnknownInstruction(*instruction));
        };
        if needs_arg {
            if !silent {
                cprintln!(
                    "<blue>{:02x}</> <yellow>{:02x}</> -> <blue>{}</> <yellow>{}</>;",
                    instruction,
                    argument,
                    name,
                    argument,
                )
            }
            result.push_str(&format!("{} {};\n", name, argument));
        } else {
            if !silent {
                cprintln!("<blue>{:02x}</> 00 -> <blue>{}</>;", instruction, name,)
            }
            result.push_str(&format!("{};\n", name));
        }
    }
    Ok(result)
}
