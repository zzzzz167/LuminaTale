use std::str::Chars;

//Tentative Token
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Character,
    Scene, Show, Hide, Play, Stop, Label, Choice, Lua, Jump, Call,
    EnChoice, EnLua, EnLabel,
    
    LuaBlock(String),
    Ident(String),
    Str(String),
    Num(f64),
    Colon,
    At, Equals, Minus, Dollar,
    Newline,
    Comment(String),
    ParamKey(String),
    Eof,
}

pub struct Lexer<'a> {
    src: &'a str,
    chars: std::iter::Peekable<Chars<'a>>,
    start: usize,
    current: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {src, chars: src.chars().peekable(), start:0, current:0}
    }

    //Advance a character and capture that character
    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.chars.next()
    }

    //Look back to a character
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    //Look back n - 1 character
    fn peek_nth(&mut self, n: usize) -> Option<char> {
        self.src[self.current..].chars().nth(n)
    }

    fn slice(&self) -> &'a str {
        &self.src[self.start..self.current]
    }

    fn slice_from(&self, idx: usize) -> &'a str {
        &self.src[idx..self.current]
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn consume_escape(&mut self) -> char {
        match self.advance() {
            Some('n') => '\n',
            Some('t') => '\t',
            Some('r') => '\r',
            Some('"') => '"',
            Some('\\') => '\\',
            Some(c) => c,
            None => '\\',
        }
    }

    //Handle quotation marks
    //FIXME：单个引号规则混乱
    fn string_literal(&mut self) -> String {
        let quote = self.advance().unwrap();
        let triple = quote == '"' && self.peek() == Some(&'"') && self.peek_nth(1) == Some('"');
        if triple {
            for _ in 0..2 {self.advance();}
            let mut content = String::new();
            loop {
                match self.advance() {
                    Some('"') => {
                        if self.peek() == Some(&'"') && self.peek_nth(1) == Some('"') {
                            for _ in 0..2 {self.advance();}
                            return content;
                        }
                    },
                    Some('\\') => content.push(self.consume_escape()),
                    Some(c) => content.push(c),
                    None => break
                }
            }
            content
        }else {
            let mut content = String::new();
            while let Some(c) = self.advance() {
                match c {
                    '\\' => content.push(self.consume_escape()),
                    '"' => break,
                    _ => content.push(c),
                }
            }
            content
        }
    }
    
    //FIXME:number匹配模糊不清 例如1.0.1也会被认为是数字
    fn number(&mut self) -> f64 {
        self.start = self.current;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || *c == '.' {self.advance();} else { break }
        }
        self.slice().parse().unwrap_or(0.0)
    }


    //TODO:Complete all token processing
    pub fn run(&mut self) -> Vec<Tok> {
        let mut tokens = Vec::new();
        while let Some(c) = self.peek() {
            match c {
                '\n' => { self.advance(); tokens.push(Tok::Newline); },
                ' ' | '\t' | '\r' => { self.skip_whitespace(); },
                '"' | '\'' => tokens.push(Tok::Str(self.string_literal())),
                '@' => { self.advance(); tokens.push(Tok::At); },
                '=' => { self.advance(); tokens.push(Tok::Equals); },
                '$' => { self.advance(); tokens.push(Tok::Dollar); },
                c if c.is_ascii_digit() => tokens.push(Tok::Num(self.number())),
                _ => {self.advance();},
            }
        }

        tokens.push(Tok::Eof);
        tokens
    }

}