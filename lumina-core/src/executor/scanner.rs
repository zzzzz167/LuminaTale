use crate::runtime::Ctx;
use viviscript_core::ast::{AudioAction, ShowAttr, Stmt};

pub struct Scanner;

impl Scanner {
    pub fn scan(
        start_stmts: &[Stmt],
        start_pc: usize,
        lookahead_steps: usize,
        ctx: &Ctx,
    )-> (Vec<String>, Vec<String>){
        let mut images = Vec::new();
        let mut audios = Vec::new();

        let mut current_stmts = start_stmts;
        let mut pc = start_pc;
        let mut steps_taken = 0;

        while steps_taken < lookahead_steps && pc < current_stmts.len() {
            let stmt = &current_stmts[pc];
            steps_taken += 1;
            pc += 1;

            match stmt {
                Stmt::Show { target, attrs, .. } => {
                    let base_name = ctx.characters.get(target)
                        .and_then(|c| c.image_tag.clone())
                        .unwrap_or_else(|| target.clone());
                    let mut suffixes = Vec::new();
                    if let Some(attr_list) = attrs {
                        for attr in attr_list {
                            if let ShowAttr::Add(tag) = attr {
                                suffixes.push(tag.as_str());
                            }
                        }
                    }
                    let full_name = if suffixes.is_empty() {
                        base_name
                    } else {
                        format!("{}_{}", base_name, suffixes.join("_"))
                    };

                    images.push(full_name);
                },
                Stmt::Scene { image, .. } => {
                    if let Some(scene_img) = image {
                        let mut parts = vec![scene_img.prefix.as_str()];
                        if let Some(attrs) = &scene_img.attrs {
                            for a in attrs {
                                parts.push(a.as_str());
                            }
                        }
                        images.push(parts.join("_"));
                    }
                },
                Stmt::Audio { action, channel, resource, .. } => {
                    if *action == AudioAction::Play {
                        if let Some(res_path) = resource {
                            let is_bgm = channel == "music" || res_path.starts_with("bgm_");

                            if !is_bgm {
                                audios.push(res_path.clone());
                            }
                        }
                    }
                },
                Stmt::Label { .. } | Stmt::Jump { .. } | Stmt::Choice { .. } | Stmt::If { .. } | Stmt::Call { .. } => {
                    break;
                }

                _ => {}
            }
        }
        (images, audios)
    }
}