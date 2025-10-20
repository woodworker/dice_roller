use thiserror::Error;

pub type Result<T> = std::result::Result<T, DiceError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DiceError {
    #[error("Unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),

    #[error("Unexpected end of input")]
    UnexpectedEOF,

    #[error("Expected {expected}, found {found} at position {pos}")]
    UnexpectedToken {
        expected: String,
        found: String,
        pos: usize,
    },

    #[error("Invalid dice notation: {0}")]
    InvalidDiceNotation(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid number of dice: {0}")]
    InvalidDiceCount(i64),

    #[error("Invalid number of sides: {0}")]
    InvalidDiceSides(i64),

    #[error("Invalid modifier: {0}")]
    InvalidModifier(String),

    #[error("Empty expression")]
    EmptyExpression,

    #[error("Maximum iterations exceeded (possible infinite loop)")]
    MaxIterationsExceeded,

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(String),
}
