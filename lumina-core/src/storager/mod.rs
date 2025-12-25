pub mod types;

use crate::storager::types::SaveFile;
use crate::{Ctx, Executor, ScriptManager};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::Arc;

pub fn save(path: &str, ctx: Ctx, exe: Executor) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let save = SaveFile {
        ctx: ctx.clone(),
        stack: exe.snapshot()
    };
    let config = bincode::config::standard();
    bincode::serde::encode_into_std_write(&save, &mut writer, config)?;
    Ok(())
}

pub fn load(path: &str, manager: Arc<ScriptManager>) -> anyhow::Result<(Ctx, Executor)> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let config = bincode::config::standard();
    let save: SaveFile = bincode::serde::decode_from_std_read(&mut reader, config)?;
    let mut exe = Executor::new(manager);

    exe.restore(save.stack);
    Ok((save.ctx, exe))
}