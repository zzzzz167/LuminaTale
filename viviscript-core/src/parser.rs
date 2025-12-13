//! Recursive-descent parser that turns a token stream into an AST.
//!
//! The parser is intentionally panic-happy: any syntax error immediately aborts
//! with a descriptive message.  This keeps the implementation small and makes
//! test failures easy to diagnose.

use crate::ast::{AudioAction, AudioOptions, ChoiceArm, SceneImage, Script, ShowAttr, Speaker, Stmt, Transition};
use crate::lexer::{Span, Tok, TokKind};
use regex::Regex;
use log::{debug, error, warn};

/// Parser control-flow state.
#[derive(PartialEq)]
enum Status {
    Run,
    Stop,
}

/// Recursive-descent parser for the visual-novel scripting language.
pub struct Parser<'a> {
    toks: &'a [Tok],
    cursor: usize,
    status: Status,
}

impl<'a> Parser<'a> {
    /// Creates a new parser positioned at the beginning of `toks`.
    pub fn new(toks: &'a [Tok]) -> Self {
        debug!("Parser created with {} tokens", toks.len());
        Self {
            toks,
            cursor: 0,
            status: Status::Run,
        }
    }

    /// Returns the next token *without* advancing the cursor.
    fn peek(&self) -> Option<&TokKind> {
        self.toks.get(self.cursor).map(|t| &t.tok)
    }

    fn peek_line(&self) -> usize {
        self.toks
            .get(self.cursor)
            .map(|t| t.span.line)
            .unwrap_or(0)
    }

    fn peek_nth(&self, n: usize) -> Option<&TokKind> {
        self.toks.get(self.cursor + n).map(|t| &t.tok)
    }

