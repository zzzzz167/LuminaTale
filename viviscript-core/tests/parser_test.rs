use viviscript_core::lexer::Lexer;
use viviscript_core::parser::Parser;
use viviscript_core::ast::{ContainerKind, Stmt, UiStmt};

fn parse_code(input: &str) -> Result<viviscript_core::ast::Script, Vec<viviscript_core::parser::ParseError>> {
    let tokens = Lexer::new(input).run();
    Parser::new(&tokens).parse()
}

#[test]
fn test_basic_ui_screen() {
    let input = r#"
screen main_menu
    vbox
        text "Title" size=30
        button "Start" action="jump start"
    envbox
enscreen
"#;

    let script = parse_code(input).unwrap_or_else(|errs| {
        panic!("Parse failed: {:#?}", errs);
    });

    if let Stmt::ScreenDef { id, root, .. } = &script.body[0] {
        assert_eq!(id, "main_menu");
        assert_eq!(root.len(), 1);

        if let UiStmt::Container { kind, children, .. } = &root[0] {
            assert_eq!(*kind, ContainerKind::VBox);
            assert_eq!(children.len(), 2);
        } else {
            panic!("Root element should be a VBox");
        }
    } else {
        panic!("First statement should be ScreenDef");
    }
}

#[test]
fn test_zbox_and_overlay() {
    let input = r#"
screen hud
    zbox
        image "bar_bg"
        image "bar_fill" width=100 align="left"
    enzbox
enscreen
"#;

    let script = parse_code(input).unwrap_or_else(|errs| {
        panic!("Parse failed: {:#?}", errs);
    });

    let root = match &script.body[0] {
        Stmt::ScreenDef { root, .. } => &root[0],
        _ => panic!(),
    };

    match root {
        UiStmt::Container { kind, .. } => assert_eq!(*kind, ContainerKind::ZBox),
        _ => panic!("Expected ZBox"),
    }
}

#[test]
fn test_keyword_clash_fix() {
    let input = r#"
scene bg_school with fade_in
play music "bgm" fade_in=2.0 loop
"#;
    let res = parse_code(input);
    if let Err(e) = &res {
        println!("Keyword clash error: {:?}", e);
    }
    assert!(res.is_ok(), "Failed to parse keywords");
}

#[test]
fn test_error_recovery() {
    let input = r#"
label start
    :"Line 1"
    UNKNOWN_COMMAND_ERROR !!!
    :"Line 2"
enlb
"#;
    let res = parse_code(input);
    assert!(res.is_err(), "Should return error");
    let errs = res.unwrap_err();
    assert_eq!(errs.len(), 1);
    assert!(errs[0].line >= 4);
}

#[test]
fn test_complex_nested_layout() {
    let input = r#"
screen settings
    frame at center
        vbox
            text "Volume"
            hbox
                button "-"
                text "50"
                button "+"
            enhbox
            button "Back" action="return"
        envbox
    enframe
enscreen
"#;
    parse_code(input).unwrap_or_else(|errs| {
        panic!("Parse failed: {:#?}", errs);
    });
}