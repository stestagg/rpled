use crate::ast::Spanned;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(i64),
    Float(f64),
    String(String),

    // Keywords
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    If,
    In,
    Local,
    Nil,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,

    // Identifiers
    Ident(String),

    // Operators and Delimiters
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Caret,          // ^
    Eq,             // ==
    Ne,             // ~=
    Lt,             // <
    Le,             // <=
    Gt,             // >
    Ge,             // >=
    Assign,         // =
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Comma,          // ,
    Semicolon,      // ;
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Float(fl) => write!(f, "{}", fl),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::And => write!(f, "and"),
            Token::Break => write!(f, "break"),
            Token::Do => write!(f, "do"),
            Token::Else => write!(f, "else"),
            Token::Elseif => write!(f, "elseif"),
            Token::End => write!(f, "end"),
            Token::False => write!(f, "false"),
            Token::For => write!(f, "for"),
            Token::Function => write!(f, "function"),
            Token::If => write!(f, "if"),
            Token::In => write!(f, "in"),
            Token::Local => write!(f, "local"),
            Token::Nil => write!(f, "nil"),
            Token::Not => write!(f, "not"),
            Token::Or => write!(f, "or"),
            Token::Repeat => write!(f, "repeat"),
            Token::Return => write!(f, "return"),
            Token::Then => write!(f, "then"),
            Token::True => write!(f, "true"),
            Token::Until => write!(f, "until"),
            Token::While => write!(f, "while"),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Caret => write!(f, "^"),
            Token::Eq => write!(f, "=="),
            Token::Ne => write!(f, "~="),
            Token::Lt => write!(f, "<"),
            Token::Le => write!(f, "<="),
            Token::Gt => write!(f, ">"),
            Token::Ge => write!(f, ">="),
            Token::Assign => write!(f, "="),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedChar { ch: char, pos: usize },
    UnterminatedString { pos: usize },
    InvalidEscape { ch: char, pos: usize },
    InvalidNumber { text: String, pos: usize },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedChar { ch, pos } => {
                write!(f, "Unexpected character '{}' at position {}", ch, pos)
            }
            LexError::UnterminatedString { pos } => {
                write!(f, "Unterminated string at position {}", pos)
            }
            LexError::InvalidEscape { ch, pos } => {
                write!(f, "Invalid escape sequence '\\{}' at position {}", ch, pos)
            }
            LexError::InvalidNumber { text, pos } => {
                write!(f, "Invalid number '{}' at position {}", text, pos)
            }
        }
    }
}

impl std::error::Error for LexError {}

