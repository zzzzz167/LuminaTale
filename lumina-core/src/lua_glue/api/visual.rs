use mlua::{Lua, Table, Value};
use std::collections::HashMap;
use crate::event::{LayoutConfig, TransitionConfig};
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

    let cb_layout = cb.clone();
    table.set("register_layout", lua.create_function(move |_, (name, tbl): (String, Table)| {
        cb_layout.push(LuaCommand::RegisterLayout {
            name,
            config: LayoutConfig {
                x: tbl.get("x").unwrap_or(0.5),
                y: tbl.get("y").unwrap_or(1.0),
                anchor_x: tbl.get("anchor_x").unwrap_or(0.5),
                anchor_y: tbl.get("anchor_y").unwrap_or(1.0),
            }
        });
       Ok(())
    })?)?;

    let cb_trans = cb.clone();
    table.set("register_transition", lua.create_function(move |_, (name, tbl): (String, Table)| {
        let mut props_map = HashMap::new();
        let duration: f32 = tbl.get("duration").unwrap_or(1.0);
        let easing: String = tbl.get("easing").unwrap_or("linear".to_string());
        let mask_img: Option<String> = tbl.get("mask_img").ok();
        let vague: Option<f32> = tbl.get("vague").ok();

        if let Ok(props_table) = tbl.get::<Table>("props") {
            for pair in props_table.pairs::<String, mlua::Table>() {
                if let Ok((key, val_table)) = pair {
                    // 解析 from (可选)
                    let from_val: Option<f32> = val_table.get("from").ok();
                    // 解析 to (必须)
                    let to_val: f32 = val_table.get("to").unwrap_or(0.0);

                    props_map.insert(key, (from_val, to_val));
                }
            }
        }

        let config = TransitionConfig {
            duration,
            easing,
            mask_img,
            vague,
            props: props_map,
        };

        cb_trans.push(LuaCommand::RegisterTransition {
            name,
            config,
        });
       Ok(())
    })?)?;

    let cb_mark = cb.clone();
    table.set("mark_as_dynamic", lua.create_function(move |_, name: String| {
        cb_mark.push(LuaCommand::MarkDynamic { name });
        Ok(())
    })?)?;
    
    Ok(())
}