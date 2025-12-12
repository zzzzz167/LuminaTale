use mlua::{Lua, FromLua, Value, Table};
use std::sync::{Arc, Mutex};
use log::{error, info};

#[derive(Debug,Clone)]
pub enum LuaCommand {
    Jump(String),
}

#[derive(Debug,Clone)]
pub struct CommandBuffer {
    queue: Arc<Mutex<Vec<LuaCommand>>>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push(&self, cmd: LuaCommand) {
        if let Ok(mut q) = self.queue.lock() {
            q.push(cmd);
        }
    }

    pub fn drain(&self) -> Vec<LuaCommand> {
        if let Ok(mut q) = self.queue.lock() {
            std::mem::take(&mut q)
        } else {
            vec![]
        }
    }
}

pub fn init_lua(lua: &Lua) -> CommandBuffer {
    let cmd_buffer = CommandBuffer::new();
    let cb_clone = cmd_buffer.clone();

    let globals = lua.globals();
    let lumina = lua.create_table().unwrap();

    let cb = cb_clone.clone();
    lumina.set("jump", lua.create_function(move |_, target: String| {
        cb.push(LuaCommand::Jump(target));
        Ok(())
    }).unwrap()).unwrap();

    globals.set("print", lua.create_function(|_, msg: String| {
        info!("[Lua] {}", msg);
        Ok(())
    }).unwrap()).unwrap();

    globals.set("lumina", lumina).unwrap();
    cmd_buffer
}

pub fn evel_bool(lua: &Lua, expr: &str) -> bool {
    let chunk = format!("return {}", expr);

    lua.load(&chunk).eval::<bool>().unwrap_or_else(|e| {
        error!("Lua eval error for condition '{}': {}", expr, e);
        false
    })
}