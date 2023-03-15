use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::env;

fn compile(file_name: &str, debug: bool) -> Vec<u8> {
    let file = File::open(file_name).expect("Unable to open file");
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
    // define result as a vector of u8
    let mut result = Vec::new() as Vec<u8>;
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
                    if debug {

                    }
                    result.push(opcode[0]);
                    if opcode[1] == 1 {
                        if debug { 
                            let number = j[instruction.len()..].trim().parse::<u8>().unwrap();
                            println!("{} {} -> {:02x} {:02x}", instruction, number, opcode[0], number)
                        }
                        result.push(j[instruction.len()..].trim().parse::<u8>().unwrap());
                    }
                    else if !j[instruction.len()..].trim().is_empty() {
                        panic!("Unknown instruction: {}", j); // verify if the instruction has a parameter
                    }
                    else if debug {
                        println!("{} -> {:02x}", instruction, opcode[0])
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
    result
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let debug = args.len() > 1 && args[1] == "-v";
    // recuperer le prochain argument
    let mode = &args[1 + debug as usize];
    if mode == "-o" {
        let file_output = &args[2 + debug as usize];
        let file_input = &args[3 + debug as usize];
        // if file exists, delete it
        let mut file = File::create(file_output).expect("Unable to create file");
        let resultat = compile(file_input, debug);
        // transform the vector of u8 to a string in hexadecimal
        let res = resultat.iter().map(|x| format!("{:02x}", x)).collect::<Vec<String>>().join(" ");
        file.write_all(&res.as_bytes()).expect("Unable to write data");
        
    }

}
