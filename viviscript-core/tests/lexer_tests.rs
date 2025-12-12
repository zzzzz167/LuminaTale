#[cfg(test)]
mod tests {
    use viviscript_core::lexer::Lexer;
    use viviscript_core::lexer::TokKind;

    fn lex(src: &str) -> Vec<TokKind> {
        let mut lexer = Lexer::new(src);

        lexer.run()
            .into_iter()
            .filter(|t| !matches!(t.tok, TokKind::Eof))
            .map(|x| x.tok)
            .collect()
    }

    fn assert_lex(src: &str, expect: Vec<TokKind>) {
        let got = lex(src);
        assert_eq!(got, expect, 
                   "\nsource:\n{}\nexpected: {:?}\ngot:      {:?}",
                   src, expect, got)
        ;
    }

    #[test]
    fn keywords_and_idents() {
        assert_lex(
            r#"character scene show hide play stop label jump call"#,
            vec![
                TokKind::Character,
                TokKind::Scene,
                TokKind::Show,
                TokKind::Hide,
                TokKind::Play,
                TokKind::Stop,
                TokKind::Label,
                TokKind::Jump,
                TokKind::Call,
            ],
        )
    }
    
    #[test]
    fn string_literals() {
        assert_lex(
            r#""hello" 'world' """triple""" "#,
            vec![
                TokKind::Str("hello".into()),
                TokKind::Str("world".into()),
                TokKind::Str("triple".into()),
            ],
        );
    }

    #[test]
    fn number_or_ident() {
        assert_lex(
            "42 3.14 2e10 2e-3 0xff bad42",
            vec![
                TokKind::Num(42.0),
                TokKind::Num(3.14),
                TokKind::Num(2e10),
                TokKind::Num(2e-3),
                TokKind::Ident("0xff".into()),
                TokKind::Ident("bad42".into()),
            ],
        );
    }

    #[test]
    fn comments() {
        assert_lex(
            "-- this is a comment\n42",
            vec![TokKind::Comment(" this is a comment".into()), TokKind::Newline, TokKind::Num(42.0)],
        );
    }

    #[test]
    fn lua_block() {
        let src = r#"
lua
    print("hello")
enlua
"#;
        let mut lexer = Lexer::new(src);
        let toks = lexer.run();
        assert!(matches!(toks[0].tok, TokKind::Lua));
        assert!(matches!(toks[1].tok, TokKind::LuaBlock(ref s) if s.trim() == r#"print("hello")"#));
    }

    #[test]
    fn choice_block() {
        let src = r#"
choice
    "Yes":jump good_end
    "No":call bad_end
enco"#;
        let got = lex(src);
        let expected = vec![
            TokKind::Choice,
            TokKind::Newline,
            TokKind::Str("Yes".into()),
            TokKind::Colon,
            TokKind::Jump,
            TokKind::Ident("good_end".into()),
            TokKind::Newline,
            TokKind::Str("No".into()),
            TokKind::Colon,
            TokKind::Call,
            TokKind::Ident("bad_end".into()),
            TokKind::Newline,
            TokKind::EnChoice,
        ];
        assert_eq!(got, expected);
    }

    #[test]
    fn colon_line() {
        assert_lex(
            ": This is dialogue\n",
            vec![TokKind::Colon, TokKind::Str("This is dialogue".into()), TokKind::Newline],
        );
    }

    #[test]
    fn dollar_lua_line() {
        assert_lex(
            "$ x = 1 + 2\n",
            vec![
                TokKind::Dollar,
                TokKind::LuaBlock("x = 1 + 2".into()),
                TokKind::Newline,
            ],
        );
    }

    #[test]
    fn mixed_whitespace() {
        assert_lex(
            "  scene   \t show  \n  hide",
            vec![TokKind::Scene, TokKind::Show, TokKind::Newline, TokKind::Hide],
        );
    }

    #[test]
    fn unexpected_char() {
        // 仅检查能解析的部分；异常字符会留下 warning，但 token 流仍继续。
        assert_lex("scene ^ hide", vec![TokKind::Scene, TokKind::Hide]);
    }
}