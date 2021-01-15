use std::iter::Iterator;
use std::iter::Peekable;
use std::vec::IntoIter;

use crate::expression::Expression;
use crate::token::Token;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn handle_next(&mut self) -> Result<Token, String> {
        self.tokens.next().ok_or("Error get next token".to_string())
    }

    //Null Denotation
    pub fn nud(&mut self, token: Token) -> Result<Expression, String> {
        match token {
            Token::ILLEGAL => Err("Input not supported".to_string()),
            Token::IDENTIFIER(i) => Ok(Expression::Variable(i)),
            Token::Num(n) => Ok(Expression::Num(n)),
            Token::Sub | Token::Add => {
                let tok = self.handle_next()?;
                match tok {
                    Token::Num(n) => Ok(Expression::Unary(token, Box::new(Expression::Num(n)))),
                    _ => Err("Input not supported".to_string()),
                }
            }
            Token::LParen => {
                let mut parenthesis = Vec::new();
                let mut counter: usize = 1;

                while let Some(token) = self.tokens.next() {
                    match &token {
                        Token::LParen => counter += 1,
                        Token::RParen => {
                            match counter {
                                1 => {
                                    return Parser::new(parenthesis)
                                        .expr(0)
                                        .map(|t| Expression::Paren(Box::new(t)));
                                }
                                0 => return Err("Unmatched closing paren".to_string()),
                                _ => {}
                            };
                            counter -= 1;
                        }
                        _ => {}
                    };
                    parenthesis.push(token);
                }

                Err("Unmatched closing paren".to_string())
            }
            Token::RParen => Err("Unmatched closing paren".to_string()),
            Token::If => {
                let lhs = self.handle_next()?;
                let cmp = self.handle_next()?;
                let rhs = self.handle_next()?;

                let lhs = self.nud(lhs)?;
                let rhs = self.nud(rhs)?;

                let left = Box::new(Expression::Binary(Box::new(lhs), cmp, Box::new(rhs)));

                let _then = self.handle_next()?;
                let then_branch = self.handle_next()?;
                let then_expression = self.nud(then_branch)?;

                let _else_if = self.handle_next()?;
                let else_branch = self.handle_next()?;
                let else_expression = self.nud(else_branch)?;

                Ok(Expression::Conditional(
                    left,
                    Box::new(then_expression),
                    Box::new(else_expression),
                ))
            }
            _ => Err(format!("Token {:?} error", token)),
        }
    }

    //Left Denotation
    pub fn led(&mut self, bp: usize, left: Expression, token: Token) -> Result<Expression, String> {
        match token {
            Token::Add | Token::Sub | Token::Mul | Token::Div | Token::ASSIGN => {
                let rhs = self.expr(bp)?;
                Ok(Expression::Binary(Box::new(left), token, Box::new(rhs)))
            }
            _ => Err(format!("Token {:?} error", token)),
        }
    }

    pub fn expr(&mut self, rbp: usize) -> Result<Expression, String> {
        let first_token = self.handle_next()?;
        let mut left = self.nud(first_token)?;

        while let Some(peeked) = self.tokens.peek() {
            if *peeked == Token::ILLEGAL {
                return Err("Input not supported".to_string());
            }

            if rbp >= peeked.lbp() {
                break;
            }

            let op = self.handle_next()?;
            left = self.led(op.lbp(), left, op)?;
        }

        Ok(left)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nud() {
        let tokens = vec![
            Token::Sub,
            Token::from(3),
            Token::Mul,
            Token::from(2),
            Token::EOF,
        ];
        let expression = Parser::new(tokens).expr(0).unwrap();

        let expected = Expression::Binary(
            Box::new(Expression::Unary(Token::Sub, Box::new(Expression::from(3)))),
            Token::Mul,
            Box::new(Expression::from(2)),
        );

        assert_eq!(expected, expression);
    }

    #[test]
    fn test_error() {
        let tokens = vec![Token::Mul, Token::from(2), Token::EOF];
        let expression = Parser::new(tokens).expr(0).map_err(|e| e);

        let expected = Err(String::from("Token Mul error"));

        assert_eq!(expected, expression);
    }

    #[test]
    fn test_binary() {
        let tokens = vec![Token::from(3), Token::Div, Token::from(2), Token::EOF];
        let expression = Parser::new(tokens).expr(0).unwrap();

        let expected = Expression::Binary(
            Box::new(Expression::from(3)),
            Token::Div,
            Box::new(Expression::from(2)),
        );

        assert_eq!(expected, expression);
    }

    #[test]
    fn test_unary() {
        let tokens = vec![Token::Sub, Token::from(2), Token::EOF];
        let expression = Parser::new(tokens).expr(0).unwrap();

        let expected = Expression::Unary(Token::Sub, Box::new(Expression::from(2)));

        assert_eq!(expected, expression);
    }

    #[test]
    fn test_precedence_add_and_mul() {
        let tokens = vec![
            Token::from(3),
            Token::Add,
            Token::from(2),
            Token::Mul,
            Token::from(2),
            Token::EOF,
        ];
        let expression = Parser::new(tokens).expr(0).unwrap();

        let expected = Expression::Binary(
            Box::new(Expression::from(3)),
            Token::Add,
            Box::new(Expression::Binary(
                Box::new(Expression::from(2)),
                Token::Mul,
                Box::new(Expression::from(2)),
            )),
        );

        assert_eq!(expected, expression);
    }

    #[test]
    fn test_precedence_add_and_sub() {
        let tokens = vec![
            Token::from(3),
            Token::Add,
            Token::from(2),
            Token::Sub,
            Token::from(2),
            Token::EOF,
        ];
        let expression = Parser::new(tokens).expr(0).unwrap();

        let expected = Expression::Binary(
            Box::new(Expression::Binary(
                Box::new(Expression::from(3)),
                Token::Add,
                Box::new(Expression::from(2)),
            )),
            Token::Sub,
            Box::new(Expression::from(2)),
        );

        assert_eq!(expected, expression);
    }

    #[test]
    fn test_assignment() {
        let tokens = vec![
            Token::IDENTIFIER("a".to_string()),
            Token::ASSIGN,
            Token::from(2),
        ];

        let expression = Parser::new(tokens).expr(0).unwrap();

        let expected = Expression::Binary(
            Box::new(Expression::Variable("a".to_string())),
            Token::ASSIGN,
            Box::new(Expression::from(2)),
        );

        assert_eq!(expected, expression);
    }

    #[test]
    fn test_if_then_else() {
        let tokens = vec![
            Token::If,
            Token::from(1),
            Token::LT,
            Token::from(9),
            Token::Then,
            Token::from(1),
            Token::Else,
            Token::from(0),
        ];

        let expression = Parser::new(tokens).expr(0).unwrap();

        let expected = Expression::Conditional(
            Box::new(Expression::Binary(
                Box::new(Expression::from(1)),
                Token::LT,
                Box::new(Expression::from(9)),
            )),
            Box::new(Expression::from(1)),
            Box::new(Expression::from(0)),
        );

        assert_eq!(expected, expression);
    }
}
