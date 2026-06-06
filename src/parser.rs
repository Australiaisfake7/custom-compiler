use super::lexer::{Token, LexError};
use std::{convert::TryFrom, path::Component::ParentDir};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { expected: &'static str, got: Token },
    UnexpectedEof,
}

enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    LogicalAnd,
    LogicalOr,
    Assign,
}

enum UnaryOp {
    LNot,
    Negate,
}

enum LiteralType {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

enum Expression {
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>
    },
    Unary {
        operator: UnaryOp,
        right: Box<Expression>
    },
    Literal(LiteralType),
}

impl TryFrom<&Token> for BinaryOp {
    type Error = Token;

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        return match token {
            Token::Plus => Ok(BinaryOp::Add),
            Token::Minus => Ok(BinaryOp::Subtract),
            Token::Asterix => Ok(BinaryOp::Multiply),
            Token::Slash => Ok(BinaryOp::Divide),
            Token::Equal => Ok(BinaryOp::Equal),
            Token::NotEqual => Ok(BinaryOp::NotEqual),
            Token::Less => Ok(BinaryOp::Less),
            Token::LessEqual => Ok(BinaryOp::LessEqual),
            Token::Greater => Ok(BinaryOp::Greater),
            Token::GreaterEqual => Ok(BinaryOp::GreaterEqual),
            Token::LAnd => Ok(BinaryOp::LogicalAnd),
            Token::LOr => Ok(BinaryOp::LogicalOr),
            Token::Assign => Ok(BinaryOp::Assign),
            _ => Err(token.clone()),
        }
    }
}

impl TryFrom<&Token> for UnaryOp {
    type Error = Token;

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        return match token {
            Token::LNot => Ok(UnaryOp::LNot),
            Token::Minus => Ok(UnaryOp::Negate),
            _ => Err(token.clone()),
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize
}

impl Parser {
    fn peek(&self) -> &Token {
        return self.tokens.get(self.position).expect("Attempted to access noexistent token in parser");
    }
    fn previous(&self) -> &Token {
        return self.tokens.get(self.position - 1).expect("Attempted to access noexistent token in parser");
    }
    fn advance(&mut self) -> &Token {
        self.position += 1;
        return self.previous();
    }
    fn match_advance(&mut self, token: &Token) -> bool {
        let t: &Token = self.peek();
        if matches!(token, t) {
            self.advance();
            return true;
        }
        return false;
    }
    fn expression(&mut self) -> Box<Expression> {
        return self.equality();
    }
    fn equality(&mut self) -> Box<Expression> {
        let mut expr: Box<Expression> = self.inequality();

        while (self.match_advance(&Token::Equal) || self.match_advance(&Token::NotEqual)) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()) {
                Ok(op) => op,
                Err(token) => {
                    println!("Invalid token {:?} for binary operator", token);
                    BinaryOp::Null
                }
            };
            let right: Box<Expression> = self.inequality();
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return expr;
    }
    fn inequality(&mut self) -> Box<Expression> {
        let mut expr: Box<Expression> = self.term();

        while (self.match_advance(&Token::Less) || self.match_advance(&Token::LessEqual) || self.match_advance(&Token::Greater) || self.match_advance(&Token::GreaterEqual)) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()) {
                Ok(op) => op,
                Err(token) => {
                    println!("Invalid token {:?} for binary operator", token);
                    BinaryOp::Null
                }
            };
            let right: Box<Expression> = self.inequality();
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return expr;
    }
    fn term(&mut self) -> Box<Expression> {
        let mut expr: Box<Expression> = self.factor();

        while (self.match_advance(&Token::Plus) || self.match_advance(&Token::Minus)) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()) {
                Ok(op) => op,
                Err(token) => {
                    println!("Invalid token {:?} for binary operator", token);
                    BinaryOp::Null
                }
            };
            let right: Box<Expression> = self.inequality();
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return expr;
    }
    fn factor(&mut self) -> Box<Expression> {
        let mut expr: Box<Expression> = self.term();

        while (self.match_advance(&Token::Asterix) || self.match_advance(&Token::Slash)) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()) {
                Ok(op) => op,
                Err(token) => {
                    println!("Invalid token {:?} for binary operator", token);
                    BinaryOp::Null
                }
            };
            let right: Box<Expression> = self.unary();
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return expr;
    }
    fn unary(&mut self) -> Box<Expression> {
        if self.match_advance(&Token::LNot) || self.match_advance(&Token::Minus) {
            let op: UnaryOp = match UnaryOp::try_from(self.previous()) {
                Ok(op) => op,
                Err(token) => {
                    println!("Invalid token {:?} for unary operator", token);
                    UnaryOp::Null
                }
            };
            let right: Box<Expression> = self.inequality();
            return Box::new(Expression::Unary {
                operator: op,
                right: right
            });
        }
        return self.literal();
    }
    fn literal(&self) -> Box<Expression> {
        return Box::new(match self.advance() {
            Token::Bool(val) => Expression::Literal(LiteralType::Bool(*val)),
            Token::Null => Expression::Literal(LiteralType::Null),
            Token::Int(val) => Expression::Literal(LiteralType::Int(*val)),
            Token::Float(val) => Expression::Literal(LiteralType::Float(*val)),
            Token::String(val) => Expression::Literal(LiteralType::String(*val)),
            _ => println!()
        });
    }
}