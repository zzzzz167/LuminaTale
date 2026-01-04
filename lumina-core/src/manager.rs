use std::collections::HashMap;
use std::path::{Path};
use std::sync::Arc;
use walkdir::WalkDir;
use log::{info};
use anyhow::{Result, Context};
use rustc_hash::FxHashMap;

use viviscript_core::ast::{Script, Stmt};
use viviscript_core::{lexer::Lexer, parser::Parser};
use crate::runtime::Character;

/// 脚本管理器：负责加载、预处理和索引所有脚本
pub struct ScriptManager {
    // 原始 AST 列表 (用于扫描全局定义)
    pub programs: Vec<Arc<Script>>,

    // Label 表 (用于跳转执行)
    pub label_map: FxHashMap<String, Arc<[Stmt]>>,

    // 辅助数据
    pub source_cache: HashMap<String, String>,
    label_sources: HashMap<String, String>,
}

impl ScriptManager {
    pub fn new() -> Self {
        Self {
            programs: Vec::new(),
            label_map: FxHashMap::default(),
            label_sources: HashMap::new(),
            source_cache: HashMap::new(),
        }
    }

    /// 扫描并加载项目
    pub fn load_project(&mut self, root_dir: impl AsRef<Path>) -> Result<()> {
        let root = root_dir.as_ref();
        info!("Scanning script project at: {:?}", root);

        let mut loaded_count = 0;
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "vivi") {
                self.load_file(path)?;
                loaded_count += 1;
            }
        }

        info!("Project loaded. Files: {}, Labels: {}", loaded_count, self.label_map.len());
        Ok(())
    }

    pub fn collect_characters(&self) -> HashMap<String, Character> {
        let mut chars = HashMap::new();
        for script in &self.programs {
            for stmt in &script.body {
                if let Stmt::CharacterDef { id, name, image_tag, voice_tag, .. } = stmt {
                    chars.insert(id.clone(), Character {
                        id: id.clone(),
                        name: name.clone(),
                        image_tag: image_tag.clone(),
                        voice_tag: voice_tag.clone(),
                    });
                }
            }
        }
        chars
    }

    pub fn get_label(&self, name: &str) -> Option<Arc<[Stmt]>> {
        self.label_map.get(name).cloned()
    }

    fn load_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read script: {:?}", path))?;

        // 1. 解析
        let tokens = Lexer::new(&content).run();
        let parse_result = Parser::new(&tokens).parse();

        let mut ast = match parse_result {
            Ok(script) => script,
            Err(errors) => {
                // 打印错误日志，而不是崩溃
                log::error!("Syntax Error in {:?}:", path);
                for err in errors {
                    log::error!("   Line {}: {}", err.line, err.msg);
                }
                anyhow::bail!("Parse failed for {:?}", path);
            }
        };

        let file_key = path.file_stem().unwrap().to_string_lossy().to_string();

        // 2. 预处理 (原本在 Executor 里的逻辑)
        // 展开 Narration
        self.pre_narration_lines(&mut ast.body);

        // 生成唯一 ID 并建立索引
        let mut dummy_map = FxHashMap::default();
        self.preprocess_block(&mut ast.body, &file_key, &mut dummy_map);

        // 3. 将收集到的 Label 放入全局表
        // 注意：这里我们不仅放入了顶层 Label，也放入了 Choice/If 产生的临时 Block
        self.label_map.extend(dummy_map);
        self.build_top_level_index(&ast.body, &file_key)?;

        let script_arc = Arc::new(ast);
        self.programs.push(script_arc);

        self.source_cache.insert(path.to_string_lossy().to_string(), content);
        Ok(())
    }

    fn build_top_level_index(&mut self, stmts: &[Stmt], file_key: &str) -> Result<()> {
        for stmt in stmts {
            if let Stmt::Label { id, body, .. } = stmt {
                if let Some(existing_file) = self.label_sources.get(id) {
                    if existing_file != file_key {
                        // 如果发现重名，直接报错，阻止游戏启动！
                        anyhow::bail!(
                            "Label collision detected!\n  Label '{}' is defined in:\n    1. {}\n    2. {}",
                            id, existing_file, file_key
                        );
                    }
                }
                // 记录来源
                self.label_sources.insert(id.clone(), file_key.to_string());
                // 插入 Map
                self.label_map.insert(id.clone(), Arc::from(body.as_slice()));
                // 递归
                self.build_top_level_index(body, file_key)?;
            }
        }
        Ok(())
    }

    fn pre_narration_lines(&self, body: &mut Vec<Stmt>) {
        let mut new_body = Vec::new();
        for stmt in body.drain(..) {
            match stmt {
                Stmt::Narration {span, lines} => {
                    for l in lines {
                        new_body.push(Stmt::Narration {span, lines: vec![l]});
                    }
                },
                Stmt::Label { span, id, mut body } => {
                    self.pre_narration_lines(&mut body);
                    new_body.push(Stmt::Label { span, id, body });
                },
                Stmt::Choice { span, title, mut arms, id } => {
                    for arm in &mut arms {
                        self.pre_narration_lines(&mut arm.body);
                    }
                    new_body.push(Stmt::Choice { span, title, arms, id });
                }
                Stmt::If { span, mut branches, mut else_branch, id } => {
                    for (_, body) in &mut branches {
                        self.pre_narration_lines(body);
                    }
                    if let Some(body) = &mut else_branch {
                        self.pre_narration_lines(body);
                    }
                    new_body.push(Stmt::If { span, branches, else_branch, id });
                }
                _ => new_body.push(stmt),
            }
        }
        *body = new_body;
    }

    fn preprocess_block(
        &self,
        stmts: &mut [Stmt],
        scope_name: &str,
        map: &mut FxHashMap<String, Arc<[Stmt]>>
    ) {
        let mut counters: HashMap<&str, usize> = HashMap::new();

        for stmt in stmts {
            match stmt {
                Stmt::Label {id, body, ..} => {
                    self.preprocess_block(body, id, map);
                    // Label 本身会在 build_top_level_index 里被收集，这里主要是为了递归处理其内部的 If/Choice
                },
                Stmt::If { branches, else_branch, id, .. } => {
                    let count = counters.entry("if").or_insert(0);
                    let base_id = format!("{}@if_{}", scope_name, count);
                    *count += 1;
                    *id = Some(base_id.clone());

                    for (idx, (_, body)) in branches.iter_mut().enumerate() {
                        let branch_id = format!("{}_b{}", base_id, idx);
                        self.preprocess_block(body, &branch_id, map);
                        map.insert(branch_id.clone(), Arc::from(body.as_slice()));
                    }

                    if let Some(body) = else_branch {
                        let branch_id = format!("{}_else", base_id);
                        self.preprocess_block(body, &branch_id, map);
                        map.insert(branch_id.clone(), Arc::from(body.as_slice()));
                    }
                },
                Stmt::Choice { arms, id, .. } => {
                    let count = counters.entry("choice").or_insert(0);
                    let base_id = format!("{}@choice_{}", scope_name, count);
                    *count += 1;
                    *id = Some(base_id.clone());

                    for (idx, arm) in arms.iter_mut().enumerate() {
                        let arm_id = format!("{}_opt{}", base_id, idx);
                        self.preprocess_block(&mut arm.body, &arm_id, map);
                        map.insert(arm_id.clone(), Arc::from(arm.body.as_slice()));
                    }
                },
                _ => {}
            }
        }
    }
}