use mlua::{Lua, Table};

pub fn register(lua: &Lua, rust_log: &Table) -> mlua::Result<()> {
    rust_log.set("info", lua.create_function(|_, msg: String| {
        log::info!("{}", msg);
        Ok(())
    })?)?;

    rust_log.set("warn", lua.create_function(|_, msg: String| {
        log::warn!("{}", msg);
        Ok(())
    })?)?;

    rust_log.set("error", lua.create_function(|_, msg: String| {
        log::error!("{}", msg);
        Ok(())
    })?)?;

    rust_log.set("debug", lua.create_function(|_, msg: String| {
        log::debug!("{}", msg);
        Ok(())
    })?)?;

    Ok(())
}