pub fn lex(source: &str) -> Result<Vec<Spanned<Token>>, LexError> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some((pos, ch)) = chars.next() {
        match ch {
            // Whitespace
            ' ' | '\t' | '\r' | '\n' => continue,

            // Comments
            '-' if chars.peek().map(|(_, c)| *c) == Some('-') => {
                // Skip until end of line
                chars.next(); // consume second '-'
                while let Some((_, ch)) = chars.peek() {
                    if *ch == '\n' {
                        break;
                    }
                    chars.next();
                }
            }

            // Operators and delimiters
            '+' => tokens.push(Spanned::new(Token::Plus, pos..pos + 1)),
            '-' => tokens.push(Spanned::new(Token::Minus, pos..pos + 1)),
            '*' => tokens.push(Spanned::new(Token::Star, pos..pos + 1)),
            '/' => tokens.push(Spanned::new(Token::Slash, pos..pos + 1)),
            '%' => tokens.push(Spanned::new(Token::Percent, pos..pos + 1)),
            '^' => tokens.push(Spanned::new(Token::Caret, pos..pos + 1)),
            '(' => tokens.push(Spanned::new(Token::LParen, pos..pos + 1)),
            ')' => tokens.push(Spanned::new(Token::RParen, pos..pos + 1)),
            '{' => tokens.push(Spanned::new(Token::LBrace, pos..pos + 1)),
            '}' => tokens.push(Spanned::new(Token::RBrace, pos..pos + 1)),
            '[' => tokens.push(Spanned::new(Token::LBracket, pos..pos + 1)),
            ']' => tokens.push(Spanned::new(Token::RBracket, pos..pos + 1)),
            ',' => tokens.push(Spanned::new(Token::Comma, pos..pos + 1)),
            ';' => tokens.push(Spanned::new(Token::Semicolon, pos..pos + 1)),

            // Two-character operators
            '=' => {
                if chars.peek().map(|(_, c)| *c) == Some('=') {
                    chars.next();
                    tokens.push(Spanned::new(Token::Eq, pos..pos + 2));
                } else {
                    tokens.push(Spanned::new(Token::Assign, pos..pos + 1));
                }
            }
            '~' => {
                if chars.peek().map(|(_, c)| *c) == Some('=') {
                    chars.next();
                    tokens.push(Spanned::new(Token::Ne, pos..pos + 2));
                } else {
                    return Err(LexError::UnexpectedChar { ch: '~', pos });
                }
            }
            '<' => {
                if chars.peek().map(|(_, c)| *c) == Some('=') {
                    chars.next();
                    tokens.push(Spanned::new(Token::Le, pos..pos + 2));
                } else {
                    tokens.push(Spanned::new(Token::Lt, pos..pos + 1));
                }
            }
            '>' => {
                if chars.peek().map(|(_, c)| *c) == Some('=') {
                    chars.next();
                    tokens.push(Spanned::new(Token::Ge, pos..pos + 2));
                } else {
                    tokens.push(Spanned::new(Token::Gt, pos..pos + 1));
                }
            }

            // String literals
            '"' | '\'' => {
                let quote = ch;
                let start_pos = pos;
                let mut string = String::new();

                loop {
                    match chars.next() {
                        Some((_, '\\')) => {
                            // Escape sequence
                            match chars.next() {
                                Some((_, 'n')) => string.push('\n'),
                                Some((_, 'r')) => string.push('\r'),
                                Some((_, 't')) => string.push('\t'),
                                Some((_, '0')) => string.push('\0'),
                                Some((_, '\\')) => string.push('\\'),
                                Some((_, '"')) => string.push('"'),
                                Some((_, '\'')) => string.push('\''),
                                Some((esc_pos, esc_ch)) => {
                                    return Err(LexError::InvalidEscape { ch: esc_ch, pos: esc_pos });
                                }
                                None => {
                                    return Err(LexError::UnterminatedString { pos: start_pos });
                                }
                            }
                        }
                        Some((_, c)) if c == quote => {
                            // End of string
                            let end_pos = chars.peek().map(|(p, _)| *p).unwrap_or(source.len());
                            tokens.push(Spanned::new(
                                Token::String(string),
                                start_pos..end_pos,
                            ));
                            break;
                        }
                        Some((_, '\n')) => {
                            return Err(LexError::UnterminatedString { pos: start_pos });
                        }
                        Some((_, c)) => {
                            string.push(c);
                        }
                        None => {
                            return Err(LexError::UnterminatedString { pos: start_pos });
                        }
                    }
                }
            }

            // Numbers
            '0'..='9' => {
                let start_pos = pos;
                let mut num_str = String::new();
                num_str.push(ch);

                // Collect digits
                while let Some((_, c @ '0'..='9')) = chars.peek() {
                    num_str.push(*c);
                    chars.next();
                }

                // Check for decimal point or exponent
                let mut is_float = false;

                if chars.peek().map(|(_, c)| *c) == Some('.') {
                    // Check if followed by digit (to distinguish from method calls)
                    let mut temp_chars = chars.clone();
                    temp_chars.next(); // skip '.'
                    if let Some((_, '0'..='9')) = temp_chars.peek() {
                        is_float = true;
                        num_str.push('.');
                        chars.next(); // consume '.'

                        while let Some((_, c @ '0'..='9')) = chars.peek() {
                            num_str.push(*c);
                            chars.next();
                        }
                    }
                }

                // Check for exponent
                if let Some((_, c @ ('e' | 'E'))) = chars.peek() {
                    is_float = true;
                    num_str.push(*c);
                    chars.next();

                    if let Some((_, c @ ('+' | '-'))) = chars.peek() {
                        num_str.push(*c);
                        chars.next();
                    }

                    if !matches!(chars.peek(), Some((_, '0'..='9'))) {
                        return Err(LexError::InvalidNumber {
                            text: num_str,
                            pos: start_pos,
                        });
                    }

                    while let Some((_, c @ '0'..='9')) = chars.peek() {
                        num_str.push(*c);
                        chars.next();
                    }
                }

                let end_pos = chars.peek().map(|(p, _)| *p).unwrap_or(source.len());

                if is_float {
                    let value = num_str.parse::<f64>().map_err(|_| LexError::InvalidNumber {
                        text: num_str.clone(),
                        pos: start_pos,
                    })?;
                    tokens.push(Spanned::new(Token::Float(value), start_pos..end_pos));
                } else {
                    let value = num_str.parse::<i64>().map_err(|_| LexError::InvalidNumber {
                        text: num_str.clone(),
                        pos: start_pos,
                    })?;
                    tokens.push(Spanned::new(Token::Number(value), start_pos..end_pos));
                }
            }

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let start_pos = pos;
                let mut ident = String::new();
                ident.push(ch);

                while let Some((_, c @ ('a'..='z' | 'A'..='Z' | '0'..='9' | '_'))) = chars.peek() {
                    ident.push(*c);
                    chars.next();
                }

                let end_pos = chars.peek().map(|(p, _)| *p).unwrap_or(source.len());

                let token = match ident.as_str() {
                    "and" => Token::And,
                    "break" => Token::Break,
                    "do" => Token::Do,
                    "else" => Token::Else,
                    "elseif" => Token::Elseif,
                    "end" => Token::End,
                    "false" => Token::False,
                    "for" => Token::For,
                    "function" => Token::Function,
                    "if" => Token::If,
                    "in" => Token::In,
                    "local" => Token::Local,
                    "nil" => Token::Nil,
                    "not" => Token::Not,
                    "or" => Token::Or,
                    "repeat" => Token::Repeat,
                    "return" => Token::Return,
                    "then" => Token::Then,
                    "true" => Token::True,
                    "until" => Token::Until,
                    "while" => Token::While,
                    _ => Token::Ident(ident),
                };

                tokens.push(Spanned::new(token, start_pos..end_pos));
            }

            _ => {
                return Err(LexError::UnexpectedChar { ch, pos });
            }
        }
    }

    Ok(tokens)
}
