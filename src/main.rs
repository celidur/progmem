use clap::Parser;
use color_print::cprintln;
use compiler::compile;
use decompiler::decompile;
use std::fs;

mod compiler;
mod decompiler;
mod errors;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Mode of the program
    #[arg(short, long, default_value_t = false)]
    decompile: bool,

    /// Output location
    #[arg(short, long, default_value_t = String::from("out"))]
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

fn main() {
    let args = Args::parse();
    if args.decompile {
        match decompile(args.input, args.silent) {
            Ok(code) => {
                fs::write(&args.output, &code).expect("Unable to write data");
                println!("Build done in file : {}", args.output);
            }
            Err(error) => cprintln!("<red>{}</>", error),
        }
    } else {
        match compile(args.input, args.silent, args.clean) {
            Ok(bytecode) => {
                fs::write(&args.output, &bytecode).expect("Unable to write data");
                println!("Build done in file : {}", args.output)
            }
            Err(error) => cprintln!("<red>{}</>", error),
        }
    }
}
