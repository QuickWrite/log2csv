use std::iter::{Peekable, Iterator};
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
enum TokenType {
    Equals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    Bang,              // The `!` character
    And,
    Or,
    Xor,
    Dot,
    OpenParen,
    CloseParen,
    Number,
    String,
    Identifier,
}

#[derive(Debug)]
struct Token<'a> {
    ttype: TokenType,
    sequence: &'a str,
    pos: usize,
}

impl<'a> Token<'a> {
    fn from(lexer: &Lexer<'a>, ttype: TokenType, from: usize, to: usize) -> Token<'a> {
        Token {
            ttype,
            sequence: &lexer.expression[(from - 1)..to],
            pos: from - 1
        }
    }

    fn from_char(lexer: &Lexer<'a>, ttype: TokenType, pos: usize) -> Token<'a> {
        Token::from(lexer, ttype, pos, pos)
    }

    fn as_str(&self) -> String {
        match self.ttype {
            TokenType::String => {
                let inner = &self.sequence[1..self.sequence.len() - 1];
                let mut out = String::with_capacity(inner.len());
                let mut chars = inner.chars();
                while let Some(ch) = chars.next() {
                    if ch == '\\' {
                        match chars.next() {
                            Some('\"') => out.push('\"'),
                            Some('\\') => out.push('\\'),

                            Some(other) => {
                                out.push('\\');
                                out.push(other);
                            }
                            None => {
                                out.push('\\');
                            }
                        }
                    } else {
                        out.push(ch);
                    }
                }
                out
            }
            _ => self.sequence.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LexError {
    UnterminatedString { pos: usize },
    UnexpectedChar { pos: usize, ch: char },
    InvalidAnd,
    InvalidOr,
    InvalidEquals,
}

struct Lexer<'a> {
    expression: &'a str,
    chars: Peekable<Chars<'a>>,
    cursor: usize
}

impl Lexer<'_> {
    fn new(input: &str) -> Lexer<'_> {
        Lexer {
            expression: input,
            chars: input.chars().peekable(),
            cursor: 0,
        }
    }

    fn next_char(&mut self) -> char {
        let char = self.chars.next();

        if char.is_none() {
            return '\0';
        }

        self.cursor += 1;

        char.unwrap()
    }

    fn peek(&mut self) -> char {
        *self.chars.peek().unwrap_or(&'\0')
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_ascii_whitespace() {
            self.next_char();
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace(); // Ignore Whitespace

        let first = self.next_char();
        if first == '\0' {
            return None;
        }

        // Helper that creates a token from the current cursor position.
        let make_char = |lexer, tt| Token::from_char(lexer, tt, lexer.cursor);

        let token = match first {
            '(' => Ok(make_char(self, TokenType::OpenParen)),
            ')' => Ok(make_char(self, TokenType::CloseParen)),
            '^' => Ok(make_char(self, TokenType::Xor)),
            '.' => Ok(make_char(self, TokenType::Dot)),

            '<' => {
                if self.peek() == '=' {
                    self.next_char(); // consume '='
                    Ok(Token::from(self, TokenType::LessThanEquals, self.cursor - 1, self.cursor))
                } else {
                    Ok(make_char(self, TokenType::LessThan))
                }
            }

            '>' => {
                if self.peek() == '=' {
                    self.next_char();
                    Ok(Token::from(self, TokenType::GreaterThanEquals, self.cursor - 1, self.cursor))
                } else {
                    Ok(make_char(self, TokenType::GreaterThan))
                }
            }

            '!' => {
                if self.peek() == '=' {
                    self.next_char();
                    Ok(Token::from(self, TokenType::NotEquals, self.cursor - 1, self.cursor))
                } else {
                    Ok(make_char(self, TokenType::Bang))
                }
            }

            '&' => {
                if self.next_char() == '&' {
                    Ok(Token::from(self, TokenType::And, self.cursor - 1, self.cursor))
                } else {
                    Err(LexError::InvalidAnd)
                }
            }

            '|' => {
                if self.next_char() == '|' {
                    Ok(Token::from(self, TokenType::Or, self.cursor - 1, self.cursor))
                } else {
                    Err(LexError::InvalidOr)
                }
            }

            '=' => {
                if self.next_char() == '=' {
                    Ok(Token::from(self, TokenType::Equals, self.cursor - 1, self.cursor))
                } else {
                    Err(LexError::InvalidEquals)
                }
            }

            '"' => {
                let start = self.cursor;

                loop {
                    let ch = self.next_char();
                    match ch {
                        '\\' => {
                            let _ = self.next_char();
                        }
                        '"' => {
                            break;
                        }
                        '\0' => {
                            return Some(Err(LexError::UnterminatedString { pos: start - 1 }));
                        }
                        _ => {}
                    }
                }

                Ok(Token::from(self, TokenType::String, start, self.cursor))
            }

            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = self.cursor - 1;

                while {
                    let nxt = self.peek();
                    nxt.is_ascii_alphanumeric() || nxt == '_'
                } {
                    self.next_char();
                }

                Ok(Token::from(self, TokenType::Identifier, start + 1, self.cursor))
            }

            c if c.is_ascii_digit() || (c == '.' && self.peek().is_ascii_digit()) => {
                let start = self.cursor - 1;
                let mut seen_dot = c == '.';

                while {
                    let nxt = self.peek();
                    if nxt.is_ascii_digit() {
                        true
                    } else if nxt == '.' && !seen_dot {
                        // First dot – part of a floating point literal.
                        seen_dot = true;
                        true
                    } else {
                        false
                    }
                } {
                    self.next_char();
                }

                Ok(Token::from(self, TokenType::Number, start + 1, self.cursor))
            }

            // -----------------------------------------------------------------
            // Anything else is an error (including stray characters that are not
            // whitespace – they have already been filtered out above).
            // -----------------------------------------------------------------
            other => Err(LexError::UnexpectedChar {
                pos: self.cursor - 1,
                ch: other,
            }),
        };

        Some(token)
    }
}

// Temporary implementation for debug purposes.
pub fn parse_expression(input: &str) {
    let lexer = Lexer::new(input);
    for result in lexer {
        match result {
            Ok(tok) => {
                // For strings we show the *unescaped* value, for everything else the raw slice.
                let display = match tok.ttype {
                    TokenType::String => format!("\"{}\"", tok.as_str()),
                    _ => tok.sequence.to_string(),
                };
                println!("[{}] {:?} ({})", tok.pos, tok.ttype, display);
            }
            Err(err) => {
                eprintln!("Lexical error: {:?}", err);
                break;
            }
        }
    }
}
