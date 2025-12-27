use mlua::{Lua, Table};
use crate::lua_glue::types::{CommandBuffer, LuaCommand};

pub fn register(lua: &Lua, table: &Table, cb: &CommandBuffer) -> mlua::Result<()> {
    // 1. Jump
    let cb_jump = cb.clone();
    table.set("jump", lua.create_function(move |_, target: String| {
        cb_jump.push(LuaCommand::Jump(target));
        Ok(())
    })?)?;

    // 2. Save Global
    let cb_save = cb.clone();
    table.set("save_global", lua.create_function(move |_, ()| {
        cb_save.push(LuaCommand::SaveGlobal);
        Ok(())
    })?)?;

    Ok(())
}