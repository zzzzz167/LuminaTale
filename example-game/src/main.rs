use viviscript_core::lexer::Lexer;

fn main() {
    let s = r#"
    
    """
    hello world!
    This is rust galgame!
    """
    @=$
    "@123"
    1.0.1
    "#;
    let lexer = Lexer::new(s).run();
    println!("{:?}",lexer)
}