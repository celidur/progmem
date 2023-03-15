use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output location
    #[arg(short, long)]
    output: String,

    /// Verbosity of progmem
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Input
    input: String,
}

fn compile(file_name: String, debug: bool) -> Vec<u8> {
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
    let mut result = vec![];
    result.push(0);
    result.push(0);
    for line in reader.lines() {
        let Ok(line) = line else {continue;};
        let re = Regex::new(r"(//|#|%).*").unwrap();
        let l = re
            .find(&line)
            .map_or("".to_string(), |m| m.as_str().to_string());
        if debug && !l.is_empty() {
            println!("{}", l);
        }
        let line = line
            .splitn(2, |c| c == '/' || c == '#' || c == '%')
            .next()
            .unwrap()
            .to_string();
        if line.is_empty() {
            continue;
        }
        let list = line.split(';').map(|s| s.trim()).collect::<Vec<&str>>();
        if !list.last().unwrap().is_empty() {
            panic!("Unknown instruction: {}", line);
        } // verify if the instruction as a semicolon at the end
        for j in list {
            if j.is_empty() {
                continue;
            }
            if !(j.to_lowercase() == j || j.to_uppercase() == j) {
                panic!("Instruction need to be in upper case or lower case: {}", j);
            }
            let mut found_instruction = false;
            for (instruction, opcode) in &set_instructions {
                if j.to_lowercase().starts_with(instruction) {
                    result.push(opcode[0]);

                    // println!("{} ",j);
                    if opcode[1] == 0 {
                        if !j[instruction.len()..].trim().is_empty() {
                            panic!("the instruction can't have opcode : {}", j);
                        }
                        if debug {
                            println!("{} -> {:02x} 00", instruction, opcode[0])
                        }
                        result.push(0);
                    } else if opcode[1] == 1 {
                        if debug {
                            let number = j[instruction.len()..].trim().parse::<u8>().unwrap();
                            println!(
                                "{} {} -> {:02x} {:02x}",
                                instruction, number, opcode[0], number
                            )
                        }
                        result.push(j[instruction.len()..].trim().parse::<u8>().unwrap());
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
    result[0] = (result.len() >> 8) as u8;
    result[1] = (result.len() & 0xff) as u8;
    if debug {
        println!("----------------------\nsize: {}", result.len());
    }
    result
}
fn main() {
    let args = Args::parse();
    // recuperer le prochain argument
    // if file exists, delete it
    let mut file = File::create(args.output).expect("Unable to create file");
    let resultat = compile(args.input, args.verbose);
    file.write_all(&resultat).expect("Unable to write data");
}
