use crate::config_gen;

use std::fs::OpenOptions;
use std::io::Write;
use env_logger::{Builder, Target};
use lumina_core::config::CoreConfig;

pub fn init(is_tui: bool) {
    let config_path = "config.toml";

    config_gen::ensure_config_exists(config_path);

    if let Err(e) = lumina_shared::config::init(config_path) {
        eprintln!("Config load warning: {}", e);
    }

    init_logger(is_tui);
}

fn init_logger(is_tui: bool) {
    let core_cfg: CoreConfig = lumina_shared::config::get("core");
    let level = &core_cfg.debug.log_level;

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("lumina.log")
        .expect("Failed to open log file");

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

    let mut builder = Builder::from_env(env_logger::Env::default().default_filter_or(level));
    if is_tui {
        builder.target(Target::Pipe(Box::new(log_file)));
    } else {
        builder.target(Target::Pipe(Box::new(TeeWriter(std::io::stdout(), log_file))));
    }

    builder.init();
}