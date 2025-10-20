pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod roller;
pub mod result;

pub use error::{DiceError, Result};
pub use parser::Parser;
pub use roller::DiceRoller;
pub use result::RollResult;

/// Parse and evaluate a dice notation string
pub fn roll(input: &str) -> Result<RollResult> {
    let mut parser = Parser::new(input)?;
    let expr = parser.parse()?;
    let mut roller = DiceRoller::new();
    let mut result = roller.roll(&expr)?;
    result.expression = input.to_string();
    Ok(result)
}

/// Parse and evaluate a dice notation string with a specific seed
pub fn roll_with_seed(input: &str, seed: u64) -> Result<RollResult> {
    let mut parser = Parser::new(input)?;
    let expr = parser.parse()?;
    let mut roller = DiceRoller::with_seed(seed);
    let mut result = roller.roll(&expr)?;
    result.expression = input.to_string();
    Ok(result)
}
