use mlua::{Lua, Table};
use crate::lua_glue::types::{CommandBuffer, LuaCommand};

pub fn register(lua: &Lua, table: &Table, cb: &CommandBuffer) -> mlua::Result<()> {
    let cb_vol = cb.clone();
    table.set("set_volume", lua.create_function(move |_, (channel, val): (String, f32)| {
        cb_vol.push(LuaCommand::SetVolume { channel, value: val });
        Ok(())
    })?)?;

    Ok(())
}