use std::iter::Peekable;
use std::str::Chars;

use crate::token::Token;

#[derive(Debug)]
pub struct Lexer<'a> {
	input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
	pub fn new(input: &str) -> Lexer {
		Lexer {
			input: input.chars().peekable(),
		}
	}

	pub fn lex(mut self) -> Vec<Token> {
		let mut tokens = vec![];
		loop {
			let current_token = self.next_token();
			if current_token == Token::EOF || current_token == Token::ILLEGAL {
				tokens.push(current_token);
				break;
			} else {
				tokens.push(current_token);
			}
		}

		tokens
	}

	pub fn next_token(&mut self) -> Token {
		match self.input.peek() {
			Some(ch) => match ch {
				' ' | '\t' => {
					self.input.next();
					self.next_token()
				}
				'\n' => {
					self.input.next();
					self.next_token()
				}
				ch if ch.is_numeric() => self.read_numeric(),
				'+' => {
					self.input.next();
					Token::Add
				}
				'-' => {
					self.input.next();
					Token::Sub
				}
				'*' => {
					self.input.next();
					Token::Mul
				}
				'/' => {
					self.input.next();
					Token::Div
				}
				'(' => {
					self.input.next();
					Token::LParen
				}
				')' => {
					self.input.next();
					Token::RParen
				}
				ch if ch.is_alphabetic() => {
					self.read_identifier()
				}
				'=' => {
					self.input.next();
					match self.input.peek() {
						Some(c) => {
							if *c == '=' {
								self.input.next();
								Token::EQ
							} else {
								self.input.next();
								Token::ASSIGN
							}
						}
						_ => {
							self.next_token()
						}
					}
				}
				'<' => {
					self.input.next();
					Token::LT
				}
				'>' => {
					self.input.next();
					Token::GT
				}
				_ => {
					self.input.next();
					Token::ILLEGAL
				}
			},
			None => Token::EOF,
		}
	}

	fn read_numeric(&mut self) -> Token {
		let mut literal = String::new();

		loop {
			match self.input.peek() {
				Some(&ch) => {
					if ch.is_numeric() || ch == '.' {
						literal.push(ch);
						self.input.next();
					} else {
						break;
					}
				}
				_ => break,
			}
		}

		match literal.parse() {
			Ok(l) => Token::Num(l),
			Err(_e) => Token::ILLEGAL,
		}
	}

	fn read_identifier(&mut self) -> Token {
		let mut literal = String::new();

		loop {
			match self.input.peek() {
				Some(&ch) => {
					if !ch.is_alphabetic() {
						break;
					}
					if ch.is_ascii_whitespace() {
						// self.input.next();
						break;
					};
					literal.push(ch);
					self.input.next();
				},
				_ => break
			}
		};

		match literal.as_str() {
			"if" => Token::If,
			"then" => Token::Then,
			"else" => Token::Else,
			_ => Token::IDENTIFIER(literal),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_num() {
		let lexer = Lexer::new(r#"32"#);
		let tokens = lexer.lex();

		let expected = vec![Token::from(32), Token::EOF];
		assert_eq!(expected, tokens);
	}

	#[test]
	fn test_num_float() {
		let lexer = Lexer::new(r#"32.5"#);
		let tokens = lexer.lex();

		let expected = vec![Token::Num(32.5), Token::EOF];
		assert_eq!(expected, tokens);
	}

	#[test]
	fn test_num_whitespace() {
		let lexer = Lexer::new(r#"32 2"#);
		let tokens = lexer.lex();

		let expected = vec![Token::from(32), Token::from(2), Token::EOF];

		assert_eq!(expected, tokens);
	}

	#[test]
	fn test_num_operator() {
		let lexer = Lexer::new(r#"-+/*"#);
		let tokens = lexer.lex();

		let expected = vec![
			Token::Sub,
			Token::Add,
			Token::Div,
			Token::Mul,
			Token::EOF,
		];

		assert_eq!(expected, tokens);
	}

	#[test]
	fn test_assignment() {
		let lexer = Lexer::new(r#"a = 123"#);
		let tokens = lexer.lex();

		let expected = vec![
			Token::IDENTIFIER("a".to_string()),
			Token::ASSIGN,
			Token::from(123),
			Token::EOF,
		];

		assert_eq!(expected, tokens);
	}

	#[test]
	fn test_conditional() {
		let lexer = Lexer::new(r#"if 1 < 2 then 1 else 0"#);
		let tokens = lexer.lex();

		let expected = vec![
			Token::If,
			Token::from(1),
			Token::LT,
			Token::from(2),
			Token::Then,
			Token::from(1),
			Token::Else,
			Token::from(0),
			Token::EOF,
		];

		assert_eq!(expected, tokens);
	}
}

