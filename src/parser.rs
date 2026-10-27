use crate::ast::*;
use crate::error::{DiceError, Result};
use crate::lexer::{Lexer, Token};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    current_token: Token,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;

        if tokens.is_empty() {
            return Err(DiceError::EmptyExpression);
        }

        let current_token = tokens[0].clone();

        Ok(Parser {
            tokens,
            position: 0,
            current_token,
        })
    }

    fn advance(&mut self) {
        self.position += 1;
        if self.position < self.tokens.len() {
            self.current_token = self.tokens[self.position].clone();
        } else {
            self.current_token = Token::EOF;
        }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        let expr = self.parse_expression()?;

        if self.current_token != Token::EOF {
            return Err(DiceError::UnexpectedToken {
                expected: "end of expression".to_string(),
                found: format!("{:?}", self.current_token),
                pos: self.position,
            });
        }

        Ok(expr)
    }

    /// Skip labels on values that do not keep them, e.g. `+2[Agility]`
    fn skip_labels(&mut self) {
        while matches!(self.current_token, Token::Label(_)) {
            self.advance();
        }
    }

    // Expression -> Term ((+ | -) Term)*
    fn parse_expression(&mut self) -> Result<Expr> {
        let mut left = self.parse_term()?;

        while matches!(self.current_token, Token::Plus | Token::Minus) {
            let op = match self.current_token {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.parse_term()?;
            left = Expr::binary_op(op, left, right);
        }

        Ok(left)
    }

    // Term -> Exponent ((* | / | %) Exponent)*
    fn parse_term(&mut self) -> Result<Expr> {
        let mut left = self.parse_exponent()?;

        while matches!(
            self.current_token,
            Token::Star | Token::Slash | Token::Percent
        ) {
            let op = match self.current_token {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.parse_exponent()?;
            left = Expr::binary_op(op, left, right);
        }

        Ok(left)
    }

    // Exponent -> Unary (** Unary)*
    fn parse_exponent(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;

        while matches!(self.current_token, Token::StarStar) {
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::binary_op(BinaryOperator::Exponent, left, right);
        }

        Ok(left)
    }

    // Unary -> (+ | -) Unary | Primary
    fn parse_unary(&mut self) -> Result<Expr> {
        match self.current_token {
            Token::Plus => {
                self.advance();
                self.parse_unary()
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOperator::Negate,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    // Primary -> Number | Dice | FateDice | Function | Group | Inline | (Expression)
    fn parse_primary(&mut self) -> Result<Expr> {
        match &self.current_token {
            Token::Number(n) => {
                let num = *n;
                self.advance();

                // Check if this is a dice roll (e.g., 1d20 or 4dF)
                if matches!(self.current_token, Token::D | Token::DiceF) {
                    self.parse_dice_from_number(num)
                } else {
                    self.skip_labels();
                    Ok(Expr::Literal(num))
                }
            }
            Token::D => {
                // Dice without explicit count (e.g., d20 means 1d20)
                self.parse_dice_from_number(1)
            }
            Token::DiceF => {
                // FATE dice without explicit count
                self.parse_fate_dice_from_number(1)
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;

                if self.current_token != Token::RightParen {
                    return Err(DiceError::UnexpectedToken {
                        expected: ")".to_string(),
                        found: format!("{:?}", self.current_token),
                        pos: self.position,
                    });
                }
                self.advance();

                // Check if this is a computed dice roll
                if matches!(self.current_token, Token::D | Token::DiceF) {
                    self.parse_dice_from_expr(expr)
                } else {
                    Ok(expr)
                }
            }
            Token::LeftBrace => self.parse_group(),
            Token::LeftBracket => self.parse_inline(),
            Token::Floor | Token::Ceil | Token::Round | Token::Abs => self.parse_function(),
            _ => Err(DiceError::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("{:?}", self.current_token),
                pos: self.position,
            }),
        }
    }

    fn parse_dice_from_number(&mut self, count: i64) -> Result<Expr> {
        self.parse_dice_from_expr(Expr::Literal(count))
    }

    fn parse_dice_from_expr(&mut self, count_expr: Expr) -> Result<Expr> {
        if self.current_token == Token::DiceF {
            self.advance();
            let modifiers = self.parse_modifiers()?;
            return Ok(Expr::fate_dice(count_expr, modifiers));
        }

        // Regular dice
        if self.current_token != Token::D {
            return Err(DiceError::UnexpectedToken {
                expected: "d".to_string(),
                found: format!("{:?}", self.current_token),
                pos: self.position,
            });
        }
        self.advance();

        // Parse number of sides
        let sides_expr = match &self.current_token {
            Token::Number(n) => {
                let num = *n;
                self.advance();
                Expr::Literal(num)
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                if self.current_token != Token::RightParen {
                    return Err(DiceError::UnexpectedToken {
                        expected: ")".to_string(),
                        found: format!("{:?}", self.current_token),
                        pos: self.position,
                    });
                }
                self.advance();
                expr
            }
            _ => {
                return Err(DiceError::InvalidDiceNotation(
                    "Expected number of sides".to_string(),
                ))
            }
        };

        let modifiers = self.parse_modifiers()?;

        Ok(Expr::dice(count_expr, sides_expr, modifiers))
    }

    fn parse_fate_dice_from_number(&mut self, count: i64) -> Result<Expr> {
        self.advance(); // consume dF
        let modifiers = self.parse_modifiers()?;
        Ok(Expr::fate_dice(Expr::Literal(count), modifiers))
    }

    fn parse_modifiers(&mut self) -> Result<Vec<Modifier>> {
        let mut modifiers = Vec::new();

        loop {
            let modifier = match &self.current_token {
                Token::Bang => {
                    self.advance();
                    let cp = self.parse_compare_point_optional()?;
                    Modifier::Exploding(cp)
                }
                Token::BangBang => {
                    self.advance();
                    let cp = self.parse_compare_point_optional()?;
                    Modifier::Compounding(cp)
                }
                Token::BangP => {
                    self.advance();
                    let cp = self.parse_compare_point_optional()?;
                    Modifier::Penetrating(cp)
                }
                Token::K | Token::KH => {
                    self.advance();
                    let n = self.expect_number()?;
                    Modifier::KeepHigh(n as usize)
                }
                Token::KL => {
                    self.advance();
                    let n = self.expect_number()?;
                    Modifier::KeepLow(n as usize)
                }
                Token::DH => {
                    self.advance();
                    let n = self.expect_number()?;
                    Modifier::DropHigh(n as usize)
                }
                Token::DL => {
                    self.advance();
                    let n = self.expect_number()?;
                    Modifier::DropLow(n as usize)
                }
                Token::R => {
                    self.advance();
                    let cp = self.parse_compare_point_optional_default_one()?;
                    Modifier::Reroll(cp)
                }
                Token::RO => {
                    self.advance();
                    let cp = self.parse_compare_point_optional_default_one()?;
                    Modifier::RerollOnce(cp)
                }
                Token::Greater | Token::Less | Token::Equal => {
                    let cp = self.parse_compare_point()?;
                    Modifier::Success(cp)
                }
                Token::F => {
                    self.advance();
                    let cp = self.parse_compare_point_optional()?;
                    Modifier::Failure(cp)
                }
                Token::CS => {
                    self.advance();
                    let cp = self.parse_compare_point()?;
                    Modifier::CriticalSuccess(cp)
                }
                Token::CF => {
                    self.advance();
                    let cp = self.parse_compare_point()?;
                    Modifier::CriticalFailure(cp)
                }
                Token::S | Token::SA => {
                    self.advance();
                    Modifier::SortAscending
                }
                Token::SD => {
                    self.advance();
                    Modifier::SortDescending
                }
                Token::Label(label) => {
                    let label = label.clone();
                    self.advance();
                    Modifier::Label(label)
                }
                _ => break,
            };

            modifiers.push(modifier);
        }

        Ok(modifiers)
    }

    fn parse_compare_point(&mut self) -> Result<ComparePoint> {
        let op = match &self.current_token {
            Token::Greater => CompareOp::GreaterOrEqual,
            Token::Less => CompareOp::LessOrEqual,
            Token::Equal => CompareOp::Equal,
            _ => {
                return Err(DiceError::UnexpectedToken {
                    expected: "comparison operator".to_string(),
                    found: format!("{:?}", self.current_token),
                    pos: self.position,
                })
            }
        };
        self.advance();

        let value = self.expect_number()?;
        Ok(ComparePoint::new(op, value))
    }

    fn parse_compare_point_optional(&mut self) -> Result<ComparePoint> {
        match &self.current_token {
            Token::Greater | Token::Less | Token::Equal => self.parse_compare_point(),
            Token::Number(n) => {
                let value = *n;
                self.advance();
                Ok(ComparePoint::new(CompareOp::Equal, value))
            }
            _ => Ok(ComparePoint::default_explode()),
        }
    }

    fn parse_compare_point_optional_default_one(&mut self) -> Result<ComparePoint> {
        match &self.current_token {
            Token::Greater | Token::Less | Token::Equal => self.parse_compare_point(),
            Token::Number(n) => {
                let value = *n;
                self.advance();
                Ok(ComparePoint::new(CompareOp::Equal, value))
            }
            _ => Ok(ComparePoint::new(CompareOp::Equal, 1)),
        }
    }

    fn expect_number(&mut self) -> Result<i64> {
        match &self.current_token {
            Token::Number(n) => {
                let num = *n;
                self.advance();
                Ok(num)
            }
            _ => Err(DiceError::UnexpectedToken {
                expected: "number".to_string(),
                found: format!("{:?}", self.current_token),
                pos: self.position,
            }),
        }
    }

    fn parse_function(&mut self) -> Result<Expr> {
        let func_name = match &self.current_token {
            Token::Floor => FunctionName::Floor,
            Token::Ceil => FunctionName::Ceil,
            Token::Round => FunctionName::Round,
            Token::Abs => FunctionName::Abs,
            _ => {
                return Err(DiceError::UnexpectedToken {
                    expected: "function name".to_string(),
                    found: format!("{:?}", self.current_token),
                    pos: self.position,
                })
            }
        };
        self.advance();

        if self.current_token != Token::LeftParen {
            return Err(DiceError::UnexpectedToken {
                expected: "(".to_string(),
                found: format!("{:?}", self.current_token),
                pos: self.position,
            });
        }
        self.advance();

        let arg = self.parse_expression()?;

        if self.current_token != Token::RightParen {
            return Err(DiceError::UnexpectedToken {
                expected: ")".to_string(),
                found: format!("{:?}", self.current_token),
                pos: self.position,
            });
        }
        self.advance();

        Ok(Expr::Function {
            name: func_name,
            arg: Box::new(arg),
        })
    }

    fn parse_group(&mut self) -> Result<Expr> {
        self.advance(); // consume {

        let mut rolls = Vec::new();

        loop {
            let expr = self.parse_expression()?;
            rolls.push(expr);

            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }

        if self.current_token != Token::RightBrace {
            return Err(DiceError::UnexpectedToken {
                expected: "}".to_string(),
                found: format!("{:?}", self.current_token),
                pos: self.position,
            });
        }
        self.advance();

        let modifiers = self.parse_group_modifiers()?;

        Ok(Expr::Group { rolls, modifiers })
    }

    fn parse_group_modifiers(&mut self) -> Result<Vec<GroupModifier>> {
        let mut modifiers = Vec::new();

        loop {
            let modifier = match &self.current_token {
                Token::K | Token::KH => {
                    self.advance();
                    let n = self.expect_number()?;
                    GroupModifier::KeepHigh(n as usize)
                }
                Token::KL => {
                    self.advance();
                    let n = self.expect_number()?;
                    GroupModifier::KeepLow(n as usize)
                }
                Token::DH => {
                    self.advance();
                    let n = self.expect_number()?;
                    GroupModifier::DropHigh(n as usize)
                }
                Token::DL => {
                    self.advance();
                    let n = self.expect_number()?;
                    GroupModifier::DropLow(n as usize)
                }
                Token::Greater | Token::Less | Token::Equal => {
                    let cp = self.parse_compare_point()?;
                    GroupModifier::Success(cp)
                }
                Token::F => {
                    self.advance();
                    let cp = self.parse_compare_point_optional()?;
                    GroupModifier::Failure(cp)
                }
                _ => break,
            };

            modifiers.push(modifier);
        }

        Ok(modifiers)
    }

    fn parse_inline(&mut self) -> Result<Expr> {
        self.advance(); // consume [[

        let expr = self.parse_expression()?;

        if self.current_token != Token::RightBracket {
            return Err(DiceError::UnexpectedToken {
                expected: "]]".to_string(),
                found: format!("{:?}", self.current_token),
                pos: self.position,
            });
        }
        self.advance();

        // Check for label
        let label = if let Token::Label(l) = &self.current_token {
            let label = Some(l.clone());
            self.advance();
            label
        } else {
            None
        };

        Ok(Expr::Inline {
            expr: Box::new(expr),
            label,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let mut parser = Parser::new("1d20+5").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr, Expr::BinaryOp { .. }));
    }

    #[test]
    fn test_parse_keep_high() {
        let mut parser = Parser::new("4d6kh3").unwrap();
        let expr = parser.parse().unwrap();
        if let Expr::Dice { modifiers, .. } = expr {
            assert_eq!(modifiers.len(), 1);
            assert!(matches!(modifiers[0], Modifier::KeepHigh(3)));
        } else {
            panic!("Expected Dice expression");
        }
    }

    #[test]
    fn test_parse_exploding() {
        let mut parser = Parser::new("3d6!").unwrap();
        let expr = parser.parse().unwrap();
        if let Expr::Dice { modifiers, .. } = expr {
            assert_eq!(modifiers.len(), 1);
            assert!(matches!(modifiers[0], Modifier::Exploding(_)));
        } else {
            panic!("Expected Dice expression");
        }
    }

    #[test]
    fn test_parse_fate_dice() {
        let mut parser = Parser::new("4dF").unwrap();
        let expr = parser.parse().unwrap();
        assert!(matches!(expr, Expr::FateDice { .. }));
    }

    #[test]
    fn test_parse_labels_on_numbers() {
        let mut parser = Parser::new("1d20+2[Strength]+1d4[Bless]").unwrap();
        assert!(parser.parse().is_ok());
    }

    #[test]
    fn test_parse_rejects_trailing_tokens() {
        let mut parser = Parser::new("1d20+2kh").unwrap();
        assert!(parser.parse().is_err());
    }
}
