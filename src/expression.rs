use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Num(f64),
    Unary(Token, Box<Expression>),
    Binary(Box<Expression>, Token, Box<Expression>),
    Paren(Box<Expression>),
    Variable(String),
    Conditional(Box<Expression>, Box<Expression>, Box<Expression>),
}

impl From<isize> for Expression {
    fn from(n: isize) -> Self {
        Expression::Num(n as f64)
    }
}
