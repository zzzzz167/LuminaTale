use crate::ast::{AudioAction, AudioChannel, AudioOptions, ChoiceArm, SceneImage, Script, ShowAttr, Speaker, Stmt, Transition};
use crate::lexer::{Span, Tok, TokKind};
use regex::Regex;

#[derive(PartialEq)]
enum Status {
    Run,
    Stop,
}

pub struct Parser<'a> {
    toks: &'a [Tok],
    cursor: usize,
    status: Status,
}

impl<'a> Parser<'a> {
    pub fn new(toks: &'a [Tok]) -> Self {
        Self {
            toks,
            cursor: 0,
            status: Status::Run,
        }
    }

    fn peek(&self) -> Option<&TokKind> {
        self.toks.get(self.cursor).map(|t| &t.tok)
    }

    fn bump(&mut self) -> &'a Tok {
        let tok = &self.toks[self.cursor];
        self.cursor += 1;
        tok
    }

    fn span(&self) -> Span {
        self.toks[self.cursor].span
    }

    fn at(&self, k: TokKind) -> bool {
        self.peek()
            .map(|tk| std::mem::discriminant(tk) == std::mem::discriminant(&k))
            .unwrap_or(false)
    }

    fn expect(&mut self, expect: TokKind) -> &'a Tok {
        let tok = self.bump();
        assert_eq!(tok.tok, expect, "expected {:?}, got {:?} at {:?}", expect, tok.tok, tok.span);
        tok
    }

    fn expect_any<I>(&mut self, token: I) -> &'a Tok
    where
        I: IntoIterator<Item = TokKind>,
    {
        let mut v = Vec::new();
        let mut matched = false;
        v.extend(token);
        if let Some(next) = self.peek(){

            for i in v.iter().cloned() {
                if i == *next {
                    matched = true;
                    break
                }
            }
        }
        let tok = self.bump();
        if !matched {
            panic!("expect {:?}, but got {:?}", v, tok.tok);
        } else {
            tok
        }
    }

    fn consume(&mut self, k: TokKind) -> bool {
        if self.peek() == Some(&k) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn ident(&mut self) -> String {
        match &self.bump().tok {
            TokKind::Ident(s) => String::from(s),
            x => panic!("expected Ident, got {:?}", x),
        }
    }

    fn string(&mut self) -> String {
        match &self.bump().tok {
            TokKind::Str(s) => String::from(s),
            x => panic!("expected String, got {:?}", x),
        }
    }

    fn num(&mut self) -> f64 {
        match &self.bump().tok {
            TokKind::Num(n) => *n,
            x => panic!("expected Num, got {:?}", x),
        }
    }

    fn str_or_ident(&mut self) -> String {
        match self.peek() {
            Some(TokKind::Str(_)) => self.string(),
            Some(TokKind::Ident(_)) => self.ident(),
            _ => panic!("expected Str or Ident, got {:?}", self.peek()),
        }
    }

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

    pub fn parse(mut self) -> Script {
        let mut body = Vec::new();
        while self.peek().is_some() && self.status == Status::Run {
            match self.stmt() {
                Some(s) => body.push(s),
                None => {}
            }
        }
        Script { body }
    }

    fn stmt(&mut self) -> Option<Stmt> {
        match self.peek() {
            Some(TokKind::Character) => Some(self.character()),
            Some(TokKind::Label) => Some(self.label()),
            Some(TokKind::Choice) => Some(self.choice()),
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
                let span = self.span();
                Some(Stmt::Error {
                    span,
                    msg: format!("Undo statement, got {:?}", self.bump()),
                })
            }
        }
    }

    fn label(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Label);
        let id = self.ident();
        let mut body = Vec::new();
        while !matches!(self.peek(), Some(TokKind::EnLabel) | None) {
            if self.at(TokKind::Eof) {panic!("Unexpected EOF")}
            match self.stmt() {
                Some(s) => body.push(s),
                None => {}
            }
        }
        self.bump();
        Stmt::Label { span, id, body }
    }

    fn jump(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Jump);
        let target = self.ident();
        Stmt::Jump { span, target }
    }

    fn call(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Call);
        let target = self.ident();
        Stmt::Call { span, target }
    }

    fn choice(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Choice);
        let mut title = None;
        if self.at(TokKind::Str("".into())) {
            title = Some(self.string());
        }

        let mut arms = Vec::new();

        while !self.consume(TokKind::EnChoice) {
            self.skip_trivia();
            let mut body = Vec::new();
            let text = self.string();
            self.expect(TokKind::Colon);
            match self.stmt() {
                Some(s) => body.push(s),
                None => {}
            }

            if body.is_empty() {
                panic!("empty body");
            }

            arms.push(ChoiceArm { text, body });
            self.skip_trivia();
        }
        Stmt::Choice { span, title, arms }
    }

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
                _ => panic!("Not a available paramKey {}", key),
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

    fn dialogue(&mut self) -> Stmt {
        let span = self.span();
        let name = self.ident();
        let mut alias = None;
        if self.at(TokKind::At) {
            self.consume(TokKind::At);
            alias = Some(self.str_or_ident());
        }

        self.expect(TokKind::Colon);
        let str = self.string();

        let mut voice_index = None;
        let mut text = String::new();
        let re = Regex::new(r"\(([^()]*)\)$").unwrap();
        if let Some(caps) = re.captures(&str) {
            voice_index = Some(caps.get(1).unwrap().as_str().to_string());
            text.push_str(&*re.replace(&str, ""));
        } else {
            text += &str;
        }

        Stmt::Dialogue {
            span,
            speaker: Speaker { name, alias },
            text,
            voice_index,
        }
    }

    fn narration(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Colon);
        let mut lines = Vec::new();
        for i in self.string().trim().lines() {
            lines.push(i.to_string());
        }
        Stmt::Narration { span, lines }
    }

    fn luablock(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Lua);
        if !self.at(TokKind::LuaBlock("".into())) {
            panic!("expected lua block, but got {:?}", self.bump());
        }
        let code = self.bump().tok.as_str().unwrap().to_string();
        self.skip_trivia();
        self.expect(TokKind::EnLua);

        Stmt::LuaBlock {span, code}
    }
    
    fn dollar_luablock(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Dollar);
        if !self.at(TokKind::LuaBlock("".into())) {
            panic!("expected lua block, but got {:?}", self.bump());
        }
        let code = self.bump().tok.as_str().unwrap().to_string();

        Stmt::LuaBlock {span, code}

    }

    fn play_audio(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Play);
        let action = AudioAction::Play;
        let mut r#loop = false;
        let channel = match self.str_or_ident().as_str() {
            "music" => {
                r#loop = true;
                AudioChannel::Music
            }
            "sound" => AudioChannel::Sound,
            "voice" => AudioChannel::Voice,
            _ => panic!("No channel named {}", self.ident()),
        };
        let resource = Some(self.str_or_ident());

        let mut volume = None;
        let mut fade_in = None;
        let mut have_a_loop = false;
        while let Some(TokKind::ParamKey(k) | TokKind::Flag(k)) = self.peek() {
            let key = k.clone();
            if self.at(TokKind::Flag("".into())) {
                self.bump();
                if have_a_loop {
                    panic!("Already have define on 'loop' keyword");
                }
                match key.as_str() {
                    "loop" => r#loop = true,
                    "noloop" => r#loop = false,
                    _ => panic!("Not available flag named {}", key),
                }
                have_a_loop = true;
            } else {
                self.bump();
                self.expect(TokKind::Equals);
                let val = self.num() as f32;
                match key.as_str() {
                    "volume" => volume = Some(val),
                    "fade_in" => fade_in = Some(val),
                    _ => panic!("Not available paramKey named {}", key),
                }
            }
        }

        let options = AudioOptions {
            volume,
            fade_in,
            r#loop,
            fade_out: None,
        };
        Stmt::Audio {
            span,
            action,
            channel,
            resource,
            options,
        }
    }

    fn stop_audio(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Stop);
        let action = AudioAction::Stop;
        let channel = match self.str_or_ident().as_str() {
            "music" => AudioChannel::Music,
            "sound" => AudioChannel::Sound,
            "voice" => AudioChannel::Voice,
            _ => panic!("No channel named {}", self.ident()),
        };
        let mut fade_out = None;
        while let Some(TokKind::ParamKey(k)) = self.peek() {
            let key = k.clone();
            self.bump();
            self.expect(TokKind::Equals);
            let val = self.num() as f32;
            match key.as_str() {
                "fade_out" => fade_out = Some(val),
                _ => panic!("Not available paramKey named {}", key),
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
                    panic!("Invalid form");
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
                        panic!("expected Newline or Eof");
                    }
                } else {
                    panic!("Not available reserved keyword {}", k);
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

    fn hide(&mut self) -> Stmt {
        let span = self.span();
        self.expect(TokKind::Hide);
        let target = self.str_or_ident();
        if !self.at(TokKind::Comment("".into())) {
            self.expect_any([TokKind::Eof,TokKind::Newline]);
        }
        Stmt::Hide {span, target}
    }
}
