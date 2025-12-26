pub mod types;

use crate::storager::types::{GlobalSave, SaveFile};
use crate::{Ctx, Executor, ScriptManager};
use crate::config::SystemConfig;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn get_save_path(filename: &str) -> PathBuf {
    let cfg: SystemConfig = lumina_shared::config::get("system");
    let dir = Path::new(&cfg.save_path);

    if !dir.exists() {
        let _ = fs::create_dir_all(dir);
    }

    dir.join(filename)
}

pub fn save(filename: &str, ctx: Ctx, exe: Executor) -> anyhow::Result<()> {
    let full_path = get_save_path(filename);

    let file = File::create(full_path)?;
    let mut writer = BufWriter::new(file);
    let save = SaveFile {
        ctx: ctx.clone(),
        stack: exe.snapshot()
    };
    let config = bincode::config::standard();
    bincode::serde::encode_into_std_write(&save, &mut writer, config)?;
    Ok(())
}

pub fn load(filename: &str, manager: Arc<ScriptManager>) -> anyhow::Result<(Ctx, Executor)> {
    let full_path = get_save_path(filename);
    let file = File::open(full_path)?;
    let mut reader = BufReader::new(file);
    let config = bincode::config::standard();
    let save: SaveFile = bincode::serde::decode_from_std_read(&mut reader, config)?;
    let mut exe = Executor::new(manager);

    exe.restore(save.stack);
    Ok((save.ctx, exe))
}

pub fn save_global(filename: &str, data: &serde_json::Value) -> anyhow::Result<()> {
    let full_path = get_save_path(filename);
    let file = File::create(full_path)?;
    let mut writer = BufWriter::new(file);

    let save = GlobalSave { sf: data.clone() };

    serde_json::to_writer_pretty(&mut writer, &save)?;
    writer.flush()?;

    Ok(())
}

pub fn load_global(filename: &str) -> anyhow::Result<serde_json::Value> {
    let full_path = get_save_path(filename);

    if !full_path.exists() {
        return Ok(serde_json::Value::Null);
    }

    let file = File::open(full_path)?;
    let reader = BufReader::new(file);

    let save: GlobalSave = serde_json::from_reader(reader)?;
    Ok(save.sf)
}