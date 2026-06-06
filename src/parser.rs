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
    fn peek(&self) -> Result<&Token, ParseError> {
        self.tokens.get(self.position).ok_or(ParseError::UnexpectedEof)
    }
    fn previous(&self) -> Result<&Token, ParseError> {
        self.tokens.get(self.position - 1).ok_or(ParseError::UnexpectedEof)
    }
    fn advance(&mut self) -> Result<&Token, ParseError> {
        self.position += 1;
        self.previous()
    }
    fn match_advance(&mut self, tokens: &[Token]) -> bool {
        let disc = match self.peek() {
            Ok(v) => std::mem::discriminant(v),
            Err(_) => return false,
        };

        for token in tokens {
            if std::mem::discriminant(token) == disc {
                self.advance();
                return true;
            }
        }
        return false;
    }
    fn expression(&mut self) -> Result<Box<Expression>, ParseError> {
        return self.equality();
    }
    fn equality(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.inequality()?;

        while (self.match_advance(&[Token::Equal, Token::NotEqual])) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Equality", got: token });
                }
            };
            let right: Box<Expression> = self.inequality()?;
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return Ok(expr);
    }
    fn inequality(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.term()?;

        while (self.match_advance(&[Token::Less, Token::LessEqual, Token::Greater, Token::GreaterEqual])) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Inequality", got: token });
                }
            };
            let right: Box<Expression> = self.term()?;
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return Ok(expr);
    }
    fn term(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.factor()?;

        while (self.match_advance(&[Token::Plus, Token::Minus])) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Term", got: token });
                }
            };
            let right: Box<Expression> = self.factor()?;
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return Ok(expr);
    }
    fn factor(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.unary()?;

        while (self.match_advance(&[Token::Asterix, Token::Slash])) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Factor", got: token });
                }
            };
            let right: Box<Expression> = self.unary()?;
            expr = Box::new(Expression::Binary {
                left: expr,
                operator: op,
                right: right
            });
        }

        return Ok(expr);
    }
    fn unary(&mut self) -> Result<Box<Expression>, ParseError> {
        if self.match_advance(&[Token::LNot, Token::Minus]) {
            let op: UnaryOp = match UnaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { expected: "Unary", got: token });
                }
            };
            let right: Box<Expression> = self.unary()?;
            let expr: Box<Expression> = Box::new(Expression::Unary {
                operator: op,
                right: right
            });
            return Ok(expr);
        }

        return Ok(self.literal()?);
    }
    fn literal(&mut self) -> Result<Box<Expression>, ParseError> {
        return match self.advance()? {
            Token::Bool(v) => Ok(Box::new(Expression::Literal(LiteralType::Bool(*v)))),
            Token::Int(v) => Ok(Box::new(Expression::Literal(LiteralType::Int(*v)))),
            Token::Float(v) => Ok(Box::new(Expression::Literal(LiteralType::Float(*v)))),
            Token::String(v) => Ok(Box::new(Expression::Literal(LiteralType::String(v.clone())))),
            Token::Null => Ok(Box::new(Expression::Literal(LiteralType::Null))),
            other => Err(ParseError::UnexpectedToken { expected: "Literal", got: other.clone() })
        };
    }
}