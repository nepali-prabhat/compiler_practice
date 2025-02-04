#![allow(dead_code)]

pub(crate) mod cursor;
mod tests;

use cursor::Cursor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TokenKind {
    // ,
    COMMA,
    // :
    COLON,
    // ;
    SEMICOLON,
    // (
    LPAREN,
    // )
    RPAREN,
    // [
    LBRACK,
    // ]
    RBRACK,
    // {
    LCURLY,
    // }
    RCURLY,
    // .
    DOT,

    // :=
    ASSIGN,

    // algebric operators
    // +
    PLUS,
    // -
    MINUS,
    // *
    TIMES,
    // /
    DIVIDE,
    // %
    PERCENT,
    // logical operators
    // =
    EQ,
    // <>
    NEQ,
    // <
    LT,
    // <=
    LE,
    // >
    GT,
    // >=
    GE,
    // &
    AND,
    // |
    OR,

    // keywords
    // array
    ARRAY,
    // if
    IF,
    // then
    THEN,
    // else
    ELSE,
    // while
    WHILE,
    // for
    FOR,
    // to
    TO,
    // do
    DO,
    // let
    LET,
    // in
    IN,
    // end
    END,
    // of
    OF,
    // break
    BREAK,
    // function
    FUNCTION,
    // var
    VAR,
    // type
    TYPE,
    // nil
    NIL,

    ID,
    // Ids and data types
    STRING,
    INT,
    FLOAT,

    COMMENT,

    EOF,
    UNKNOWN,
    WHITESPACE,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Token {
    kind: TokenKind,
    pos: TokenPos,
}
impl Token {
    fn new(kind: TokenKind, pos: TokenPos) -> Token {
        Token { kind, pos }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct TokenPos(u32, u32);
impl TokenPos {
    fn new(lo: u32, hi: u32) -> TokenPos {
        TokenPos(lo, hi)
    }
}

pub(crate) struct StringReader<'a> {
    src: &'a str,
    cursor: Cursor<'a>,
    pos: u32,
}
impl StringReader<'_> {
    fn new<'a>(src: &'a str) -> StringReader<'a> {
        StringReader {
            src,
            cursor: Cursor::new(&src),
            pos: 0,
        }
    }
}

