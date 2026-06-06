use std::str::Chars;

#[derive(Debug, Clone)]
pub enum Token {
    LeftBracket, RightBracket, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Slash, Asterix, Colon, Semicolon,

    Identifier(String), String(String), Int(i64), Float(f64),

    Assign, Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    LAnd, LOr, LNot, 

    If, Else, For, Class,

    EOF
}

#[derive(Debug)]
pub struct LexError {
    message: String,
    line: usize,
    character: usize,
}

pub struct Scanner<'a> {
    source: Chars<'a>,
    line: usize,
    character: usize,
}

impl<'a> Default for Scanner<'a> {
    fn default() -> Self {
        return Self {
            source: "".chars(),
            line: 0,
            character: 0,
        };
    }
}

impl<'a> Scanner<'a> {
    pub fn from_source(source: Chars<'a>) -> Self {
        return Self {
            source: source,
            ..Default::default()
        };
    }
    pub fn new() -> Self {
        return Default::default();
    }
    fn advance(&mut self) -> Option<char> {
        match self.peek() {
            Some('\n') => {
                self.line += 1;
                self.character = 0;
            },
            Some(_) => { self.character += 1; },
            None => (),
        }
        return self.source.next();
    }
    fn peek(&self) -> Option<char> {
        return self.source.clone().next();
    }
    fn match_advance(&mut self, c: char) -> bool {
        if Some(c) == self.peek() {
            self.advance();
            return true;
        }
        else {
            return false;
        }
    }
    fn read_next_token(&mut self) -> Result<Token, LexError> {
        return match self.advance() {
            Some('{') => Ok(Token::LeftBrace),
            Some('}') => Ok(Token::RightBrace),
            Some('(') => Ok(Token::LeftBracket),
            Some(')') => Ok(Token::RightBracket),
            Some(',') => Ok(Token::Comma),
            Some('.') => Ok(Token::Dot),
            Some('-') => Ok(Token::Minus),
            Some('+') => Ok(Token::Plus),
            Some('/') => Ok(Token::Slash),
            Some('*') => Ok(Token::Asterix),
            Some(':') => Ok(Token::Colon),
            Some(';') => Ok(Token::Semicolon),
            Some('=') => {
                if self.match_advance('=') { Ok(Token::Equal) }
                else { Ok(Token::Assign) }
            },
            Some('!') => {
                if self.match_advance('=') { Ok(Token::NotEqual) }
                else { Ok(Token::LNot) }
            },
            Some('<') => {
                if self.match_advance('=') { Ok(Token::LessEqual) }
                else { Ok(Token::Less) }
            },
            Some('>') => {
                if self.match_advance('=') { Ok(Token::GreaterEqual) }
                else { Ok(Token::Greater) }
            },
            Some('&') => {
                if self.match_advance('&') { Ok(Token::LAnd) }
                else {
                    Err(LexError {
                        message: "Expected '&&', single '&' is not valid.".to_string(),
                        line: self.line,
                        character: self.character,
                    })
                }
            },
            Some('|') => {
                if self.match_advance('|') { Ok(Token::LOr) }
                else {
                    Err(LexError {
                        message: "Expected '||', single '|' is not valid.".to_string(),
                        line: self.line,
                        character: self.character,
                    })
                }
            },
            Some('"') => {
                let mut s: String = String::new();
                loop {
                    match self.advance() {
                        Some('"') => break,
                        Some(c) => s.push(c),
                        None => return Err(LexError {
                            message: "Unfinished string literal".to_string(),
                            line: self.line,
                            character: self.character,
                        }),
                    }
                }
                Ok(Token::String(s))
            },
            Some(c) if c.is_ascii_digit() => {
                let line: usize = self.line;
                let character: usize = self.character;
                
                let mut s: String = c.to_string();
                let mut is_float: bool = false;
                
                loop {
                    match self.peek() {
                        Some(d) if d.is_ascii_digit() => { self.advance(); s.push(d); },
                        Some('.') if !is_float => { self.advance(); s.push('.'); is_float = true; },
                        _ => break,
                    }
                }

                if is_float {
                    match s.parse::<f64>() {
                        Ok(n) => Ok(Token::Float(n)),
                        Err(e) => Err(LexError { message: e.to_string(), line: line, character: character }),
                    }
                }
                else {
                    match s.parse::<i64>() {
                        Ok(n) => Ok(Token::Int(n)),
                        Err(e) => Err(LexError { message: e.to_string(), line: line, character: character }),
                    }
                }
            },
            Some(c) if c.is_alphabetic() => {
                let mut s: String = c.to_string();

                loop {
                    match self.peek() {
                        Some(a) if a.is_alphabetic() => { s.push(a); self.advance(); },
                        _ => break,
                    }
                }

                Ok(match s.as_str() {
                    "if" => Token::If,
                    "else" => Token::Else,
                    "for" => Token::For,
                    "class" => Token::Class,
                    _ => Token::Identifier(s),
                })
            },
            Some(c) if c == ' ' || c == '\t' || c == '\n' || c == '\r' => self.read_next_token(),
            Some(invalid) => Err(LexError { 
                message: format!("Invalid character {} found.", invalid),
                line: self.line,
                character: self.character 
            }),
            None => Ok(Token::EOF),
        };
    }
    pub fn read_all_tokens(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let token = self.read_next_token();

            match token {
                Ok(t) if matches!(t, Token::EOF) => return Ok(tokens),
                Ok(t) => tokens.push(t),
                Err(e) => return Err(e),
            }
        }
    }
}