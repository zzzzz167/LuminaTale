use mlua::{Lua, Table, Value};
use std::collections::HashMap;
use crate::lua_glue::types::{CommandBuffer, LuaCommand};

pub fn register(lua: &Lua, table: &Table, cb: &CommandBuffer) -> mlua::Result<()> {
    let cb_transform = cb.clone();

    table.set("transform", lua.create_function(move |_, (target, props, duration, easing): (String, Table, Option<f32>, Option<String>)| {
        let mut props_map = HashMap::new();

        for pair in props.pairs::<String, Value>() {
            if let Ok((k, v)) = pair {
                if let Value::Number(n) = v {
                    props_map.insert(k, n as f32);
                } else if let Value::Integer(n) = v {
                    props_map.insert(k, n as f32);
                }
            }
        }

        cb_transform.push(LuaCommand::ModifyVisual {
            target,
            props: props_map,
            duration: duration.unwrap_or(0.0), // 默认 0 秒 (瞬移)
            easing: easing.unwrap_or_else(|| "linear".into()),
        });
        Ok(())
    })?)?;
    Ok(())
}