mod setup;
mod config_gen;

use std::{env};
use std::sync::Arc;
use lumina_shared;
use lumina_core::config::CoreConfig;
use lumina_core::ScriptManager;

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
    let project_root = &core_cfg.script_path;

    log::info!("Loading project from: {:?}", project_root);

    let mut manager = ScriptManager::new();
    if let Err(e) = manager.load_project(project_root) {
        log::error!("Failed to load project: {}", e);
        panic!("Project load failed");
    }

    let manager_arc = Arc::new(manager);

    log::info!("Project loaded successfully");

    #[cfg(feature = "tui")]
    if is_tui_mode {
        log::info!("Mode: TUI (User Requested)");
        run_tui(manager_arc);
        return;
    }

    #[cfg(feature = "skia")]
    {
        log::info!("Mode: Skia (Default)");
        run_skia(manager_arc);
        return;
    }

    #[cfg(feature = "tui")]
    {
        log::info!("Mode: TUI (Fallback)");
        run_tui(manager_arc);
        return;
    }

    #[cfg(not(any(feature = "skia", feature = "tui")))]
    {
        log::error!("No renderer features enabled! Compile with --features skia or --features tui");
    }
}

#[cfg(feature = "skia")]
fn run_skia(manager: Arc<ScriptManager>) {
    use lumina_skia_renderer::SkiaRenderer;
    let app = SkiaRenderer::new(manager);
    app.run();
}

#[cfg(feature = "tui")]
fn run_tui(manager: Arc<ScriptManager>) {
    use lumina_core::{Ctx, TuiRenderer};
    use lumina_core::renderer::Renderer;

    let mut ctx = Ctx::default();

    match TuiRenderer::new() {
        Ok(mut renderer) => {
            renderer.run_event_loop(&mut ctx, manager);
        }
        Err(e) => {
            log::error!("Failed to initialize TUI: {}", e);
        }
    }
}