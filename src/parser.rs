use super::lexer::{Token, LexError};

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
}

enum Expression {
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>
    },
    Unary {
        left: Box<Expression>,
        operator: UnaryOp
    },
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
        let expr: Box<Expression> = self.inequality();
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