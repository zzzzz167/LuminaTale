use viviscript_core::lexer::{Lexer, Tok};

#[test]
fn basics() {
    let src = r#"
character alice name="Alice" image_tag="alice_normal"
"#;
    let tokens = Lexer::new(src).run();
    assert_eq!(tokens[0], Tok::Newline)
}

#[test]
fn triple_quotes() {
    let src = r#""""hello\nworld""""#;
    let tokens = Lexer::new(src).run();
    assert_eq!(tokens[0], Tok::Str(String::from("hello\nworld")))
}

#[test]
fn escapes() {
    let s0 = Lexer::new(r#""line1\nline2""#).run();
    assert_eq!(s0[0], Tok::Str("line1\nline2".to_string()));

    let s2 = Lexer::new(r#"\"\\\""#).run();
    assert_eq!(s2[0], Tok::Str("\\".to_string()));
}