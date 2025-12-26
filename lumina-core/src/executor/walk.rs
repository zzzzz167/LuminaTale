use std::ops::Add;
use std::sync::OnceLock;
use viviscript_core::ast::{Stmt, AudioAction, ShowAttr, Transition};
use regex::Regex;
use mlua::Lua;
use lumina_shared::config;
use crate::runtime::Ctx;
use crate::event::OutputEvent;
use crate::runtime::assets::{Audio, DialogueRecord, Sprite};
use crate::lua_glue;
use crate::config::{AudioConfig, GraphicsConfig};

#[derive(Debug, Clone)]
pub struct StmtEffect {
    pub events: Vec<OutputEvent>,
    pub next: NextAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NextAction {
    Continue,
    Jump(String),
    Call(String),
    WaitChoice(Vec<(String, Vec<Stmt>)>),
    WaitInput,
    EnterBlock(String, Vec<Stmt>),
}

fn interpolate(lua: &Lua, text: &str) -> String {
    // 缓存正则表达式，避免重复编译
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\{([^}]+)\}").unwrap());

    re.replace_all(text, |caps: &regex::Captures| {
        let expr = &caps[1]; // 拿到花括号里面的内容，例如 "f.score"
        lua_glue::eval_string(lua, expr)
    }).to_string()
}

