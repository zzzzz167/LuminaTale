//! Recursive-descent parser that turns a token stream into an AST.
//!
//! The parser is intentionally panic-happy: any syntax error immediately aborts
//! with a descriptive message.  This keeps the implementation small and makes
//! test failures easy to diagnose.

use crate::ast::{AudioAction, AudioOptions, ChoiceArm, ContainerKind, SceneImage, Script, ShowAttr, Speaker, Stmt, Transition, UiProp, UiStmt, WidgetKind};
use crate::lexer::{Span, Tok, TokKind};
use regex::Regex;
use log::{debug, error, warn};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub msg: String,
}

/// Recursive-descent parser for the visual-novel scripting language.
pub struct Parser<'a> {
    toks: &'a [Tok],
    cursor: usize,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser positioned at the beginning of `toks`.
    pub fn new(toks: &'a [Tok]) -> Self {
        debug!("Parser created with {} tokens", toks.len());
        Self {
            toks,
            cursor: 0,
            errors: Vec::new(),
        }
    }

    /// Returns the next token *without* advancing the cursor.
    fn peek(&self) -> Option<&'a TokKind> {
        self.toks.get(self.cursor).map(|t| &t.tok)
    }

    fn peek_line(&self) -> usize {
        self.toks
            .get(self.cursor)
            .map(|t| t.span.line)
            .unwrap_or(0)
    }

    fn peek_nth(&self, n: usize) -> Option<&'a TokKind> {
        self.toks.get(self.cursor + n).map(|t| &t.tok)
    }

    /// Advances the cursor and returns the consumed token.
    fn bump(&mut self) -> &'a Tok {
        if self.cursor < self.toks.len() {
            let t = &self.toks[self.cursor];
            self.cursor += 1;
            t
        } else {
            // 安全边界，防止溢出
            if self.toks.is_empty() {
                panic!("Parser created with empty token stream");
            }
            &self.toks[self.toks.len() - 1]
        }
    }

    /// Returns the span of the *current* token (useful for error reporting).
    fn span(&self) -> Span {
        if self.cursor > 0 {
            self.toks[self.cursor - 1].span
        } else {
            self.toks[0].span
        }
    }

    /// Checks whether the next token has the same discriminant as `k`.
    fn at(&self, k: TokKind) -> bool {
        self.peek()
            .map(|tk| std::mem::discriminant(tk) == std::mem::discriminant(&k))
            .unwrap_or(false)
    }

    fn error<T>(&mut self, msg: impl Into<String>) -> Result<T, ()> {
        let line = self.peek_line();
        self.errors.push(ParseError { line, msg: msg.into() });
        Err(())
    }

    /// Consumes the next token and panics if it is not exactly `expect`.
    fn expect(&mut self, expect: TokKind) -> Result<&'a Tok, ()> {
        let tok = self.bump();
        if std::mem::discriminant(&tok.tok) != std::mem::discriminant(&expect) {
            return self.error(format!("Expected {:?}, got {:?}", expect, tok.tok));
        }
        Ok(tok)
    }
    
    /// Consumes the next token and panics if it is **not** in `kinds`.
    fn expect_any<I>(&mut self, kinds: I) -> Result<&'a Tok, ()>
    where
        I: IntoIterator<Item = TokKind>,
    {
        let kinds: Vec<_> = kinds.into_iter().collect();
        let tok = self.bump();
        if !kinds.iter().any(|k| std::mem::discriminant(&tok.tok) == std::mem::discriminant(k)) {
            return self.error(format!("Expected one of {:?}, got {:?}", kinds, tok.tok));
        }
        Ok(tok)
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
    fn ident(&mut self) -> Result<String, ()> {
        match &self.bump().tok {
            TokKind::Ident(s) => Ok(s.clone()),
            x => self.error(format!("Expected identifier, got {:?}", x)),
        }
    }

    /// Consumes and returns a string literal token.
    fn string(&mut self) -> Result<String, ()> {
        match &self.bump().tok {
            TokKind::Str(s) => Ok(s.clone()),
            x => self.error(format!("Expected string, got {:?}", x)),
        }
    }

    /// Consumes and returns a numeric literal token.
    fn num(&mut self) -> Result<f64, ()> {
        match &self.bump().tok {
            TokKind::Num(n) => Ok(*n),
            x => self.error(format!("Expected number, got {:?}", x)),
        }
    }

    /// Consumes either a string literal or an identifier.
    fn str_or_ident(&mut self) -> Result<String, ()> {
        match self.peek() {
            Some(TokKind::Str(_)) => self.string(),
            Some(TokKind::Ident(_)) => self.ident(),
            Some(TokKind::ParamKey(s)) | Some(TokKind::Flag(s)) => {
                self.bump();
                Ok(s.clone())
            },
            Some(TokKind::Num(n)) => {
                self.bump();
                Ok(n.to_string())
            },
            Some(x) => {
                let msg = format!("Expected string or identifier, got {:?}", x);
                self.bump();
                self.error(msg)
            },
            None => self.error("Unexpected EOF"),
        }
    }

    fn at_val(&self) -> bool {
        matches!(self.peek(),
            Some(
                TokKind::Str(_) |
                TokKind::Ident(_)|
                TokKind::ParamKey(_) |
                TokKind::Flag(_) |
                TokKind::Num(_)
            ))
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

    fn synchronize(&mut self) {
        while let Some(k) = self.peek() {
            if matches!(k, TokKind::Newline | TokKind::Eof) {
                self.bump(); // 吃掉换行符
                return;
            }
            self.bump();
        }
    }

    fn parse_block(&mut self, terminators: &[TokKind]) -> Result<Vec<Stmt>, ()> {
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
                    return Ok(body);
                }
            }

            // 解析下一条语句
            // 注意：stmt() 内部会处理 Newline/Comment 并返回 None
            match self.stmt() {
                Ok(Some(s)) => body.push(s),
                Ok(None) => {}, // 空行
                Err(_) => {
                    self.synchronize();
                    // 在 block 内出错不应直接跳出 block，而是尝试解析下一行
                }
            }
        }
    }

    /// Entry-point: parses the entire token stream into a [`Script`].
    pub fn parse(mut self) -> Result<Script, Vec<ParseError>> {
        debug!("Starting parse");
        let mut body = Vec::new();
        while self.peek().is_some() {
            if self.at(TokKind::Eof) {
                break;
            }

            match self.stmt() {
                Ok(Some(s)) => body.push(s),
                Ok(None) => {}
                Err(_) => {
                    self.synchronize();
                }
            }
        }
        debug!("Parse complete: {} top-level statements", body.len());
        if self.errors.is_empty() {
            Ok(Script { body })
        } else {
            Err(self.errors)
        }
    }

    /// Top-level statement dispatcher.
    fn stmt(&mut self) -> Result<Option<Stmt>, ()> {
        match self.peek() {
            Some(TokKind::Character) => Ok(Some(self.character()?)),
            Some(TokKind::Label) => Ok(Some(self.label()?)),
            Some(TokKind::Choice) => Ok(Some(self.choice()?)),
            Some(TokKind::If) => Ok(Some(self.if_stmt()?)),
            Some(TokKind::Jump) => Ok(Some(self.jump()?)),
            Some(TokKind::Call) => Ok(Some(self.call()?)),
            Some(TokKind::Colon) => Ok(Some(self.narration()?)),
            Some(TokKind::Play) => Ok(Some(self.play_audio()?)),
            Some(TokKind::Stop) => Ok(Some(self.stop_audio()?)),
            Some(TokKind::Scene) => Ok(Some(self.scene()?)),
            Some(TokKind::Hide) => Ok(Some(self.hide()?)),
            Some(TokKind::Dollar) => Ok(Some(self.dollar_luablock()?)),
            Some(TokKind::Lua) => Ok(Some(self.luablock()?)),
            Some(TokKind::Ident(_)) => Ok(Some(self.dialogue()?)),
            Some(TokKind::Show) => Ok(Some(self.show()?)),
            Some(TokKind::Screen) => Ok(Some(self.screen_def()?)),
            Some(TokKind::Newline) | Some(TokKind::Comment(_)) => {
                self.skip_trivia();
                Ok(None)
            }
            Some(TokKind::Eof) => {
                self.bump();
                Ok(None)
            }
            _ => {
                let t = self.bump();
                // 不使用 self.error 以避免这里产生 parse error，
                // 对于未知的 token 我们只是由 log 警告并跳过
                warn!("line {}: skipped unexpected token {:?}", t.span.line, t.tok);
                Ok(None)
            }
        }
    }

    /// Parses a `label <id> enlb` statement.
    fn label(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Label)?;
        let id = self.ident()?;
        let mut body = Vec::new();
        while !matches!(self.peek(), Some(TokKind::EnLabel) | None) {
            if self.at(TokKind::Eof) {
                return self.error(format!("Unexpected EOF inside label '{}'", id));
            }
            match self.stmt() {
                Ok(Some(s)) => body.push(s),
                Ok(None) => {},
                Err(_) => self.synchronize(),
            }
        }
        self.expect(TokKind::EnLabel)?;
        Ok(Stmt::Label { span, id, body })
    }

    /// Parses a `jump <label>` statement.
    fn jump(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Jump)?;
        let target = self.ident()?;
        Ok(Stmt::Jump { span, target })
    }
    
    /// Parses a `call <label>` statement.
    fn call(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Call)?;
        let target = self.ident()?;
        Ok(Stmt::Call { span, target })
    }
    
    /// Parses a `choice [title] ... enco` statement.
    fn choice(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Choice)?;

        self.skip_trivia();

        let mut title = None;
        if let Some(TokKind::Str(_)) = self.peek() {
            if self.peek_nth(1) != Some(&TokKind::Colon) {
                title = Some(self.string()?);
            }
        }

        let mut arms = Vec::new();

        while !self.at(TokKind::EnChoice) {
            self.skip_trivia();
            if self.at(TokKind::EnChoice) { break; }

            if self.at(TokKind::Eof) {
                return self.error("Unexpected EOF inside choice");
            }

            let text = if self.at(TokKind::Str("".into())) {
                self.string()?
            } else {
                return self.error("Expected string literal for choice option");
            };

            self.expect(TokKind::Colon)?;
            let body = self.parse_block(&[TokKind::Str("".into()), TokKind::EnChoice])?;

            arms.push(ChoiceArm { text, body });
        }

        self.expect(TokKind::EnChoice)?;
        Ok(Stmt::Choice { span, title, arms, id: None })
    }

    /// Parses a character statement.
    fn character(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Character)?;
        let id = self.ident()?;
        let mut name = None;
        let mut image_tag = None;
        let mut voice_tag = None;
        while let Some(TokKind::ParamKey(k)) = self.peek() {
            let key = k.clone();
            self.bump();
            self.expect(TokKind::Equals)?;
            let val = self.str_or_ident()?;
            match key.as_str() {
                "name" => name = Some(val),
                "image_tag" => image_tag = Some(val),
                "voice_tag" => voice_tag = Some(val),
                _ => return self.error(format!("Unknown parameter key '{}'", key)),
            }
        }

        if name.is_none() {
            return self.error("Character definition requires 'name' parameter");
        }

        Ok(Stmt::CharacterDef {
            span,
            id,
            name: name.unwrap(),
            image_tag,
            voice_tag,
        })
    }
    
    /// Parses `<speaker> [ @ alias ]: "text"` dialogue.
    fn dialogue(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        let name = self.ident()?;
        let alias = if self.at(TokKind::At) {
            self.bump();
            Some(self.str_or_ident()?)
        } else {
            None
        };

        self.expect(TokKind::Colon)?;
        let raw = self.str_or_ident()?;
        
        let re = Regex::new(r"\(([^()]*)\)$").unwrap();
        let (text, voice_index) = if let Some(caps) = re.captures(&raw) {
            let idx = caps.get(1).unwrap().as_str().to_string();
            let txt = re.replace(&raw, "").trim_end().to_string();
            (txt, Some(idx))
        } else {
            (raw, None)
        };

        Ok(Stmt::Dialogue {
            span,
            speaker: Speaker { name, alias },
            text,
            voice_index,
        })
    }

    /// Parses a colon-style narration block.
    fn narration(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Colon)?;
        let mut lines = Vec::new();
        if self.at(TokKind::Str("".into())) {
            let s = self.string()?;
            for i in s.trim().lines() {
                lines.push(i.to_string());
            }
        }
        Ok(Stmt::Narration { span, lines })
    }

    /// Parses a `lua ... enlua` block.
    fn luablock(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Lua)?;
        if !matches!(self.peek(), Some(TokKind::LuaBlock(_))) {
            return self.error("Expected lua block content");
        }
        let code = match self.bump().tok.clone() {
            TokKind::LuaBlock(s) => s,
            _ => unreachable!(),
        };
        self.skip_trivia();
        self.expect(TokKind::EnLua)?;

        Ok(Stmt::LuaBlock { span, code })
    }

    /// Parses a `$lua_block` inline Lua expression.
    fn dollar_luablock(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Dollar)?;
        if !matches!(self.peek(), Some(TokKind::LuaBlock(_))) {
            return self.error("Expected lua block content after $");
        }
        let code = match self.bump().tok.clone() {
            TokKind::LuaBlock(s) => s,
            _ => unreachable!(),
        };

        Ok(Stmt::LuaBlock { span, code })
    }

    /// Parses `play <channel> <resource> [options...] `.
    fn play_audio(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Play)?;
        let action = AudioAction::Play;
        let mut r#loop = false;
        let channel = self.str_or_ident()?;
        let resource = Some(self.str_or_ident()?);

        let mut volume = None;
        let mut fade_in = None;
        let mut fade_out = None;
        let mut have_a_loop = false;

        loop {
            // Check for ParamKey or Flag
            match self.peek() {
                Some(TokKind::Flag(k)) => {
                    let key = k.clone();
                    self.bump();
                    if have_a_loop {
                        return self.error("Already had a loop define");
                    }
                    match key.as_str() {
                        "loop" => r#loop = true,
                        "noloop" => r#loop = false,
                        _ => return self.error(format!("Unknown flag {}", key)),
                    }
                    have_a_loop = true;
                }
                Some(TokKind::ParamKey(k)) => {
                    let key = k.clone();
                    self.bump();
                    self.expect(TokKind::Equals)?;
                    let val = self.num()? as f32;
                    match key.as_str() {
                        "volume" => volume = Some(val),
                        "fade_in" => fade_in = Some(val),
                        "fade_out" => fade_out = Some(val),
                        _ => return self.error(format!("Unknown param '{}'", key)),
                    }
                }
                _ => break,
            }
        }

        let options = AudioOptions {
            volume,
            fade_in,
            fade_out,
            r#loop,
        };
        Ok(Stmt::Audio {
            span,
            action,
            channel,
            resource,
            options,
        })
    }

    /// Parses `stop <channel> [ options... ]`.
    fn stop_audio(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Stop)?;
        let action = AudioAction::Stop;
        let channel = self.str_or_ident()?;
        let mut fade_out = None;

        while let Some(TokKind::ParamKey(k)) = self.peek() {
            let key = k.clone();
            self.bump();
            self.expect(TokKind::Equals)?;
            let val = self.num()? as f32;
            match key.as_str() {
                "fade_out" => fade_out = Some(val),
                _ => return self.error(format!("Unknown param '{}'", key)),
            }
        }

        let options = AudioOptions {
            volume: None,
            fade_in: None,
            r#loop: false,
            fade_out,
        };
        Ok(Stmt::Audio {
            span,
            action,
            channel,
            resource: None,
            options,
        })
    }

    /// Parses `scene [ <image> [ attrs... ] ] [ with <effect> ]`.
    fn scene(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        let mut image = None;
        let mut transition = None;
        self.expect(TokKind::Scene)?;

        if matches!(self.peek(), Some(TokKind::Ident(_))) {
            let prefix = self.ident()?;
            let mut attrs_vec = Vec::new();
            while let Some(TokKind::Str(_) | TokKind::Ident(_)) = self.peek() {
                attrs_vec.push(self.str_or_ident()?);
            }
            let mut attrs = None;
            if !attrs_vec.is_empty() {
                attrs = Some(attrs_vec);
            }
            image = Some(SceneImage { prefix, attrs });
        } else if matches!(self.peek(), Some(TokKind::Str(_))) {
            let prefix = self.string()?;
            let attrs = None;
            // check terminator
            match self.peek() {
                Some(TokKind::Reserved(s)) if s == "with" => {},
                Some(TokKind::Newline) | Some(TokKind::Eof) | Some(TokKind::Comment(_)) => {},
                _ => return self.error("Expected Newline, Eof or 'with'"),
            }
            image = Some(SceneImage { prefix, attrs })
        }

        if let Some(TokKind::Reserved(k)) = self.peek() {
            if k == "with" {
                self.bump(); // eat 'with'
                let effect = self.str_or_ident()?;
                transition = Some(Transition { effect });
            }
        }

        if !self.at(TokKind::Comment("".into())) {
            self.expect_any([TokKind::Eof, TokKind::Newline])?;
        }

        Ok(Stmt::Scene {
            span,
            image,
            transition,
        })
    }

    /// Parses `show <target> [attr|-attr...] [at <pos>] [with <effect>]`.
    fn show(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Show)?;
        let target = self.str_or_ident()?;
        let mut attrs = None;
        let mut position = None;
        let mut transition = None;
        let mut attrs_vec = Vec::new();

        while let Some(k) = self.peek() {
            match k {
                TokKind::Minus => {
                    self.bump();
                    attrs_vec.push(ShowAttr::Remove(self.str_or_ident()?));
                }
                TokKind::Str(_) | TokKind::Ident(_) => {
                    attrs_vec.push(ShowAttr::Add(self.str_or_ident()?));
                }
                _ => break,
            }
        }
        if !attrs_vec.is_empty() {
            attrs = Some(attrs_vec);
        }

        while let Some(TokKind::Reserved(k)) = self.peek() {
            if k == "with" {
                self.bump();
                let effect = self.str_or_ident()?;
                transition = Some(Transition { effect });
            } else if k == "at" {
                self.bump();
                position = Some(self.str_or_ident()?);
            } else {
                break;
            }
        }

        if !self.at(TokKind::Comment("".into())) {
            self.expect_any([TokKind::Eof, TokKind::Newline])?;
        }

        Ok(Stmt::Show {
            span,
            target,
            attrs,
            position,
            transition,
        })
    }

    /// Parses `hide <target>`.
    fn hide(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Hide)?;
        let target = self.str_or_ident()?;

        let mut transition = None;
        if let Some(TokKind::Reserved(k)) = self.peek() {
            if k == "with" {
                self.bump();
                let effect = self.str_or_ident()?;
                transition = Some(Transition { effect });
            }
        }

        if !self.at(TokKind::Comment("".into())) {
            self.expect_any([TokKind::Eof, TokKind::Newline])?;
        }
        Ok(Stmt::Hide {
            span,
            target,
            transition,
        })
    }

    fn if_stmt(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::If)?;

        let mut branches = Vec::new();

        let cond = match &self.bump().tok {
            TokKind::Condition(s) => s.clone(),
            _ => return self.error("Expected condition after 'if'"),
        };

        let body = self.parse_block(&[TokKind::Elif, TokKind::Else, TokKind::EnIf])?;
        branches.push((cond, body));

        while self.at(TokKind::Elif) {
            self.bump();
            let cond = match &self.bump().tok {
                TokKind::Condition(s) => s.clone(),
                _ => return self.error("Expected condition after 'elif'"),
            };
            let body = self.parse_block(&[TokKind::Elif, TokKind::Else, TokKind::EnIf])?;
            branches.push((cond, body));
        }

        let mut else_branch = None;
        if self.consume(TokKind::Else) {
            if self.at(TokKind::Colon) {
                self.bump();
            }

            let body = self.parse_block(&[TokKind::EnIf])?;
            else_branch = Some(body);
        }

        self.expect(TokKind::EnIf)?;

        Ok(Stmt::If {
            span,
            branches,
            else_branch,
            id: None,
        })
    }

    fn screen_def(&mut self) -> Result<Stmt, ()> {
        let span = self.span();
        self.expect(TokKind::Screen)?;
        let id = self.ident()?;

        let mut ui_nodes = Vec::new();

        loop {
            self.skip_trivia();
            if self.at(TokKind::EnScreen) || self.at(TokKind::Eof) {
                break;
            }

            if let Some(node) = self.ui_node()? {
                ui_nodes.push(node);
            }
        }

        self.expect(TokKind::EnScreen)?;
        Ok(Stmt::ScreenDef { span, id, root: ui_nodes })
    }

    fn ui_node(&mut self) -> Result<Option<UiStmt>, ()> {
        let span = self.span();

        if let Some(kind) = self.match_container_start() {
            self.bump(); // consume vbox/zbox...
            let props = self.parse_ui_props()?; // width=100 align=center

            let mut children = Vec::new();
            let end_tok = self.container_end_tok(kind);

            loop {
                self.skip_trivia();
                if self.at(end_tok.clone()) || self.at(TokKind::Eof) {
                    break;
                }
                if let Some(child) = self.ui_node()? {
                    children.push(child);
                }
            }
            self.expect(end_tok)?;

            return Ok(Some(UiStmt::Container { span, kind, props, children }));
        }

        if let Some(kind) = self.match_widget_start() {
            self.bump();
            // Widget 通常跟着一个值 (string 或 ident)
            // e.g., button "Start"  image "bg.png"
            let mut value = None;
            if self.at_val() {
                value = Some(self.str_or_ident()?);
            }

            let props = self.parse_ui_props()?;
            // Widget 不需要冒号和结束符，它是单行的

            return Ok(Some(UiStmt::Widget { span, kind, value, props }));
        }

        if self.at(TokKind::Newline) || self.at(TokKind::Eof) {
            self.bump();
            return Ok(None);
        }

        // 4. 注释
        if let TokKind::Comment(_) = self.peek().unwrap() {
            self.bump();
            return Ok(None);
        }

        self.error(format!("Expected UI component (vbox, button, etc.), got {:?}", self.peek()))
    }

    fn match_container_start(&self) -> Option<ContainerKind> {
        match self.peek() {
            Some(TokKind::VBox) => Some(ContainerKind::VBox),
            Some(TokKind::HBox) => Some(ContainerKind::HBox),
            Some(TokKind::ZBox) => Some(ContainerKind::ZBox),
            Some(TokKind::Frame) => Some(ContainerKind::Frame),
            _ => None
        }
    }

    fn container_end_tok(&self, kind: ContainerKind) -> TokKind {
        match kind {
            ContainerKind::VBox => TokKind::EnVBox,
            ContainerKind::HBox => TokKind::EnHBox,
            ContainerKind::ZBox => TokKind::EnZBox,
            ContainerKind::Frame => TokKind::EnFrame,
        }
    }

    fn match_widget_start(&self) -> Option<WidgetKind> {
        match self.peek() {
            Some(TokKind::Button) => Some(WidgetKind::Button),
            Some(TokKind::Image) => Some(WidgetKind::Image),
            Some(TokKind::Text) => Some(WidgetKind::Text),
            _ => None
        }
    }

    fn parse_ui_props(&mut self) -> Result<Vec<UiProp>, ()> {
        let mut props = Vec::new();
        loop {
            if let Some(TokKind::ParamKey(key)) = self.peek() {
                let key = key.clone();
                self.bump();
                self.expect(TokKind::Equals)?;
                let val = self.str_or_ident()?;
                props.push(UiProp { key, val });
                continue;
            }

            if let Some(TokKind::Ident(key)) = self.peek() {
                // 向后看一位，如果是 =，则视为属性
                if let Some(TokKind::Equals) = self.peek_nth(1) {
                    let key = key.clone();
                    self.bump(); // eat key
                    self.bump(); // eat =
                    let val = self.str_or_ident()?;
                    props.push(UiProp { key, val });
                    continue;
                }
            }

            if let Some(TokKind::Reserved(k)) = self.peek() {
                if k == "at" {
                    self.bump();
                    let val = self.str_or_ident()?;
                    props.push(UiProp { key: "align".to_string(), val });
                    continue;
                }
            }

            break;
        }
        Ok(props)
    }
}
