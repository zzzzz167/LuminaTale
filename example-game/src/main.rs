use viviscript_core::lexer::Lexer;
use viviscript_core::parser::Parser;
use lumina_core::Ctx;
use lumina_core::config;
use std::fs;
use std::fs::File;
use env_logger::Target;
use lumina_core::renderer::driver::Driver;
#[cfg(feature = "tui")]
use lumina_core::TuiRenderer;

#[cfg(feature = "skia")]
use lumina_skia_renderer::SkiaRenderer;

fn main() {
    config::init_global("config.toml");
    
    #[cfg(feature = "tui")]{
        let log_file = File::create("lumina.log").expect("Failed to create log file");
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(config::get().debug.log_level.clone()))
            .target(Target::Pipe(Box::new(log_file)))
            .init();
    }

    #[cfg(feature = "skia")]
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(config::get().debug.log_level.clone()))
        .init();
    
    log::info!("Starting Lumina runtime");
    let s = fs::read_to_string("example-game/game/test.vivi").expect("Should not fail");
    log::debug!("Loaded script: {} bytes", s.len());
    
    let lexer = Lexer::new(&s).run();
    log::debug!("Lexing complete: {} tokens", lexer.len());
    
    let ast = Parser::new(&lexer).parse();
    
    if config::get().debug.show_ast {
        log::debug!("AST: {:#?}", ast);
    }
    
    log::info!("Parsing complete");
    
    let mut ctx = Ctx::default();

    #[cfg(feature = "tui")] {
        let renderer = TuiRenderer::new().unwrap();
        let mut driver = Driver::new(&mut ctx, ast, renderer);
        driver.run(&mut ctx);
    }

    #[cfg(feature = "skia")] {
        let event_loop = SkiaRenderer::new();
        let mut app = SkiaRenderer::default();
        
        event_loop.run_app(&mut app).unwrap();
    }

    log::info!("Game finished");
}