impl StringReader<'_> {
    pub fn next_token(&mut self) -> Token {
        loop {
            let start = self.pos;
            let ch = match self.cursor.bump() {
                Some(c) => c,
                None => return Token::new(TokenKind::EOF, TokenPos(self.pos, self.pos)),
            };

            // Calculate kind. We also advance cursor to the next token in this process
            let kind: TokenKind = match ch {
                c if is_whitespace(c) => self.whitespace(),
                ',' => TokenKind::COMMA,
                ';' => TokenKind::SEMICOLON,
                '(' => TokenKind::LPAREN,
                ')' => TokenKind::RPAREN,
                '[' => TokenKind::LBRACK,
                ']' => TokenKind::RBRACK,
                '{' => TokenKind::RCURLY,
                '}' => TokenKind::LCURLY,
                '.' => TokenKind::DOT,

                ':' => self.colon(),

                '+' => TokenKind::PLUS,
                '-' => TokenKind::MINUS,
                '*' => TokenKind::TIMES,
                '%' => TokenKind::PERCENT,
                '=' => TokenKind::EQ,

                '<' => self.less_than(),
                '>' => self.greater_than(),

                '&' => TokenKind::AND,
                '|' => TokenKind::OR,

                '0'..='9' => self.cook_number(),
                '"' => self.cook_string(),
                '/' => self.slash(),

                c => match c {
                    'a'..='z' | 'A'..='Z' => self.cook_identifier(start),
                    _ => TokenKind::UNKNOWN,
                },
            };
            let token_len = self.cursor.len_advanced();
            self.cursor.reset_len();
            self.pos += token_len;

            if kind == TokenKind::WHITESPACE {
                continue;
            }

            let token = Token::new(kind, TokenPos(start, self.pos));
            return token;
        }
    }

    fn cook_identifier(&mut self, start: u32) -> TokenKind {
        loop {
            match self.cursor.peek_first() {
                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    self.cursor.bump();
                }
                _ => break,
            };
        }

        let end: usize = (start + self.cursor.len_advanced())
            .try_into()
            .expect("input program length falls within usize bounds");
        let start: usize = start
            .try_into()
            .expect("input program length falls within usize bounds");

        let token = &self.src[start..end];
        // TODO: find out if this pattern matching needs to be optimized
        // or if llvm optimizes this automatically
        match token {
            "array" => TokenKind::ARRAY,
            "if" => TokenKind::IF,
            "then" => TokenKind::THEN,
            "else" => TokenKind::ELSE,
            "while" => TokenKind::WHILE,
            "for" => TokenKind::FOR,
            "to" => TokenKind::TO,
            "do" => TokenKind::DO,
            "let" => TokenKind::LET,
            "in" => TokenKind::IN,
            "end" => TokenKind::END,
            "of" => TokenKind::OF,
            "break" => TokenKind::BREAK,
            "function" => TokenKind::FUNCTION,
            "var" => TokenKind::VAR,
            "type" => TokenKind::TYPE,
            "nil" => TokenKind::NIL,
            _ => TokenKind::ID,
        }
    }

    fn whitespace(&mut self) -> TokenKind {
        debug_assert!(is_whitespace(self.cursor.prev()));
        self.cursor.bump_while(is_whitespace);
        TokenKind::WHITESPACE
    }
    fn colon(&mut self) -> TokenKind {
        debug_assert!(self.cursor.prev() == ':');
        if self.cursor.peek_first() == '=' {
            self.cursor.bump();
            TokenKind::ASSIGN
        } else {
            TokenKind::COLON
        }
    }
    fn less_than(&mut self) -> TokenKind {
        debug_assert!(self.cursor.prev() == '<');
        match self.cursor.peek_first() {
            '>' => {
                self.cursor.bump();
                TokenKind::NEQ
            }
            '=' => {
                self.cursor.bump();
                TokenKind::LE
            }
            _ => TokenKind::LT,
        }
    }
    fn greater_than(&mut self) -> TokenKind {
        debug_assert!(self.cursor.prev() == '>');
        match self.cursor.peek_first() {
            '=' => {
                self.cursor.bump();
                TokenKind::GE
            }
            _ => TokenKind::GT,
        }
    }

    fn cook_number(&mut self) -> TokenKind {
        debug_assert!('0' <= self.cursor.prev() && self.cursor.prev() <= '9');
        let mut decimal_found = false;
        loop {
            match self.cursor.peek_first() {
                '0'..='9' => {
                    self.cursor.bump();
                }
                '.' => {
                    if decimal_found {
                        break;
                    }
                    decimal_found = true;
                    self.cursor.bump();
                }
                c if is_whitespace(c) => break,
                _ => break,
            }
        }
        if !decimal_found {
            TokenKind::INT
        } else {
            TokenKind::FLOAT
        }
    }

    fn cook_string(&mut self) -> TokenKind {
        debug_assert!(self.cursor.prev() == '"');
        while let Some(c) = self.cursor.bump() {
            match c {
                '"' => {
                    return TokenKind::STRING;
                }
                '\\' if self.cursor.peek_first() == '\\' || self.cursor.peek_first() == '"' => {
                    // Bump again to skip escaped character.
                    self.cursor.bump();
                }
                _ => continue,
            }
        }
        TokenKind::UNKNOWN
    }

    fn slash(&mut self) -> TokenKind {
        debug_assert!(self.cursor.prev() == '/');
        // it could just be devide
        match self.cursor.peek_first() {
            '*' => self.cook_comment(),
            _ => TokenKind::DIVIDE,
        }
    }

    fn cook_comment(&mut self) -> TokenKind {
        let mut comment_level = 1;
        loop {
            match (self.cursor.peek_first(), self.cursor.peek_second()) {
                ('*', '/') => {
                    comment_level -= 1;
                    self.cursor.bump();
                    self.cursor.bump();
                    if comment_level == 0 {
                        break;
                    }
                }
                ('/', '*') => {
                    comment_level += 1;
                    self.cursor.bump();
                    self.cursor.bump();
                }
                (_, _) => match self.cursor.bump() {
                    Some(_) => continue,
                    None => {
                        if self.cursor.is_eof() {
                            break;
                        }
                    }
                },
            }
        }
        TokenKind::COMMENT
    }
}

fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}
