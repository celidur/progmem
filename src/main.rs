use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let file = File::open("progmemTest.txt").expect("Unable to open file");
    let reader = BufReader::new(file);
    let set_instructions = [
        ("dbt", [0b00000001, 0]),
        ("att", [0b00000010, 1]),
        ("dal", [0b01000100, 1]),
        ("det", [0b01000101, 0]),
        ("sgo", [0b01001000, 1]),
        ("sar", [0b00001001, 0]),
        ("mar", [0b01100000, 0]),
        ("mav", [0b01100010, 1]),
        ("mre", [0b01100011, 1]),
        ("trd", [0b01100100, 0]),
        ("trg", [0b01100101, 0]),
        ("dbc", [0b11000000, 1]),
        ("fbc", [0b11000001, 0]),
        ("fin", [0b11111111, 0]),
    ];

    let mut result = String::new();
    for line in reader.lines() {
        let line = line.unwrap().splitn(2, |c| c == '/' || c == '#' || c == '%').next().unwrap().to_string();
        if line.is_empty() {continue}
        let list = line.split(';').map(|s| s.trim()).collect::<Vec<&str>>();
        if !list.last().unwrap().is_empty(){panic!("Unknown instruction: {}", line);} // verify if the instruction as a semicolon at the end
        for j in list{
            if j.is_empty(){continue}
            if !(j.to_lowercase() == j || j.to_uppercase() == j) {
                panic!("Unknown instruction: {}", j); // verify if the instruction is in only uppercase or only lowercase
            }
            let mut found_instruction = false;
            for (instruction, opcode) in &set_instructions {
                if j.to_lowercase().starts_with(instruction) {
                    result += &format!("{0:02x} ", opcode[0]);
                    if opcode[1] == 1 {
                        result += &format!("{0:02x} ", j[instruction.len()..].trim().parse::<u8>().unwrap());
                    }
                    else if !j[instruction.len()..].trim().is_empty() {
                        panic!("Unknown instruction: {}", j); // verify if the instruction has a parameter
                    }
                    found_instruction = true;
                    break;
                }
            }
            if !found_instruction {
                panic!("Unknown instruction: {}", j); // verify if the instruction is in the set
            }
        }
    }
    println!("{}", result);
}
