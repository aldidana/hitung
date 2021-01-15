#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LParen,
    RParen,
    Add,
    Sub,
    Mul,
    Div,
    Num(f64),
    EOF,
    ILLEGAL,
    ASSIGN,
    IDENTIFIER(String),
    If,
    Then,
    Else,
    EQ,
    LT,
    GT,
}

impl From<i32> for Token {
    fn from(n: i32) -> Self {
        Token::Num(n as f64)
    }
}

impl Token {
    //Left Binding Power
    pub fn lbp(&self) -> usize {
        match *self {
            Token::Add => 10,
            Token::Sub => 10,
            Token::Mul => 20,
            Token::Div => 20,
            Token::LParen => 99,
            Token::ASSIGN => 100,
            Token::RParen => 0,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_token_add() {
        let actual = Token::Add.lbp();
        let expected: usize = 10;

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_token_sub() {
        let actual = Token::Sub.lbp();
        let expected: usize = 10;

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_token_mul() {
        let actual = Token::Mul.lbp();
        let expected: usize = 20;

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_token_div() {
        let actual = Token::Div.lbp();
        let expected: usize = 20;

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_token_open_paren() {
        let actual = Token::LParen.lbp();
        let expected: usize = 99;

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_token_close_paren() {
        let actual = Token::RParen.lbp();
        let expected: usize = 0;

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_token_assignment() {
        let actual = Token::ASSIGN.lbp();
        let expected: usize = 100;

        assert_eq!(expected, actual);
    }
}
