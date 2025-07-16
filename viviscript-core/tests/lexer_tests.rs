use viviscript_core::lexer::{Lexer, Tok};

#[test]
fn basics() {
    let src = r#"
character alice name="Alice" image_tag="alice_normal"
"#;
    let tokens = Lexer::new(src).run();
    assert_eq!(tokens, 
               vec![Tok::Character, 
                    Tok::Ident("alice".to_string()), 
                    Tok::ParamKey("name".to_string()), 
                    Tok::Equals, 
                    Tok::Str("Alice".to_string()), 
                    Tok::ParamKey("image_tag".to_string()),
                    Tok::Equals,
                    Tok::Str("alice_normal".to_string()),
                    Tok::Newline, 
                    Tok::Eof])
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

    let s2 = Lexer::new(r#""\\\"""#).run();
    assert_eq!(s2[0], Tok::Str("\\\"".to_string()));
}

#[test]
fn lua_block() {
    let src1 = r#"
lua
print("test")
enlua"#;
    let toks1 = Lexer::new(src1).run();
    assert_eq!(toks1, vec![Tok::Lua,Tok::LuaBlock("\nprint(\"test\")".to_string()),Tok::EnLua,Tok::Eof]);
    
    let src2 = r#"lua print("test") enlua"#;
    let toks2 = Lexer::new(src2).run();
    assert_eq!(toks2, vec![Tok::Lua,Tok::LuaBlock(" print(\"test\")".to_string()),Tok::EnLua,Tok::Eof]);
}

#[test]
fn test_keywords() {
    let source = "character scene show hide play stop label enlb jump call";
    let tokens = Lexer::new(source).run();
    assert_eq!(
        tokens,
        vec![
            Tok::Character,
            Tok::Scene,
            Tok::Show,
            Tok::Hide,
            Tok::Play,
            Tok::Stop,
            Tok::Label,
            Tok::EnLabel,
            Tok::Jump,
            Tok::Call,
            Tok::Eof,
        ]
    );
}

#[test]
fn test_identifiers() {
    let source = "some_identifier another_ident_123";
    let tokens = Lexer::new(source).run();
    assert_eq!(
        tokens,
        vec![
            Tok::Ident("some_identifier".to_string()),
            Tok::Ident("another_ident_123".to_string()),
            Tok::Eof,
        ]
    );
}

#[test]
fn test_strings() {
    let source = r#""simple string" 'single-quoted string' :a colon string
:"""multi-line
string test""" 
:another_colon_string
"#;
    let tokens = Lexer::new(source).run();
    assert_eq!(
        tokens,
        vec![
            Tok::Str("simple string".to_string()),
            Tok::Str("single-quoted string".to_string()),
            Tok::Colon,
            Tok::Str("a colon string".to_string()),
            Tok::Newline,
            Tok::Colon,
            Tok::Str("multi-line\nstring test".to_string()),
            Tok::Newline,
            Tok::Colon,
            Tok::Str("another_colon_string".to_string()),
            Tok::Newline,
            Tok::Eof,
        ]
    );
}

#[test]
fn test_comments() {
    let source = r#"-- This is a comment
valid_code -- This is after code
"#;
    let tokens = Lexer::new(source).run();
    assert_eq!(
        tokens,
        vec![
            Tok::Comment(" This is a comment".to_string()),
            Tok::Newline,
            Tok::Ident("valid_code".to_string()),
            Tok::Comment(" This is after code".to_string()),
            Tok::Newline,
            Tok::Eof,
        ]
    );
}

#[test]
fn test_numbers() {
    let source = "123 45.67 1.2e10 1invalid1";
    let tokens = Lexer::new(source).run();
    assert_eq!(
        tokens,
        vec![
            Tok::Num(123.0),
            Tok::Num(45.67),
            Tok::Num(1.2e10),
            Tok::Ident("1invalid1".to_string()),
            Tok::Eof,
        ]
    );
}

#[test]
fn test_choice_block() {
    let source = r#"
choice Test
Option 1:jump label1
Option 2:lua print("hello") enlua
enco
"#;
    let tokens = Lexer::new(source).run();
    assert_eq!(
        tokens,
        vec![
            Tok::Choice,
            Tok::Str("Test".to_string()),
            Tok::Newline,
            Tok::Str("Option 1".to_string()),
            Tok::Colon,
            Tok::Jump,
            Tok::Ident("label1".to_string()),
            Tok::Newline,
            Tok::Str("Option 2".to_string()),
            Tok::Colon,
            Tok::Lua,
            Tok::LuaBlock(" print(\"hello\")".to_string()),
            Tok::EnLua,
            Tok::Newline,
            Tok::EnChoice,
            Tok::Newline,
            Tok::Eof,
        ]
    );
}

#[test]
fn test_scene_show_hide() {
    let source = r#"scene bg_forest with fade_in
show character1 at left
hide character1
"#;
    let tokens = Lexer::new(source).run();
    assert_eq!(
        tokens,
        vec![
            Tok::Scene,
            Tok::Ident("bg_forest".to_string()),
            Tok::ParamKey("with".to_string()),
            Tok::ParamKey("fade_in".to_string()),
            Tok::Newline,
            Tok::Show,
            Tok::Ident("character1".to_string()),
            Tok::ParamKey("at".to_string()),
            Tok::Ident("left".to_string()),
            Tok::Newline,
            Tok::Hide,
            Tok::Ident("character1".to_string()),
            Tok::Newline,
            Tok::Eof,
        ]
    );
}