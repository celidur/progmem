use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Error line {0}: Impossible to read line")]
    ReadLine(usize),
    #[error("Error line {0}: Instruction '{1}' needs to end with a semicolon")]
    LackSemiColon(usize, String),
    #[error("Error line {0}: Instruction '{1}' needs to be in upper case or lower case")]
    CaseError(usize, String),
    #[error("Error line {0}: Unknown instruction {1}")]
    UnknownInstruction(usize, String),
    #[error("Error line {0}: There is no loop before")]
    NoLoop(usize),
    #[error("Error line {0}: You can't embed loop")]
    MultipleLoop(usize),
    #[error("Error line {0}: Instruction '{1}' need an operand")]
    MissingOperand(usize, String),
    #[error("Error line {0}: Instruction '{1}' have an invalid operand {2}")]
    InvalidOperand(usize, String, String),
    #[error("The program doesn't start")]
    MissingStart,
    #[error("The program doesn't end")]
    MissingEnd,
    #[error("The loop doesn't end")]
    MissingLoopEnd,
}

#[derive(Error, Debug)]
pub enum DecompilerError {
    #[error("Invalid file format")]
    FileFormat,
    #[error("Unknown instruction: {0}")]
    UnknownInstruction(u8),
}
