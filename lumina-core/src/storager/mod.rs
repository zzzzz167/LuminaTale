pub mod types;

use std::fs::File;
use std::io::{BufWriter, BufReader};
use viviscript_core::ast::Script;
use crate::{Ctx, Executor};
use crate::storager::types::SaveFile;

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

pub fn load(path: &str, script: &mut Script) -> anyhow::Result<(Ctx, Executor)> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let config = bincode::config::standard();
    let save: SaveFile = bincode::serde::decode_from_std_read(&mut reader, config)?;
    let mut exe = Executor::new();

    let mut dummy_ctx = Ctx::default();
    
    exe.preload_script(&mut dummy_ctx, script);
    exe.restore(save.stack);
    Ok((save.ctx, exe))
}