    /// Advances the cursor and returns the consumed token.
    fn bump(&mut self) -> &'a Tok {
        let tok = &self.toks[self.cursor];
        self.cursor += 1;
        tok
    }

    /// Returns the span of the *current* token (useful for error reporting).
    fn span(&self) -> Span {
        self.toks[self.cursor].span
    }

    /// Checks whether the next token has the same discriminant as `k`.
    fn at(&self, k: TokKind) -> bool {
        self.peek()
            .map(|tk| std::mem::discriminant(tk) == std::mem::discriminant(&k))
            .unwrap_or(false)
    }

    /// Consumes the next token and panics if it is not exactly `expect`.
    fn expect(&mut self, expect: TokKind) -> &'a Tok {
        let tok = self.bump();
        if tok.tok != expect {
            error!(
                "line {}: expected {:?}, got {:?}",
                tok.span.line, expect, tok.tok
            );
            std::process::exit(1);
        }
        tok
    }
    
    /// Consumes the next token and panics if it is **not** in `kinds`.
    fn expect_any<I>(&mut self, kinds: I) -> &'a Tok
    where
        I: IntoIterator<Item = TokKind>,
    {
        let kinds: Vec<_> = kinds.into_iter().collect();
        let tok = self.bump();
        if !kinds.iter().any(|k| tok.tok == *k) {
            error!(
                "line {}: expected one of {:?}, got {:?}",
                tok.span.line, kinds, tok.tok
            );
            std::process::exit(1);
        }
        tok
    }

    /// Advances the cursor only if the next token matches `k`.
    fn consume(&mut self, k: TokKind) -> bool {
        if self.peek() == Some(&k) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Consumes and returns an identifier token.
    fn ident(&mut self) -> String {
        match &self.bump().tok {
            TokKind::Ident(s) => String::from(s),
            x => {
                error!("line {}: expected identifier, got {:?}", self.peek_line(), x);
                std::process::exit(1);
            }
        }
    }

    /// Consumes and returns a string literal token.
    fn string(&mut self) -> String {
        match &self.bump().tok {
            TokKind::Str(s) => String::from(s),
            x => {
                error!("line {}: expected string, got {:?}", self.peek_line(), x);
                std::process::exit(1);
            }
        }
    }

    /// Consumes and returns a numeric literal token.
    fn num(&mut self) -> f64 {
        match &self.bump().tok {
            TokKind::Num(n) => *n,
            x => {
                error!("line {}: expected number, got {:?}", self.peek_line(), x);
                std::process::exit(1);
            }
        }
    }

    /// Consumes either a string literal or an identifier.
    fn str_or_ident(&mut self) -> String {
        match self.peek() {
            Some(TokKind::Str(_)) => self.string(),
            Some(TokKind::Ident(_)) => self.ident(),
            _ => {
                error!("line {}: expected string or identifier", self.peek_line());
                std::process::exit(1);
            }
        }
    }

    /// Skips over any trivia (new-lines and comments) at the current position.
    fn skip_trivia(&mut self) {
        while let Some(k) = self.peek() {
            match k {
                TokKind::Newline | TokKind::Comment(_) => {
                    self.bump();
                }
                _ => break,
            }
        }
    }

    fn parse_block(&mut self, terminators: &[TokKind]) -> Vec<Stmt> {
        let mut body = Vec::new();
        loop {
            // 安全性检查
            if self.at(TokKind::Eof) {
                error!("Unexpected EOF inside block");
                std::process::exit(1);
            }

            // 检查终止符
            if let Some(tok) = self.peek() {
                let is_term = terminators.iter().any(|t|
                    std::mem::discriminant(t) == std::mem::discriminant(tok)
                );
                if is_term {
                    return body;
                }
            }

            // 解析下一条语句
            // 注意：stmt() 内部会处理 Newline/Comment 并返回 None
            if let Some(s) = self.stmt() {
                body.push(s);
            } else if self.status == Status::Stop {
                // 如果 stmt 遇到了 EOF 并返回 None，且设置了 Stop
                break;
            }
        }
        body
    }

    /// Entry-point: parses the entire token stream into a [`Script`].
    pub fn parse(mut self) -> Script {
        debug!("Starting parse");
        let mut body = Vec::new();
        while self.peek().is_some() && self.status == Status::Run {
            match self.stmt() {
                Some(s) => body.push(s),
                None => {}
            }
        }
        debug!("Parse complete: {} top-level statements", body.len());
        Script { body }
    }

    /// Top-level statement dispatcher.
    fn stmt(&mut self) -> Option<Stmt> {
        match self.peek() {
            Some(TokKind::Character) => Some(self.character()),
            Some(TokKind::Label) => Some(self.label()),
            Some(TokKind::Choice) => Some(self.choice()),
            Some(TokKind::If) => Some(self.if_stmt()),
            Some(TokKind::Jump) => Some(self.jump()),
            Some(TokKind::Call) => Some(self.call()),
            Some(TokKind::Colon) => Some(self.narration()),
            Some(TokKind::Play) => Some(self.play_audio()),
            Some(TokKind::Stop) => Some(self.stop_audio()),
            Some(TokKind::Scene) => Some(self.scene()),
            Some(TokKind::Hide) => Some(self.hide()),
            Some(TokKind::Dollar) => Some(self.dollar_luablock()),
            Some(TokKind::Lua) => Some(self.luablock()),
            Some(TokKind::Ident(_)) => Some(self.dialogue()),
            Some(TokKind::Show) => Some(self.show()),
            Some(TokKind::Newline) | Some(TokKind::Comment(_)) => {
                self.skip_trivia();
                None
            }
            Some(TokKind::Eof) => {
                self.status = Status::Stop;
                None
            }
            _ => {
                let line = self.peek_line();
                let tok = self.bump();
                warn!("line {}: skipped unexpected token {:?}", line, tok.tok);
                None
            }
        }
    }

    /// Parses a `label <id> enlb` statement.
    fn label(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Label);
        let id = self.ident();
        let mut body = Vec::new();
        while !matches!(self.peek(), Some(TokKind::EnLabel) | None) {
            if self.at(TokKind::Eof) {
                error!("line {}: unexpected EOF inside label '{}'", span.line, id);
                std::process::exit(1);
            }
            match self.stmt() {
                Some(s) => body.push(s),
                None => {}
            }
        }
        self.bump();
        Stmt::Label { span, id, body }
    }

    /// Parses a `jump <label>` statement.
    fn jump(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Jump);
        let target = self.ident();
        Stmt::Jump { span, target }
    }
    
    /// Parses a `call <label>` statement.
    fn call(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Call);
        let target = self.ident();
        Stmt::Call { span, target }
    }
    
    /// Parses a `choice [title] ... enco` statement.
    fn choice(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Choice);

        self.skip_trivia();

        let mut title = None;
        if let Some(TokKind::Str(_)) = self.peek() {
            if self.peek_nth(1) != Some(&TokKind::Colon) {
                title = Some(self.string());
            }
        }

        let mut arms = Vec::new();

        while !self.at(TokKind::EnChoice) {
            self.skip_trivia();
            if self.at(TokKind::EnChoice) { break; }

            let text = if self.at(TokKind::Str("".into())) {
                self.string()
            } else {
                let line = self.peek_line();
                error!("line {}: Expected string literal for choice option, got {:?}", line, self.peek());
                std::process::exit(1);
            };

            self.expect(TokKind::Colon);
            let body = self.parse_block(&[TokKind::Str("".into()), TokKind::EnChoice]);

            arms.push(ChoiceArm { text, body });
        }

        self.expect(TokKind::EnChoice);
        Stmt::Choice { span, title, arms, id: None }
    }

    /// Parses a character statement.
    fn character(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Character);
        let id = self.ident();
        let mut name = None;
        let mut image_tag = None;
        let mut voice_tag = None;
        while let Some(TokKind::ParamKey(k)) = self.peek() {
            let key = k.clone();
            self.bump();
            self.expect(TokKind::Equals);
            let val = self.str_or_ident();
            match key.as_str() {
                "name" => name = Some(val),
                "image_tag" => image_tag = Some(val),
                "voice_tag" => voice_tag = Some(val),
                _ => {
                    error!("line {}: unknown parameter key '{}'", self.peek_line(), key);
                    std::process::exit(1);
                }
            }
        }
        Stmt::CharacterDef {
            span,
            id,
            name: name.expect("name"),
            image_tag,
            voice_tag,
        }
    }
    
    /// Parses `<speaker> [ @ alias ]: "text"` dialogue.
    fn dialogue(&mut self) -> Stmt {
        let span = self.span();
        let name = self.ident();
        let alias = if self.at(TokKind::At) {
            self.bump();
            Some(self.str_or_ident())
        } else {
            None
        };

        self.expect(TokKind::Colon);
        let raw = self.string();
        
        let re = Regex::new(r"\(([^()]*)\)$").unwrap();
        let (text, voice_index) = if let Some(caps) = re.captures(&raw) {
            let idx = caps.get(1).unwrap().as_str().to_string();
            let txt = re.replace(&raw, "").trim_end().to_string();
            (txt, Some(idx))
        } else {
            (raw, None)
        };

        Stmt::Dialogue {
            span,
            speaker: Speaker { name, alias },
            text,
            voice_index,
        }
    }

    /// Parses a colon-style narration block.
    fn narration(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Colon);
        let mut lines = Vec::new();
        if self.at(TokKind::Str("".into())) {
            for i in self.string().trim().lines() {
                lines.push(i.to_string());
            }
        }
        Stmt::Narration { span, lines }
    }

    /// Parses a `lua ... enlua` block.
    fn luablock(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Lua);
        if !self.at(TokKind::LuaBlock("".into())) {
            error!("line {}:expected lua block, but got {:?}", self.peek_line(),self.bump());
            std::process::exit(1);
        }
        let code = self.bump().tok.as_str().unwrap().to_string();
        self.skip_trivia();
        self.expect(TokKind::EnLua);

        Stmt::LuaBlock {span, code}
    }

    /// Parses a `$lua_block` inline Lua expression.
    fn dollar_luablock(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Dollar);
        if !self.at(TokKind::LuaBlock("".into())) {
            error!("line {}:expected lua block, but got {:?}", self.peek_line(),self.bump());
            std::process::exit(1);
        }
        let code = self.bump().tok.as_str().unwrap().to_string();

        Stmt::LuaBlock {span, code}

    }

    /// Parses `play <channel> <resource> [options...] `.
    fn play_audio(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Play);
        let action = AudioAction::Play;
        let mut r#loop = false;
        let channel = self.str_or_ident();
        let resource = Some(self.str_or_ident());

        let mut volume = None;
        let mut fade_in = None;
        let mut fade_out = None;
        let mut have_a_loop = false;
        while let Some(TokKind::ParamKey(k) | TokKind::Flag(k)) = self.peek() {
            let key = k.clone();
            if self.at(TokKind::Flag("".into())) {
                self.bump();
                if have_a_loop {
                    error!("line {}: Already had a loop define",self.peek_line());
                    std::process::exit(1);
                }
                match key.as_str() {
                    "loop" => r#loop = true,
                    "noloop" => r#loop = false,
                    _ => {
                        error!("line {}: Not available flag named {}",self.peek_line(), key);
                        std::process::exit(1);
                    },
                }
                have_a_loop = true;
            } else {
                self.bump();
                self.expect(TokKind::Equals);
                let val = self.num() as f32;
                match key.as_str() {
                    "volume" => volume = Some(val),
                    "fade_in" => fade_in = Some(val),
                    "fade_out" => fade_out = Some(val),
                    _ => {
                        error!("line {}: unknown param '{}'", self.peek_line(), key);
                        std::process::exit(1);
                    }
                }
            }
        }

        let options = AudioOptions {
            volume,
            fade_in,
            fade_out,
            r#loop,
        };
        Stmt::Audio {
            span,
            action,
            channel,
            resource,
            options,
        }
    }

    /// Parses `stop <channel> [ options... ]`.
    fn stop_audio(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Stop);
        let action = AudioAction::Stop;
        let channel = self.str_or_ident();
        let mut fade_out = None;
        while let Some(TokKind::ParamKey(k)) = self.peek() {
            let key = k.clone();
            self.bump();
            self.expect(TokKind::Equals);
            let val = self.num() as f32;
            match key.as_str() {
                "fade_out" => fade_out = Some(val),
                _ => { 
                    error!("line {}: unknown param '{}'", self.peek_line(), key);
                    std::process::exit(1);
                },
            }
        }
        let options = AudioOptions {
            volume: None,
            fade_in: None,
            r#loop: false,
            fade_out,
        }; //r#loop didn't have any effect at 'stop'
        Stmt::Audio {
            span,
            action,
            channel,
            resource: None,
            options,
        }
    }

    /// Parses `scene [ <image> [ attrs... ] ] [ with <effect> ]`.
    fn scene(&mut self) -> Stmt {
        let span = self.span();
        let mut image = None;
        let mut transition = None;
        self.expect(TokKind::Scene);

        match self.peek() {
            Some(TokKind::Ident(_)) => {
                let prefix = self.ident();
                let mut attrs_vec = Vec::new();
                while let Some(TokKind::Str(_) | TokKind::Ident(_)) = self.peek() {
                    attrs_vec.push(self.str_or_ident());
                }
                let mut attrs = None;
                if !attrs_vec.is_empty() {
                    attrs = Some(attrs_vec);
                }
                image = Some(SceneImage { prefix, attrs });
            }
            Some(TokKind::Str(_)) => {
                let prefix = self.string();
                let attrs = None;
                let next = self.peek();
                if next != Some(&TokKind::Reserved("with".to_string()))
                    && next != Some(&TokKind::Newline)
                    && next != Some(&TokKind::Eof)
                    && !self.at(TokKind::Comment("".into()))
                {
                    error!("line {}:expected Newline or Eof",self.peek_line());
                    std::process::exit(1);
                }
                image = Some(SceneImage { prefix, attrs })
            }
            _ => {}
        }

        match self.peek() {
            Some(TokKind::Reserved(k)) => {
                if k.as_str() == "with" {
                    self.bump();
                    let effect = self.bump().tok.as_str().unwrap().to_string();
                    transition = Some(Transition { effect });
                    if self.peek() != Some(&TokKind::Newline)
                        && self.peek() != Some(&TokKind::Eof)
                        && !self.at(TokKind::Comment("".into()))
                    {
                        error!("line {}:expected Newline or Eof",self.peek_line());
                        std::process::exit(1);
                    }
                } else {
                    error!("line {}:Not available reserved keyword {}", self.peek_line(),k);
                    std::process::exit(1);
                }
            }
            _ => {}
        }

        Stmt::Scene {
            span,
            image,
            transition,
        }
    }

    /// Parses `show <target> [attr|-attr...] [at <pos>] [with <effect>]`.
    fn show(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Show);
        let target = self.str_or_ident();
        let mut attrs = None;
        let mut position = None;
        let mut transition = None;
        let mut attrs_vec = Vec::new();
        while let Some(k) = self.peek() {
            match k {
                TokKind::Minus => {
                    self.bump();
                    attrs_vec.push(ShowAttr::Remove(self.str_or_ident()));
                },
                TokKind::Str(_) | TokKind::Ident(_) => {
                    attrs_vec.push(ShowAttr::Add(self.str_or_ident()))
                }
                _ => break
            }
        }
        if !attrs_vec.is_empty() {
            attrs = Some(attrs_vec);
        }
        
        while let Some(TokKind::Reserved(k)) = self.peek() {
            if k.as_str() == "with" {
                self.bump();
                let effect = self.bump().tok.as_str().unwrap().to_string();
                transition = Some(Transition { effect });
            } else if k.as_str() == "at" { 
                self.bump();
                position = Some(self.bump().tok.as_str().unwrap().to_string());
            }
        }

        if !self.at(TokKind::Comment("".into())) {
            self.expect_any([TokKind::Eof,TokKind::Newline]);
        }

        Stmt::Show {span,target,attrs,position,transition}
    }

    /// Parses `hide <target>`.
    fn hide(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Hide);
        let target = self.str_or_ident();

        let mut transition = None;
        if let Some(TokKind::Reserved(k)) = self.peek() {
            if k.as_str() == "with" {
                self.bump();
                let effect = self.bump().tok.as_str().unwrap().to_string();
                transition = Some(Transition { effect });
            }
        }

        if !self.at(TokKind::Comment("".into())) {
            self.expect_any([TokKind::Eof,TokKind::Newline]);
        }
        Stmt::Hide {span, target, transition}
    }

    fn if_stmt(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::If);

        let mut branches = Vec::new();

        let cond = match &self.bump().tok {
            TokKind::Condition(s) => s.clone(),
            _ => {
                error!("line {}: Expected condition after 'if'", span.line);
                std::process::exit(1);
            }
        };

        let body = self.parse_block(&[TokKind::Elif, TokKind::Else, TokKind::EnIf]);
        branches.push((cond, body));

        while self.at(TokKind::Elif) {
            self.bump();
            let cond = match &self.bump().tok {
                TokKind::Condition(s) => s.clone(),
                _ => {
                    error!("Expected condition after 'elif'");
                    std::process::exit(1);
                }
            };
            let body = self.parse_block(&[TokKind::Elif, TokKind::Else, TokKind::EnIf]);
            branches.push((cond, body));
        }

        let mut else_branch = None;
        if self.consume(TokKind::Else) {
            if self.at(TokKind::Colon) { self.bump(); }

            let body = self.parse_block(&[TokKind::EnIf]);
            else_branch = Some(body);
        }

        self.expect(TokKind::EnIf);

        Stmt::If { span, branches, else_branch, id: None}
    }
}
