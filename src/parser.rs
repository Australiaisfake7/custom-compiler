use super::lexer::{Token, DataType};
use std::convert::TryFrom;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParseError {
    UnexpectedToken { _expected: &'static str, _got: Token },
    UnexpectedReadIndex(usize),
    UnexpectedAssignmentTarget,
}
#[derive(Debug)]
pub enum BinaryOp {
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
#[derive(Debug)]
pub enum UnaryOp {
    LNot,
    Negate,
}
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LiteralType {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOp,
        right: Box<Expression>,
    },
    Literal(LiteralType),
    Variable(String),
    Assignment {
        name: String,
        value: Box<Expression>,
    },
}

pub enum Statement {
    Expression(Box<Expression>),
    Declaration {name: String, value: Box<Expression>, data_type: DataType},
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

pub fn parse_tokens(tokens: Vec<Token>) -> Result<Statement, ParseError> {
    let mut parser: Parser = Parser::from_tokens(tokens);

    return Ok(parser.statement()?);
}

struct Parser {
    tokens: Vec<Token>,
    position: usize
}

impl Default for Parser {
    fn default() -> Self {
        return Self {
            tokens: Vec::new(),
            position: 0
        };
    }
}

impl Parser {
    fn new() -> Self {
        return Default::default();
    }
    fn from_tokens(tokens: Vec<Token>) -> Self {
        let mut p: Self = Self::new();
        p.tokens = tokens;
        return p;
    }
    fn peek(&self) -> Result<&Token, ParseError> {
        self.tokens.get(self.position).ok_or(ParseError::UnexpectedReadIndex(self.position))
    }
    fn previous(&self) -> Result<&Token, ParseError> {
        if self.position > 0 {
            return self.tokens.get(self.position - 1).ok_or(ParseError::UnexpectedReadIndex(self.position - 1));
        }
        return Err(ParseError::UnexpectedReadIndex(usize::MAX));
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
                let _ = self.advance();
                return true;
            }
        }
        return false;
    }
    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_advance(&[Token::Let]) {
            return self.declaration();
        }

        let expr: Box<Expression> = self.expression()?;
        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { _expected: "Semicolon", _got: self.peek()?.clone() })
        }

        return Ok(Statement::Expression(expr));
    }
    fn declaration(&mut self) -> Result<Statement, ParseError> {
        let t: DataType = match self.advance()? {
            Token::DataType(d) => d.clone(),
            token=> return Err(ParseError::UnexpectedToken { _expected: "Data Type", _got: token.clone() })
            
        };
        let n: String = match self.advance()? {
            Token::Identifier(i) => i.clone(),
            token => return Err(ParseError::UnexpectedToken { _expected: "Identifier", _got: token.clone() })
        };
        let v: Box<Expression> = match self.advance()? {
            Token::Semicolon => return Ok(Statement::Declaration { name: n, value: Box::new(Expression::Literal(LiteralType::Null)), data_type: t }),
            Token::Assign => self.expression()?,
            token => return Err(ParseError::UnexpectedToken { _expected: "Semicolon", _got: token.clone() })
        };

        if !self.match_advance(&[Token::Semicolon]) {
            return Err(ParseError::UnexpectedToken { _expected: "Semicolon", _got: self.peek()?.clone() });
        }

        return Ok(Statement::Declaration { name: n, value: v, data_type: t });
    }
    fn expression(&mut self) -> Result<Box<Expression>, ParseError> {
        return self.assignment();
    }
    fn assignment(&mut self) -> Result<Box<Expression>, ParseError> {
        let expr: Box<Expression> = self.equality()?;

        if self.match_advance(&[Token::Assign]) {
            let value: Box<Expression> = self.assignment()?;

            if let Expression::Variable(name) = *expr {
                return Ok(Box::new(Expression::Assignment { name: name, value: value }));
            }

            return Err(ParseError::UnexpectedAssignmentTarget);
        }

        return Ok(expr);
    }
    fn equality(&mut self) -> Result<Box<Expression>, ParseError> {
        let mut expr: Box<Expression> = self.inequality()?;

        while self.match_advance(&[Token::Equal, Token::NotEqual]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { _expected: "Equality", _got: token });
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

        while self.match_advance(&[Token::Less, Token::LessEqual, Token::Greater, Token::GreaterEqual]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { _expected: "Inequality", _got: token });
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

        while self.match_advance(&[Token::Plus, Token::Minus]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { _expected: "Term", _got: token });
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

        while self.match_advance(&[Token::Asterix, Token::Slash]) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()?) {
                Ok(op) => op,
                Err(token) => {
                    return Err(ParseError::UnexpectedToken { _expected: "Factor", _got: token });
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
                    return Err(ParseError::UnexpectedToken { _expected: "Unary", _got: token });
                }
            };
            let right: Box<Expression> = self.unary()?;
            let expr: Box<Expression> = Box::new(Expression::Unary {
                operator: op,
                right: right
            });
            return Ok(expr);
        }

        return Ok(self.primary()?);
    }
    fn primary(&mut self) -> Result<Box<Expression>, ParseError> {
        return match self.advance()? {
            Token::Bool(v) => Ok(Box::new(Expression::Literal(LiteralType::Bool(*v)))),
            Token::Int(v) => Ok(Box::new(Expression::Literal(LiteralType::Int(*v)))),
            Token::Float(v) => Ok(Box::new(Expression::Literal(LiteralType::Float(*v)))),
            Token::String(v) => Ok(Box::new(Expression::Literal(LiteralType::String(v.clone())))),
            Token::Null => Ok(Box::new(Expression::Literal(LiteralType::Null))),
            Token::Identifier(name) => Ok(Box::new(Expression::Variable(name.clone()))),
            Token::LeftBracket => {
                let expr: Box<Expression> = self.expression()?;
                match self.advance() {
                    Ok(t) if t == &Token::RightBracket => return Ok(expr),
                    Ok(t) => return Err(ParseError::UnexpectedToken { _expected: ")", _got: t.clone() }),
                    Err(e) => Err(e),
                }
            },
            other => Err(ParseError::UnexpectedToken { _expected: "Primary", _got: other.clone() })
        };
    }
}