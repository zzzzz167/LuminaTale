use std::ops::Add;
use crate::runtime::Ctx;
use crate::event::OutputEvent;
use crate::runtime::assets::{Audio, DialogueRecord, Sprite};
use crate::config;
use viviscript_core::ast::{Stmt, AudioAction, ShowAttr, Transition};
use mlua::Lua;

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
    WaitChoice(Vec<Vec<Stmt>>),
    WaitInput
}
pub fn walk_stmt(ctx: &mut Ctx, lua: &Lua, stmt: &Stmt) -> StmtEffect {
    log::trace!("walk_stmt: {:?}", stmt);
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
            events.push(OutputEvent::ShowNarration { lines: lines.clone() });
            for i in lines{
                ctx.dialogue_history.push(DialogueRecord {speaker: None, text: i.clone(), voice_path: None});
            }
            NextAction::WaitInput
        },
        Stmt::Dialogue {speaker, text, voice_index, ..} => {
            let mut name = speaker.name.clone();
            let mut path = None;
            if let Some(cn) = ctx.characters.get(&name) {
                name = cn.name.clone();
                if let Some(vi) = voice_index {
                    path = Some(cn.to_owned().voice_tag.unwrap().add(&config::get().audio.voice_link).add(vi));
                }
            }
            if let Some(al) = &speaker.alias{
                name = al.clone();
            }
            if path.is_some(){
                ctx.audios.insert("voice".to_string(), Some(Audio{
                    path:path.clone().unwrap(), 
                    volume: config::get().audio.voice_volume, 
                    fade_in: 0f32, 
                    fade_out: 0f32, 
                    looping: false
                }));
                events.push(OutputEvent::PlayAudio {
                    channel: "voice".to_string(), 
                    path:path.clone().unwrap(), 
                    fade_in: 0f32, 
                    volume: config::get().audio.voice_volume, 
                    looping: false});
            }
            ctx.dialogue_history.push(DialogueRecord {speaker: Some(name.clone()), text: text.clone(), voice_path: path.clone()});
            events.push(OutputEvent::ShowDialogue {name, content: text.clone()});
            NextAction::WaitInput
        },
        Stmt::Audio {action, channel, resource, options, ..} => {
            if !ctx.audios.contains_key(channel) {
                log::error!("Audio channel {} isn't registered", channel);
            }
            if matches!(action, AudioAction::Play){
                let path = resource.clone().unwrap().to_string();
                let volume = options.volume.unwrap_or(config::get().audio.default_volume);
                let fade_in = options.fade_in.unwrap_or(config::get().audio.fade_in);
                let fade_out = options.fade_out.unwrap_or(config::get().audio.fade_out);
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
                        .unwrap_or(Transition{effect:config::get().layer.trans_effect.clone()}).effect});
                }
            }else {
                if let Some(layer) = ctx.layer_record.layer.get_mut("master") {
                    layer.clear();
                    events.push(OutputEvent::NewScene {transition: transition.clone()
                        .unwrap_or(Transition{effect:config::get().layer.trans_effect.clone()}).effect});
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
                    events.push(OutputEvent::UpdateSprite {transition:transition.clone()
                        .unwrap_or(Transition{effect:config::get().layer.trans_effect.clone()}).effect
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
                events.push(OutputEvent::NewSprite {transition:transition.clone()
                    .unwrap_or(Transition{effect:config::get().layer.trans_effect.clone()}).effect
                });
            }
            NextAction::Continue
        },
        Stmt::Hide {target, ..} => {
            if let Some(pos) = ctx.layer_record.layer.get("master").unwrap()
                .iter().position(|x| x.target == *target) {
                ctx.layer_record.layer.get_mut("master").unwrap().remove(pos);
                events.push(OutputEvent::HideSprite);
            }
            NextAction::Continue
        }
        Stmt::LuaBlock {code,..} => {
            lua.load(code).exec().unwrap_or_else(|e| log::error!("Lua: {}", e));
            NextAction::Continue
        },
        Stmt::Choice {title, arms, ..}=>{
            let options: Vec<String> = arms.iter().map(|a| a.text.clone()).collect();
            let bodies: Vec<Vec<Stmt>> = arms.iter().map(|a| a.body.clone()).collect();
            ctx.push(OutputEvent::ShowChoice { title: title.clone(), options });
            NextAction::WaitChoice(bodies)
        }
        Stmt::Jump {target,..} => NextAction::Jump(target.clone()),
        Stmt::Call {target,..} => NextAction::Call(target.clone()),
        _=> {NextAction::Continue}

    };
    StmtEffect { events, next }
}