use viviscript_core::lexer::Lexer;
use viviscript_core::parser::Parser;
use lumina_core::{Ctx, TerminalRenderer};
use lumina_core::renderer::driver::Driver;
use std::fs;

fn main() {
    let s = fs::read_to_string("example-game/game/test.vivi").expect("Should not fail");
    let lexer = Lexer::new(&s).run();
    let ast = Parser::new(&lexer).parse();
    let mut ctx = Ctx::default();
    let renderer = TerminalRenderer;
    let mut driver = Driver::new(&mut ctx, &ast, renderer);
    driver.run(&mut ctx, &ast);
}