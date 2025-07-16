//! Lexical analyser for a simple visual-novel scripting language.
//!
//! The lexer recognises keywords (`scene`, `show`, `choice`, …),
//! string/number literals, Lua blocks and a handful of punctuation
//! tokens.  It also tracks line/column information.
use std::iter::Peekable;
use std::str::Chars;
use unicode_xid::UnicodeXID;

/// All tokens that can be produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Character,
    Scene, Show, Hide, Play, Stop, 
    Label, Choice, Lua, Jump, Call,
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
enum Mode { Normal, Choice}

pub struct Lexer<'a> {
    /// Original source text (kept for debugging).
    src: &'a str,
    /// Character iterator with one-character look-ahead.
    chars: Peekable<Chars<'a>>,
    /// Current line number (1-based).
    line: usize,
    /// Current column number (0-based).
    col: usize,
    /// Are we lexing inside a choice block?
    mode: Mode,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src,
            chars: src.chars().peekable(),
            line: 1,
            col: 0,
            mode: Mode::Normal,
        }
    }
    
    /// Advance the cursor by one character, updating line/column bookkeeping.
    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c == Some('\n') {
            self.line += 1;
            self.col = 0;
        } else { 
            self.col += 1;  
        }
        c
    }
    
    /// Peek at the next character **without** advancing the cursor.
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Peek `n` characters ahead (0 == current peek).
    fn peek_nth(&mut self, n: usize) -> Option<char> {
        let mut iter = self.chars.clone();
        iter.nth(n)
    }

    /// Check whether the next characters *exactly* match `kw` and are followed
    /// by a non-identifier character.
    fn peek_keyword(&mut self, kw: &str) -> bool {
        let mut it = self.chars.clone();
        for ch in kw.chars() {
            if it.next() != Some(ch) {
                return false;
            }
        }
        matches!(it.next(), None | Some(' ') | Some('\t') | Some('\n'))
    }

    /// Consume `kw` without any checks; caller must first call `peek_keyword`.
    fn consume_keyword(&mut self, kw: &str) {
        for _ in kw.chars() {
            self.bump();
        }
    }

    /// Discard spaces and tabs, but **stop at newline**.
    fn skip_space_no_nl(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.bump();
            } else { 
                break;
            }
        }
    }

    /// Convert an escape sequence into the corresponding character.
    fn consume_escape(&mut self) -> char {
        match self.bump() {
            Some('n') => '\n',
            Some('t') => '\t',
            Some('r') => '\r',
            Some('"') => '"',
            Some('\'') => '\'',
            Some('\\') => '\\',
            Some(c) => c,
            None => '\\',
        }
    }

    /// Parse a quoted string until `delim` is reached.
    /// Handles `\"`, `\'`, and other back-slash escapes.
    fn string_literal(&mut self, delim: char) -> String {
        let mut out = String::new();
        while let Some(c) = self.bump() {
            match c {
                '\\' => out.push(self.consume_escape()),
                c if c == delim => break,
                _ => out.push(c),
            }
        }
        out
    }

    /// Parse a triple-quoted string `""" … """`.
    fn triple_quote(&mut self) -> String {
        let mut out = String::new();
        while let Some(c) = self.bump() {
            if c == '"' && self.peek() == Some('"') && self.peek_nth(1) == Some('"') {
                for _ in 0..2{self.bump();}
                break;
            }
            if c == '\\' {
                out.push(self.consume_escape());
            } else {
                out.push(c);
            }
        }
        out
    }
    
    /// Parse the remainder of a `:` line as a string.
    fn colon_line(&mut self) -> String {
        let mut out = String::new();
        self.bump();
        self.skip_space_no_nl();
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            out.push(self.bump().unwrap());
        }
        out
    }

    /// Convert an identifier-like sequence into a keyword token or `Ident`.
    fn keyword_or_ident(&mut self, first: char) -> Tok {
        let mut s = String::from(first);
        while let Some(c) = self.peek() {
            if UnicodeXID::is_xid_continue(c) || c == '_' {
                s.push(self.bump().unwrap());
            } else {
                break;
            }
        }
        match s.as_str() {
            "character" => Tok::Character,
            "scene" => Tok::Scene,
            "show" => Tok::Show,
            "hide" => Tok::Hide,
            "play" => Tok::Play,
            "stop" => Tok::Stop,
            "label" => Tok::Label,
            "choice" => Tok::Choice,
            "lua" => Tok::Lua,
            "jump" => Tok::Jump,
            "call" => Tok::Call,
            "enco" => Tok::EnChoice,
            "enlb" => Tok::EnLabel,
            "enlua" => Tok::EnLua,
            "with" | "at" | "loop" | "volume" | "fade_in" | "fade_out" | "image_tag" | "name"=> {
                Tok::ParamKey(s)
            }
            _ => Tok::Ident(s),
        }
    }

    /// Slurp everything until the terminating `enlua` keyword.
    fn lua_block(&mut self) -> String {
        let mut out = String::new();
        while let Some(c) = self.bump() {
            let mut look = String::new();
            let mut iter = self.chars.clone();
            for _ in 0..5 {
                if let Some(ch) = iter.next() {
                    look.push(ch);
                } else {
                    break;
                }
            }
            if look == "enlua" {
                break
            }
            out.push(c);
        }
        out
    }
    
    /// Parse a number literal or fall back to an identifier.
    fn number_or_ident(&mut self, first: char) -> Tok {
        let mut s = String::from(first);
        let mut allow_dot = true;
        let mut allow_exp = true;

        while let Some(&c) = self.chars.peek(){
            match c {
                '0'..'9' => s.push(self.bump().unwrap()),
                '.' if allow_dot => {
                    allow_dot = false;
                    s.push(self.bump().unwrap());
                },
                'e' | 'E' if allow_exp => {
                    allow_exp = false;
                    s.push(self.bump().unwrap());
                    if let Some(&sign) = self.chars.peek() {
                        if sign == '+' || sign == '-' {
                            s.push(self.bump().unwrap());
                        }
                    }
                },
                _ => break,
            }
        }
        
        // If we trail with a letter/underscore, treat the whole thing as ident.
        if let Some(nc) = self.chars.peek() {
            if nc.is_alphabetic() || *nc == '_' {
                while let Some(c) = self.chars.peek() {
                    if c.is_alphanumeric() || *c=='_' {
                        s.push(self.bump().unwrap());
                    } else { break }
                }
                return Tok::Ident(s);
            }
        }

        let val = s.parse().unwrap_or(0.0);
        Tok::Num(val)
    }

    /// Run the lexer to completion and return the full token stream.
    pub fn run(&mut self) -> Vec<Tok> {
        let mut tokens = Vec::new();
        let mut last_was_newline = false;

        loop {
            self.skip_space_no_nl();
            match self.peek() {
                Some('\n') => { self.bump(); continue;},
                _ => break,
            }
        }

        while let Some(c) = self.peek() {
            if c == '\n' {
                if !last_was_newline {
                    tokens.push(Tok::Newline);
                    last_was_newline = true;
                }
                self.bump();
                continue;
            }
            last_was_newline = false;

            match self.mode {
                Mode::Normal => self.normal(&mut tokens),
                Mode::Choice => self.choice(&mut tokens),
            }
        }

        tokens.push(Tok::Eof);
        tokens
    }

    /// Normal (top-level) lexing rules.
    fn normal(&mut self, tokens: &mut Vec<Tok>) {
        self.skip_space_no_nl();
        let c = match self.peek() {
            Some(c) => c,
            None => return,
        };

        match c {
            '\n' => {},
            '"' => {
                self.bump();
                if self.peek() == Some('"') && self.peek_nth(1) == Some('"') {
                    for _ in 0..2 {self.bump();}
                    tokens.push(Tok::Str(self.triple_quote()));
                } else { tokens.push(Tok::Str(self.string_literal('"'))); }
            }
            '\'' => {
                self.bump();
                tokens.push(Tok::Str(self.string_literal('\'')));
            },
            ':' => {
                tokens.push(Tok::Colon);
                if self.peek_nth(1) == Some('"') && self.peek_nth(2) == Some('"') && self.peek_nth(3) == Some('"') {
                    for _ in 0..4 {self.bump();}
                    let mut content = String::new();
                    content.push_str(&self.triple_quote());
                    tokens.push(Tok::Str(content));
                } else {
                    tokens.push(Tok::Str(self.colon_line()));
                }
            }
            '-' if self.peek_nth(1) == Some('-') => {
                let mut comments = String::new();
                for _ in 0..2 {self.bump();}
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    comments.push(self.bump().unwrap());
                }
                tokens.push(Tok::Comment(comments));
            },
            '@' => {
                self.bump();
                tokens.push(Tok::At);
            },
            '=' => {
                self.bump();
                tokens.push(Tok::Equals);
            },
            '$' => {
                self.bump();
                tokens.push(Tok::Dollar);
            },
            '-' => {
                self.bump();
                tokens.push(Tok::Minus);
            },
            c if c.is_ascii_digit() => {
                let ch = self.bump().unwrap();
                tokens.push(self.number_or_ident(ch));
            },
            c if UnicodeXID::is_xid_continue(c) || c == '_' => {
                let ch = self.bump().unwrap();
                let tok = self.keyword_or_ident(ch);
                if let Tok::Lua = tok {
                    tokens.push(tok);
                    tokens.push(Tok::LuaBlock(self.lua_block()));
                } else if let Tok::Choice = tok {
                    tokens.push(tok);
                    self.mode = Mode::Choice
                }else {
                    tokens.push(tok);
                }
            },
            _ => {self.bump();}
        }
    }

    /// Lexing rules when inside a choice block.
    fn choice(&mut self, tokens: &mut Vec<Tok>) {
        self.skip_space_no_nl();

        // End of choice block?
        if self.peek_keyword("enco") {
            self.consume_keyword("enco");
            tokens.push(Tok::EnChoice);
            self.mode = Mode::Normal;
            return;
        }

        // Collect choice text until ':' or newline
        let mut text = String::new();
        while let Some(c) = self.peek() {
            if c == ':' { break; }
            if c == '\n' {
                tokens.push(Tok::Str(text.trim_end().to_owned()));
                tokens.push(Tok::Newline);
                self.bump();
                return;
            }
            text.push(self.bump().unwrap());
        }
        tokens.push(Tok::Str(text.trim_end().to_owned()));

        // Expect ':' after the text
        if self.peek() == Some(':') {
            self.bump();
            tokens.push(Tok::Colon);
        } else {
            tokens.push(Tok::Newline);
            return;
        }

        // Continue lexing with normal rules for the body of the choice
        self.skip_space_no_nl();
        while let Some(c) = self.peek() {
            if c == '\n' {
                self.bump();
                tokens.push(Tok::Newline);
                break;
            }else {
                self.normal(tokens)
            }
        }

    }
}