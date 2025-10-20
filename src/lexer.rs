use crate::error::{DiceError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(i64),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    StarStar, // **

    // Dice
    D,        // d
    DiceF,    // dF (FATE dice)

    // Comparison operators
    Greater,
    Less,
    Equal,

    // Modifiers
    Bang,          // !
    BangBang,      // !!
    BangP,         // !p (penetrating)
    K,             // k (keep)
    KH,            // kh (keep high)
    KL,            // kl (keep low)
    Drop,          // d (when used as drop)
    DH,            // dh (drop high)
    DL,            // dl (drop low)
    R,             // r (reroll)
    RO,            // ro (reroll once)
    F,             // f (failure)
    CS,            // cs (critical success)
    CF,            // cf (critical failure)
    S,             // s (sort)
    SA,            // sa (sort ascending)
    SD,            // sd (sort descending)

    // Functions
    Floor,
    Ceil,
    Round,
    Abs,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,

    // Labels
    Label(String),

    // End of input
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();

        Lexer {
            input: chars,
            position: 0,
            current_char,
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<i64> {
        let mut num_str = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        num_str
            .parse()
            .map_err(|_| DiceError::ParseError(format!("Invalid number: {}", num_str)))
    }

    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphabetic() {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        ident
    }

    fn read_label(&mut self) -> Result<String> {
        // Skip the opening [
        self.advance();

        let mut label = String::new();

        while let Some(ch) = self.current_char {
            if ch == ']' {
                self.advance();
                return Ok(label);
            } else {
                label.push(ch);
                self.advance();
            }
        }

        Err(DiceError::ParseError("Unclosed label".to_string()))
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let ch = match self.current_char {
            Some(c) => c,
            None => return Ok(Token::EOF),
        };

        // Numbers
        if ch.is_ascii_digit() {
            let num = self.read_number()?;
            return Ok(Token::Number(num));
        }

        // Identifiers and keywords
        if ch.is_alphabetic() {
            let ident = self.read_identifier().to_lowercase();

            let token = match ident.as_str() {
                "df" => Token::DiceF,
                "d" => Token::D,
                "f" => Token::F,
                "kh" => Token::KH,
                "kl" => Token::KL,
                "k" => Token::K,
                "dh" => Token::DH,
                "dl" => Token::DL,
                "ro" => Token::RO,
                "r" => Token::R,
                "cs" => Token::CS,
                "cf" => Token::CF,
                "sa" => Token::SA,
                "sd" => Token::SD,
                "s" => Token::S,
                "floor" => Token::Floor,
                "ceil" => Token::Ceil,
                "round" => Token::Round,
                "abs" => Token::Abs,
                _ => {
                    return Err(DiceError::ParseError(format!(
                        "Unknown identifier: {}",
                        ident
                    )))
                }
            };

            return Ok(token);
        }

        // Special characters and operators
        let token = match ch {
            '+' => {
                self.advance();
                Token::Plus
            }
            '-' => {
                self.advance();
                Token::Minus
            }
            '*' => {
                self.advance();
                if self.current_char == Some('*') {
                    self.advance();
                    Token::StarStar
                } else {
                    Token::Star
                }
            }
            '/' => {
                self.advance();
                Token::Slash
            }
            '%' => {
                self.advance();
                Token::Percent
            }
            '!' => {
                self.advance();
                if self.current_char == Some('!') {
                    self.advance();
                    Token::BangBang
                } else if self.current_char == Some('p') || self.current_char == Some('P') {
                    self.advance();
                    Token::BangP
                } else {
                    Token::Bang
                }
            }
            '>' => {
                self.advance();
                Token::Greater
            }
            '<' => {
                self.advance();
                Token::Less
            }
            '=' => {
                self.advance();
                Token::Equal
            }
            '(' => {
                self.advance();
                Token::LeftParen
            }
            ')' => {
                self.advance();
                Token::RightParen
            }
            '{' => {
                self.advance();
                Token::LeftBrace
            }
            '}' => {
                self.advance();
                Token::RightBrace
            }
            '[' => {
                // Check if this is a label or a bracket for inline rolls
                if self.peek(1).map_or(false, |c| c == '[') {
                    // It's [[ for inline roll
                    self.advance();
                    self.advance();
                    Token::LeftBracket
                } else {
                    // It's a label
                    let label = self.read_label()?;
                    Token::Label(label)
                }
            }
            ']' => {
                self.advance();
                if self.current_char == Some(']') {
                    self.advance();
                    Token::RightBracket
                } else {
                    return Err(DiceError::UnexpectedChar(']', self.position));
                }
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            _ => {
                return Err(DiceError::UnexpectedChar(ch, self.position));
            }
        };

        Ok(token)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            if token == Token::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("1d20+5");
        assert_eq!(lexer.next_token().unwrap(), Token::Number(1));
        assert_eq!(lexer.next_token().unwrap(), Token::D);
        assert_eq!(lexer.next_token().unwrap(), Token::Number(20));
        assert_eq!(lexer.next_token().unwrap(), Token::Plus);
        assert_eq!(lexer.next_token().unwrap(), Token::Number(5));
        assert_eq!(lexer.next_token().unwrap(), Token::EOF);
    }

    #[test]
    fn test_modifiers() {
        let mut lexer = Lexer::new("4d6kh3");
        assert_eq!(lexer.next_token().unwrap(), Token::Number(4));
        assert_eq!(lexer.next_token().unwrap(), Token::D);
        assert_eq!(lexer.next_token().unwrap(), Token::Number(6));
        assert_eq!(lexer.next_token().unwrap(), Token::KH);
        assert_eq!(lexer.next_token().unwrap(), Token::Number(3));
    }

    #[test]
    fn test_exploding() {
        let mut lexer = Lexer::new("3d6!");
        assert_eq!(lexer.next_token().unwrap(), Token::Number(3));
        assert_eq!(lexer.next_token().unwrap(), Token::D);
        assert_eq!(lexer.next_token().unwrap(), Token::Number(6));
        assert_eq!(lexer.next_token().unwrap(), Token::Bang);
    }

    #[test]
    fn test_fate_dice() {
        let mut lexer = Lexer::new("4dF");
        assert_eq!(lexer.next_token().unwrap(), Token::Number(4));
        assert_eq!(lexer.next_token().unwrap(), Token::DiceF);
    }
}
