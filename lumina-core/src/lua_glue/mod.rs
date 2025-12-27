pub mod types;
pub mod api;

pub use types::{CommandBuffer, LuaCommand};

use mlua::{Lua, LuaSerdeExt, Table};
use log::{error, info};

pub fn init_lua(lua: &Lua) -> CommandBuffer {
    let cmd_buffer = CommandBuffer::new();

    let globals = lua.globals();

    if globals.get::<Table>("f").is_err() {
        let f_table = lua.create_table().unwrap();
        globals.set("f", f_table).unwrap();
    }

    if globals.get::<Table>("sf").is_err() {
        let sf_table = lua.create_table().unwrap();
        globals.set("sf", sf_table).unwrap();
    }

    globals.set("print", lua.create_function(|_, msg: String| {
        info!("[Lua] {}", msg);
        Ok(())
    }).unwrap()).unwrap();

    let lumina = lua.create_table().unwrap();

    api::system::register(lua, &lumina, &cmd_buffer).expect("Failed to register system API");
    api::audio::register(lua, &lumina, &cmd_buffer).expect("Failed to register audio API");

    globals.set("lumina", lumina).expect("Failed to register audio API");
    cmd_buffer
}

pub fn evel_bool(lua: &Lua, expr: &str) -> bool {
    let chunk = format!("return {}", expr);

    lua.load(&chunk).eval::<bool>().unwrap_or_else(|e| {
        error!("Lua eval error for condition '{}': {}", expr, e);
        false
    })
}

pub fn inject_vars(lua: &Lua, data: &serde_json::Value) {
    let globals = lua.globals();

    match lua.to_value(data) {
        Ok(lua_val) => {
            if lua_val.is_nil() {
                globals.set("f", lua.create_table().unwrap()).unwrap();
            } else {
                globals.set("f", lua_val).unwrap();
            }
        },
        Err(e) => error!("Failed to inject vars to Lua: {}", e),
    }
}

pub fn extract_vars(lua: &Lua) -> serde_json::Value {
    let globals = lua.globals();

    if let Ok(val) = globals.get::<mlua::Value>("f") {
        serde_json::to_value(&val).unwrap_or_else(|e| {
            error!("Failed to serialize Lua 'f' table: {}", e);
            serde_json::Value::Null
        })
    } else {
        serde_json::Value::Null
    }
}

pub fn eval_string(lua: &Lua, expr: &str) -> String {
    let chunk = format!("return tostring({})", expr);

    match lua.load(&chunk).eval::<String>() {
        Ok(s) => s,
        Err(e) => {
            log::error!("Interpolation error for '{}': {}", expr, e);
            format!("{{ERR:{}}}", expr)
        }
    }
}

pub fn inject_sf(lua: &Lua, data: &serde_json::Value) {
    let globals = lua.globals();
    match lua.to_value(data) {
        Ok(lua_val) => {
            if !lua_val.is_nil() {
                globals.set("sf", lua_val).unwrap();
            }
        }
        Err(e) => error!("Failed to inject sf to Lua: {}", e),
    }
}

pub fn extract_sf(lua: &Lua) -> serde_json::Value {
    let globals = lua.globals();
    if let Ok(val) = globals.get::<mlua::Value>("sf") {
        serde_json::to_value(&val).unwrap_or_else(|e| {
            error!("Failed to serialize Lua 'sf': {}", e);
            serde_json::Value::Null
        })
    } else {
        serde_json::Value::Null
    }
}