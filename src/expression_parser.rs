use std::iter::{Iterator, Peekable};
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
enum TokenType {
    Equals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    Bang, // The `!` character
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
            pos: from - 1,
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
    cursor: usize,
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
                    Ok(Token::from(
                        self,
                        TokenType::LessThanEquals,
                        self.cursor - 1,
                        self.cursor,
                    ))
                } else {
                    Ok(make_char(self, TokenType::LessThan))
                }
            }

            '>' => {
                if self.peek() == '=' {
                    self.next_char();
                    Ok(Token::from(
                        self,
                        TokenType::GreaterThanEquals,
                        self.cursor - 1,
                        self.cursor,
                    ))
                } else {
                    Ok(make_char(self, TokenType::GreaterThan))
                }
            }

            '!' => {
                if self.peek() == '=' {
                    self.next_char();
                    Ok(Token::from(
                        self,
                        TokenType::NotEquals,
                        self.cursor - 1,
                        self.cursor,
                    ))
                } else {
                    Ok(make_char(self, TokenType::Bang))
                }
            }

            '&' => {
                if self.next_char() == '&' {
                    Ok(Token::from(
                        self,
                        TokenType::And,
                        self.cursor - 1,
                        self.cursor,
                    ))
                } else {
                    Err(LexError::InvalidAnd)
                }
            }

            '|' => {
                if self.next_char() == '|' {
                    Ok(Token::from(
                        self,
                        TokenType::Or,
                        self.cursor - 1,
                        self.cursor,
                    ))
                } else {
                    Err(LexError::InvalidOr)
                }
            }

            '=' => {
                if self.next_char() == '=' {
                    Ok(Token::from(
                        self,
                        TokenType::Equals,
                        self.cursor - 1,
                        self.cursor,
                    ))
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

                Ok(Token::from(
                    self,
                    TokenType::Identifier,
                    start + 1,
                    self.cursor,
                ))
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

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedToken { pos: usize, token: String },
    MismatchedParen { pos: usize },
    InvalidNumber { pos: usize, text: String },
}

#[derive(Debug, PartialEq)]
pub enum ConstraintOperator {
    Equals {
        left: Box<ConstraintExpression>,
        right: Box<ConstraintExpression>,
    },
    GreaterThan {
        left: Box<ConstraintExpression>,
        right: Box<ConstraintExpression>,
    },
    GreaterEqualThan {
        left: Box<ConstraintExpression>,
        right: Box<ConstraintExpression>,
    },
    LesserThan {
        left: Box<ConstraintExpression>,
        right: Box<ConstraintExpression>,
    },
    LesserEqualThan {
        left: Box<ConstraintExpression>,
        right: Box<ConstraintExpression>,
    },
    And {
        left: Box<ConstraintExpression>,
        right: Box<ConstraintExpression>,
    },
    Or {
        left: Box<ConstraintExpression>,
        right: Box<ConstraintExpression>,
    },
    Xor {
        left: Box<ConstraintExpression>,
        right: Box<ConstraintExpression>,
    },
    Not(Box<ConstraintExpression>),
    MemberAccess {
        object: Box<ConstraintExpression>,
        field: String,
    },
}

#[derive(Debug, PartialEq)]
pub enum ConstraintExpression {
    Variable(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Operator(Box<ConstraintOperator>),
}

/// Returns a numeric precedence
///
/// larger = tighter binding.
fn precedence(tt: &TokenType) -> Option<u8> {
    match tt {
        TokenType::Or => Some(1),
        TokenType::And => Some(2),
        TokenType::Xor => Some(3),
        TokenType::Equals
        | TokenType::NotEquals
        | TokenType::LessThan
        | TokenType::LessThanEquals
        | TokenType::GreaterThan
        | TokenType::GreaterThanEquals => Some(4),
        _ => None,
    }
}

pub fn parse_expression(input: &str) -> Result<ConstraintExpression, ParseError> {
    let mut lexer = Lexer::new(input).peekable();
    let expr = parse_expr(&mut lexer, 0)?;
    // Ensure we consumed the whole input (except trailing whitespace)
    match lexer.peek() {
        Some(Ok(tok)) if tok.ttype == TokenType::CloseParen => {
            Err(ParseError::MismatchedParen { pos: tok.pos })
        }
        Some(Ok(tok)) => Err(ParseError::UnexpectedToken {
            pos: tok.pos,
            token: tok.sequence.to_string(),
        }),
        Some(Err(_)) => Err(ParseError::UnexpectedToken {
            pos: 0,
            token: "lexical error".into(),
        }),
        None => Ok(expr),
    }
}

fn parse_expr<'a>(
    lexer: &mut Peekable<Lexer<'a>>,
    min_prec: u8,
) -> Result<ConstraintExpression, ParseError> {
    let mut left = parse_primary(lexer)?;

    loop {
        let op_tok = match lexer.peek() {
            Some(Ok(tok)) => tok,
            Some(Err(_)) => {
                return Err(ParseError::UnexpectedToken {
                    pos: lexer.last().unwrap().unwrap().pos,
                    token: "lexical error".into(),
                });
            }
            None => break, // EOF
        };

        let op_prec = match precedence(&op_tok.ttype) {
            Some(p) => p,
            None => break, // not a binary operator
        };

        if op_prec < min_prec {
            break;
        }

        let op = lexer.next().unwrap().unwrap();

        let right = parse_expr(lexer, op_prec + 1)?;

        left = ConstraintExpression::Operator(Box::new(match op.ttype {
            TokenType::Equals => ConstraintOperator::Equals {
                left: Box::new(left),
                right: Box::new(right),
            },
            TokenType::NotEquals => ConstraintOperator::Equals {
                left: Box::new(ConstraintExpression::Operator(Box::new(
                    ConstraintOperator::Not(Box::new(ConstraintExpression::Operator(Box::new(
                        ConstraintOperator::Equals {
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                    )))),
                ))),
                right: Box::new(ConstraintExpression::StringLiteral("".into())),
            },
            TokenType::LessThan => ConstraintOperator::LesserThan {
                left: Box::new(left),
                right: Box::new(right),
            },
            TokenType::LessThanEquals => ConstraintOperator::LesserEqualThan {
                left: Box::new(left),
                right: Box::new(right),
            },
            TokenType::GreaterThan => ConstraintOperator::GreaterThan {
                left: Box::new(left),
                right: Box::new(right),
            },
            TokenType::GreaterThanEquals => ConstraintOperator::GreaterEqualThan {
                left: Box::new(left),
                right: Box::new(right),
            },
            TokenType::And => ConstraintOperator::And {
                left: Box::new(left),
                right: Box::new(right),
            },
            TokenType::Or => ConstraintOperator::Or {
                left: Box::new(left),
                right: Box::new(right),
            },
            TokenType::Xor => ConstraintOperator::Xor {
                left: Box::new(left),
                right: Box::new(right),
            },
            _ => unreachable!(),
        }));
    }

    Ok(left)
}

/// Parse a primary expression: literals, identifiers, or a parenthesised sub‑expr.
fn parse_primary<'a>(lexer: &mut Peekable<Lexer<'a>>) -> Result<ConstraintExpression, ParseError> {
    let token_res = lexer
        .next()
        .ok_or(ParseError::UnexpectedEof)?
        .map_err(|_| ParseError::UnexpectedToken {
            pos: 0,
            token: "lexical error".into(),
        })?;

    match token_res.ttype {
        TokenType::Number => {
            // Try integer first, then float.
            if token_res.sequence.contains('.') {
                token_res
                    .sequence
                    .parse::<f64>()
                    .map(ConstraintExpression::FloatLiteral)
                    .map_err(|_| ParseError::InvalidNumber {
                        pos: token_res.pos,
                        text: token_res.sequence.to_string(),
                    })
            } else {
                token_res
                    .sequence
                    .parse::<i64>()
                    .map(ConstraintExpression::IntegerLiteral)
                    .map_err(|_| ParseError::InvalidNumber {
                        pos: token_res.pos,
                        text: token_res.sequence.to_string(),
                    })
            }
        }

        TokenType::String => Ok(ConstraintExpression::StringLiteral(token_res.as_str())),

        TokenType::Identifier => parse_identifier_chain(lexer, token_res),

        TokenType::OpenParen => {
            let expr = parse_expr(lexer, 0)?;

            // Matching ")"
            match lexer.next() {
                Some(Ok(tok)) if tok.ttype == TokenType::CloseParen => Ok(expr),
                Some(Ok(tok)) => Err(ParseError::MismatchedParen { pos: tok.pos }),
                Some(Err(_)) => Err(ParseError::UnexpectedToken {
                    pos: token_res.pos,
                    token: "lexical error".into(),
                }),
                None => Err(ParseError::UnexpectedEof),
            }
        }

        other => Err(ParseError::UnexpectedToken {
            pos: token_res.pos,
            token: format!("{:?}", other),
        }),
    }
}

fn parse_identifier_chain<'a>(
    lexer: &mut Peekable<Lexer<'a>>,
    first: Token<'a>,
) -> Result<ConstraintExpression, ParseError> {
    let mut expr = ConstraintExpression::Variable(first.sequence.to_string());

    loop {
        match lexer.peek() {
            Some(Ok(tok)) if tok.ttype == TokenType::Dot => {
                lexer.next();

                let next_tok = lexer
                    .next()
                    .ok_or(ParseError::UnexpectedEof)?
                    .map_err(|_| ParseError::UnexpectedToken {
                        pos: 0,
                        token: "lexical error".into(),
                    })?;

                if next_tok.ttype != TokenType::Identifier {
                    return Err(ParseError::UnexpectedToken {
                        pos: next_tok.pos,
                        token: next_tok.sequence.to_string(),
                    });
                }

                expr = ConstraintExpression::Operator(Box::new(ConstraintOperator::MemberAccess {
                    object: Box::new(expr),
                    field: next_tok.sequence.to_string(),
                }));
            }
            _ => break, // no more dots
        }
    }

    Ok(expr)
}