pub fn walk_stmt(ctx: &mut Ctx, lua: &Lua, stmt: &Stmt) -> StmtEffect {
    log::trace!("walk_stmt: {:?}", stmt);

    let audio_cfg: AudioConfig = config::get("audio"); // ✅ 按需获取
    let gfx_cfg: GraphicsConfig = config::get("graphics");

    let mut events = Vec::new();
    let next = match stmt {
        Stmt::CharacterDef{id,name,image_tag,voice_tag,..} => {
            let cd = crate::runtime::Character {
                id: id.clone(),
                name: name.clone(),
                voice_tag: voice_tag.clone(),
                image_tag: image_tag.clone(),
            };
            ctx.characters.insert(id.clone(), cd);
            NextAction::Continue
        },
        Stmt::Narration { lines, .. } => {
            let processed_lines: Vec<String> = lines.iter()
                .map(|l| interpolate(lua, l))
                .collect();

            for i in &processed_lines{
                ctx.dialogue_history.push(DialogueRecord {speaker: None, text: i.clone(), voice_path: None});
            }
            events.push(OutputEvent::ShowNarration { lines: processed_lines });
            NextAction::WaitInput
        },
        Stmt::Dialogue {speaker, text, voice_index, ..} => {
            let mut name = speaker.name.clone();
            let mut path = None;
            if let Some(cn) = ctx.characters.get(&name) {
                name = cn.name.clone();
                if let Some(vi) = voice_index {
                    path = Some(cn.to_owned().voice_tag.unwrap().add(&*audio_cfg.voice_link_char).add(vi));
                }
            }
            if let Some(al) = &speaker.alias{
                name = al.clone();
            }
            if path.is_some(){
                ctx.audios.insert("voice".to_string(), Some(Audio{
                    path:path.clone().unwrap(), 
                    volume: audio_cfg.voice_volume,
                    fade_in: 0f32, 
                    fade_out: 0f32, 
                    looping: false
                }));
                events.push(OutputEvent::PlayAudio {
                    channel: "voice".to_string(), 
                    path:path.clone().unwrap(), 
                    fade_in: 0f32, 
                    volume: audio_cfg.voice_volume,
                    looping: false});
            }

            let final_text = interpolate(lua, text);

            ctx.dialogue_history.push(DialogueRecord {speaker: Some(name.clone()), text: final_text.clone(), voice_path: path.clone()});
            events.push(OutputEvent::ShowDialogue {name, content: final_text.clone()});
            NextAction::WaitInput
        },
        Stmt::Audio {action, channel, resource, options, ..} => {
            if !ctx.audios.contains_key(channel) {
                log::error!("Audio channel {} isn't registered", channel);
            }
            if matches!(action, AudioAction::Play){
                let path = resource.clone().unwrap().to_string();
                let volume = options.volume.unwrap_or(audio_cfg.master_volume);
                let fade_in = options.fade_in.unwrap_or(audio_cfg.fade_in_sec);
                let fade_out = options.fade_out.unwrap_or(audio_cfg.fade_out_sec);
                let looping = options.r#loop;
                ctx.audios.insert(channel.to_string(), Some(Audio{
                    path: path.clone(),
                    volume, fade_in, fade_out, looping
                }));
                events.push(OutputEvent::PlayAudio {channel:channel.to_string(), path: path.clone(), fade_in, volume, looping });
            }else{
                let fade_out = if let Some(k) = options.fade_out{
                    k
                } else {
                    ctx.audios.get(channel).unwrap().clone().unwrap().fade_out
                };
                events.push(OutputEvent::StopAudio {channel:channel.to_string(), fade_out});
                ctx.audios.insert(channel.to_string(), None);
            }
            NextAction::Continue
        },
        Stmt::Scene {image, transition, ..} => {
            if let Some(img) = image {
                if let Some(layer) = ctx.layer_record.layer.get_mut("master") {
                    layer.clear();
                    layer.push(Sprite {
                        target: img.clone().prefix, 
                        attrs: img.attrs.clone().unwrap_or(vec![]), 
                        position: None,
                        zindex: 0usize
                    });
                    events.push(OutputEvent::NewScene {transition: transition.clone()
                        .unwrap_or(Transition{effect: gfx_cfg.default_transition}).effect});
                }
            }else {
                if let Some(layer) = ctx.layer_record.layer.get_mut("master") {
                    layer.clear();
                    events.push(OutputEvent::NewScene {transition: transition.clone()
                        .unwrap_or(Transition{effect: gfx_cfg.default_transition}).effect});
                }
            }
            NextAction::Continue
        }
        Stmt::Show {target, attrs, position, transition, ..}=>{
            let mut old = false;
            if let Some(layer) = ctx.layer_record.layer.get_mut("master") {
                if let Some(c) = layer.iter_mut().find(|x| x.target == *target) {
                    old = true;
                    if let Some(attrs_list) = attrs {
                        for attr in attrs_list {
                            match attr {
                                ShowAttr::Add(a) =>{
                                    c.attrs.pop();
                                    c.attrs.push(a.to_string());
                                },
                                ShowAttr::Remove(a) => {
                                    if a == c.attrs.last().unwrap(){
                                        c.attrs.pop();
                                    }
                                }
                            }
                        }
                    }
                    if let Some(trans) = position {
                        c.position = Some(trans.to_string());
                    }
                    events.push(OutputEvent::UpdateSprite {target: target.clone(), transition:transition.clone()
                        .unwrap_or(Transition{effect: gfx_cfg.default_transition.clone()}).effect
                    });
                }
            }

            if !old{
                ctx.layer_record.layer.get_mut("master").unwrap()
                    .push(Sprite {
                        target:target.to_string(),
                        attrs: attrs.clone().unwrap_or(vec![]).into_iter()
                            .filter_map(|x| match x{
                                ShowAttr::Add(s) => Some(s.clone()),
                                _=>None
                            }).collect(),
                        position: position.clone(),
                        zindex: 1usize,
                    });
                events.push(OutputEvent::NewSprite {target: target.clone(), transition:transition.clone()
                    .unwrap_or(Transition{effect: gfx_cfg.default_transition}).effect
                });
            }
            NextAction::Continue
        },
        Stmt::Hide {target, transition, ..} => {
            if let Some(pos) = ctx.layer_record.layer.get("master").unwrap()
                .iter().position(|x| x.target == *target) {
                ctx.layer_record.layer.get_mut("master").unwrap().remove(pos);
                events.push(OutputEvent::HideSprite {
                    target: target.clone(),
                    transition: transition.as_ref().map(|t| t.effect.clone())
                });
            }
            NextAction::Continue
        }
        Stmt::LuaBlock {code,..} => {
            lua.load(code).exec().unwrap_or_else(|e| log::error!("Lua: {}", e));
            NextAction::Continue
        },
        Stmt::Choice {title, arms,id ,..}=>{
            let base_id = id.as_ref().expect("AST not preprocessed! Call preload_script first.");

            let processed_title = title.as_ref().map(|t| interpolate(lua, t));

            let options: Vec<String> = arms.iter()
                .map(|a| interpolate(lua, &a.text))
                .collect();

            let arms_data: Vec<(String, Vec<Stmt>)> = arms.iter().enumerate().map(|(idx, a)| {
                let arm_id = format!("{}_opt{}", base_id, idx);
                (arm_id, a.body.clone())
            }).collect();

            ctx.push(OutputEvent::ShowChoice { title: processed_title, options });
            NextAction::WaitChoice(arms_data)
        },
        Stmt::If {branches, else_branch, id, ..} => {
            let base_id = id.as_ref().expect("AST not preprocessed! Call preload_script first.");

            let mut matched = None;

            for (idx, (cond_str, body)) in branches.iter().enumerate() {
                if lua_glue::evel_bool(lua, cond_str) {
                    let block_id = format!("{}_b{}", base_id, idx);
                    matched = Some((block_id, body.clone()));
                    break
                }
            }

            if matched.is_none() {
                if let Some(body) = else_branch {
                    let block_id = format!("{}_else", base_id);
                    matched = Some((block_id, body.clone()));
                }
            }

            if let Some((bid, body)) = matched {
                NextAction::EnterBlock(bid, body)
            } else {
                NextAction::Continue
            }
        },
        Stmt::Jump {target,..} => NextAction::Jump(target.clone()),
        Stmt::Call {target,..} => NextAction::Call(target.clone()),
        _=> {NextAction::Continue}

    };
    StmtEffect { events, next }
}