use viviscript_core::lexer::Lexer;
use viviscript_core::parser::Parser;
use lumina_core::Ctx;
use lumina_core::config;
use std::{fs::OpenOptions, fs, io::Write};
use env_logger::{Builder, Target};
use lumina_core::renderer::Renderer;
#[cfg(feature = "tui")]
use lumina_core::TuiRenderer;


fn init_logger() {
    // 1. 公共参数
    let level = config::get().debug.log_level.clone();

    // 2. 根据 feature 选择输出目标
    #[cfg(feature = "tui")]
    {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("lumina.log")
            .expect("Failed to open log file");

        Builder::from_env(env_logger::Env::default().default_filter_or(level))
            .target(Target::Pipe(Box::new(log_file)))
            .init();
    }

    #[cfg(feature = "skia")]
    {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)    // Skia 追加
            .open("lumina.log")
            .expect("Failed to open log file");

        // Tee：终端带颜色 + 文件纯文本
        struct TeeWriter<W1, W2>(W1, W2);
        impl<W1: Write, W2: Write> Write for TeeWriter<W1, W2> {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let n = self.0.write(buf)?;
                self.1.write_all(&buf[..n])?;
                Ok(n)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.0.flush()?;
                self.1.flush()?;
                Ok(())
            }
        }

        Builder::from_env(env_logger::Env::default().default_filter_or(level))
            .target(Target::Pipe(Box::new(TeeWriter(std::io::stdout(), log_file))))
            .init();
    }
}

fn main() {
    config::init_global("config.toml");
    
    init_logger();

    log::info!("Starting Lumina runtime");
    let s = fs::read_to_string("example-game/game/skia_renderer_test.vivi").expect("Should not fail");
    log::debug!("Loaded script: {} bytes", s.len());
    
    let lexer = Lexer::new(&s).run();
    log::debug!("Lexing complete: {} tokens", lexer.len());
    
    let ast = Parser::new(&lexer).parse();
    
    if config::get().debug.show_ast {
        log::debug!("AST: {:#?}", ast);
    }
    
    log::info!("Parsing complete");


    #[cfg(feature = "tui")] {
        let mut ctx = Ctx::default();
        let mut renderer = TuiRenderer::new().expect("init TUI");
        renderer.run_event_loop(&mut ctx, ast);
    }

    #[cfg(feature = "skia")] {
        let app = lumina_skia_renderer::renderer::SkiaRenderer::new(ast);
        app.run();
    }

    log::info!("Game finished");
}