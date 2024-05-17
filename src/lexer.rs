// #[derive(A, B, Serialize, ..., Deserialize, ...)]
// struct User {

//     #[serde(rename = "name")]
//     name: String,

//     #[serde(thing, another_thing, ...)]
//     age: u32,
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// enum Stuff {
//     #[serde(rename = "name")]
//     Name(String),

//     #[serde(rename = "age")]
//     Age(u32),
// }

use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum TokenType<'a> {
    Hash,
    LBracket,
    RBracket,
    Comma,
    Identifier(&'a str),
    Colon,
    String(&'a str),
    Equals,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Struct,
    Enum,
    Pub,
    Langle,
    Rangle,
}

#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct LexicalToken<'a> {
    pub token: TokenType<'a>,
    pub span: Span,
}

pub struct Lexer<'a> {
    input: &'a str,
    tokens: Vec<LexicalToken<'a>>,
    cursor: usize,
    start: usize,

    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            tokens: Vec::new(),
            cursor: 0,
            start: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn lex(mut self) -> Vec<LexicalToken<'a>> {
        use TokenType::*;

        let input = self.input.as_bytes();

        while self.cursor < input.len() {
            let column = self.column;
            let c = input[self.cursor];

            self.start = self.cursor;
            self.advance();

            match c {
                b',' => self.push_token(Comma),
                b':' => self.push_token(Colon),
                b'{' => self.push_token(LBrace),
                b'}' => self.push_token(RBrace),
                b'[' => self.push_token(LBracket),
                b']' => self.push_token(RBracket),
                b'#' => self.push_token(Hash),
                b'=' => self.push_token(Equals),
                b'(' => self.push_token(LParen),
                b')' => self.push_token(RParen),
                b' ' | b'\r' | b'\t' => {}
                b'\n' => {
                    self.line += 1;
                    self.column = 1;
                }
                b'_' | b'a'..=b'z' | b'A'..=b'Z' => self.lex_identifier(),
                b'"' => {
                    self.advance();
                    self.start += 1;
                    self.lex_string()
                }
                b'/' if self.cursor < input.len() && input[self.cursor] == b'/' => {
                    self.advance();

                    while self.cursor < input.len() {
                        if input[self.cursor] == b'\n' {
                            break;
                        }

                        self.advance();
                    }
                }
                b'/' if self.cursor < input.len() && input[self.cursor] == b'*' => {
                    self.advance();

                    while self.cursor < input.len() {
                        if input[self.cursor] == b'*'
                            && self.cursor + 1 < input.len()
                            && input[self.cursor + 1] == b'/'
                        {
                            self.advance();
                            self.advance();

                            break;
                        }

                        if input[self.cursor] == b'\n' {
                            self.line += 1;
                            self.column = 1;
                        }

                        self.advance();
                    }
                }
                b'<' => self.push_token(Langle),
                b'>' => self.push_token(Rangle),
                _ => panic!(
                    "Unexpected character {} at {}:{}",
                    c as char, self.line, column
                ),
            }
        }

        self.cursor = 0;
        self.start = 0;

        self.tokens
    }

    fn advance(&mut self) {
        self.cursor += 1;
        self.column += 1;
    }

    fn push_token(&mut self, token: TokenType<'a>) {
        self.tokens.push(LexicalToken {
            token,
            span: Span {
                start: self.start,
                end: self.cursor,
            },
        });
    }

    fn lexeme(&self) -> &'a str {
        &self.input[self.start..self.cursor]
    }

    fn lex_string(&mut self) {
        let input = self.input.as_bytes();

        loop {
            let c = input[self.cursor];

            if c == b'"' {
                break;
            }

            if self.cursor >= input.len() {
                panic!("Unterminated string");
            }

            self.advance();
        }

        let lexeme = self.lexeme();

        // Skip the closing quote
        self.advance();

        self.push_token(TokenType::String(lexeme));
    }

    fn lex_identifier(&mut self) {
        let input = self.input.as_bytes();

        while self.cursor < input.len() {
            let c = input[self.cursor];

            if !(c as char).is_alphanumeric() && c != b'_' {
                break;
            }

            self.advance();
        }

        let lexeme = self.lexeme();

        if lexeme == "struct" {
            self.push_token(TokenType::Struct);
        } else if lexeme == "enum" {
            self.push_token(TokenType::Enum);
        } else if lexeme == "pub" {
            self.push_token(TokenType::Pub);
        } else {
            self.push_token(TokenType::Identifier(lexeme));
        }
    }
}

impl<'a> Display for TokenType<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use TokenType::*;

        match self {
            Hash => write!(f, "#"),
            LBracket => write!(f, "["),
            RBracket => write!(f, "]"),
            Comma => write!(f, ","),
            Identifier(ident) => write!(f, "{}", ident),
            Colon => write!(f, ":"),
            String(s) => write!(f, "\"{}\"", s),
            Equals => write!(f, "="),
            LBrace => write!(f, "{{"),
            RBrace => write!(f, "}}"),
            LParen => write!(f, "("),
            RParen => write!(f, ")"),
            Struct => write!(f, "struct"),
            Enum => write!(f, "enum"),
            Pub => write!(f, "pub"),
            Langle => write!(f, "<"),
            Rangle => write!(f, ">"),
        }
    }
}

impl<'a> Display for LexicalToken<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}
