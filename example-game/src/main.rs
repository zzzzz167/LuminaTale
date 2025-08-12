use viviscript_core::lexer::Lexer;
use viviscript_core::parser::Parser;
use lumina_core::{Ctx, TuiRenderer}; 
use lumina_core::renderer::driver::Driver;
use lumina_core::config;
use std::fs;
use std::fs::File;
use env_logger::Target;

fn main() {
    config::init_global("config.toml");
    
    let log_file = File::create("lumina.log").expect("Failed to create log file");
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(config::get().debug.log_level.clone()))
        .target(Target::Pipe(Box::new(log_file)))
        .init();
    
    log::info!("Starting Lumina runtime");
    let s = fs::read_to_string("example-game/game/test.vivi").expect("Should not fail");
    log::debug!("Loaded script: {} bytes", s.len());
    
    let lexer = Lexer::new(&s).run();
    log::debug!("Lexing complete: {} tokens", lexer.len());
    
    let mut ast = Parser::new(&lexer).parse();
    
    if config::get().debug.show_ast {
        log::debug!("AST: {:#?}", ast);
    }
    
    log::info!("Parsing complete");
    
    let mut ctx = Ctx::default();
    let renderer = TuiRenderer::new().unwrap();
    let mut driver = Driver::new(&mut ctx, ast, renderer);
    driver.run(&mut ctx);
    log::info!("Game finished");
}