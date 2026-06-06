use super::lexer::{Token, LexError};
use std::convert::TryFrom;

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
    Null,
}

enum UnaryOp {
    LNot,
    Negate,
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
    fn expression(&self) -> Box<Expression> {
        return self.equality();
    }
    fn equality(&self) -> Box<Expression> {
        let mut expr: Box<Expression> = self.inequality();

        while (self.match_advance(&Token::Equal) || self.match_advance(&Token::NotEqual)) {
            let op: BinaryOp = match BinaryOp::try_from(self.previous()) {
                Ok(op) => op,
                Err(token) => {
                    println!("Invalid token {:?} for bianry operator", token);
                    BinaryOp::Null
                }
            }
            let right: Box<Expression> = self.inequality();
            *expr = Expression::Binary {
                left: expr,
                operator: op,
                right: right
            }
        }

        return expr;
    }
    fn inequality(&self) -> Box<Expression> {
        
    }
    fn term(&self) -> Box<Expression> {
        
    }
    fn factor(&self) -> Box<Expression> {
        
    }
    fn unary(&self) -> Box<Expression> {
        
    }
    fn literal(&self) -> Box<Expression> {
        
    }
}