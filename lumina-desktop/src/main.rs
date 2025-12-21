mod setup;
mod config_gen;

use std::{env, fs};
use std::path::Path;
use std::sync::Arc;
use lumina_shared;
use viviscript_core::{lexer::Lexer, parser::Parser};
use lumina_core::config::CoreConfig;
use lumina_core::Executor;

fn main() {
    let args: Vec<String> = env::args().collect();
    let arg_tui = args.iter().any(|a| a == "--tui");

    let is_tui_mode = if cfg!(feature = "tui") {
        if cfg!(feature = "skia") {
            arg_tui
        } else {
            true
        }
    } else {
        false
    };

    setup::init(is_tui_mode);
    log::info!(">>> Lumina Desktop Launcher Started (TUI: {}) <<<", is_tui_mode);

    let core_cfg: CoreConfig = lumina_shared::config::get("core");
    let script_path = &core_cfg.script_path;

    log::info!("Loading script from config: {}", script_path);

    if !Path::new(script_path).exists() {
        log::error!("Script file not found: {}", script_path);
        panic!("Script '{}' not found. Please check config.toml or file path.", script_path);
    }

    let source = fs::read_to_string(script_path).unwrap();
    log::debug!("Loaded script: {} bytes", source.len());

    let tokens = Lexer::new(&source).run();
    log::debug!("Lexing complete: {} tokens", tokens.len());

    let mut ast = Parser::new(&tokens).parse();
    if core_cfg.debug.show_ast {
        log::debug!("AST: {:#?}", ast);
    }
    Executor::prepare_script(&mut ast);
    let script_arc = Arc::new(ast);

    log::info!("Parsing complete");

    #[cfg(feature = "tui")]
    if is_tui_mode {
        log::info!("Mode: TUI (User Requested)");
        run_tui(script_arc);
        return;
    }

    #[cfg(feature = "skia")]
    {
        log::info!("Mode: Skia (Default)");
        run_skia(script_arc);
        return;
    }

    #[cfg(feature = "tui")]
    {
        log::info!("Mode: TUI (Fallback)");
        run_tui(script_arc);
        return;
    }

    #[cfg(not(any(feature = "skia", feature = "tui")))]
    {
        log::error!("No renderer features enabled! Compile with --features skia or --features tui");
    }
}

#[cfg(feature = "skia")]
fn run_skia(script: Arc<viviscript_core::ast::Script>) {
    use lumina_skia_renderer::SkiaRenderer;
    let app = SkiaRenderer::new(script);
    app.run();
}

#[cfg(feature = "tui")]
fn run_tui(script: Arc<viviscript_core::ast::Script>) {
    use lumina_core::{Ctx, TuiRenderer};
    use lumina_core::renderer::Renderer;

    let mut ctx = Ctx::default();

    match TuiRenderer::new() {
        Ok(mut renderer) => {
            renderer.run_event_loop(&mut ctx, script);
        }
        Err(e) => {
            log::error!("Failed to initialize TUI: {}", e);
        }
    }
}