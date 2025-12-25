use criterion::{criterion_group, criterion_main, Criterion};
use lumina_core::event::InputEvent;
use lumina_core::renderer::driver::ExecutorHandle;
use lumina_core::renderer::Renderer;
use lumina_core::ScriptManager;
use lumina_core::{runtime::Ctx, OutputEvent};
use std::sync::Arc;

// 生成测试用的 .vivi 脚本源码
fn make_script_source(lines: usize) -> String {
    let mut buf = String::with_capacity(lines * 40);
    // 定义角色
    buf.push_str("character ch1 name=ch1 image_tag=ch1\n");
    buf.push_str("character ch2 name=ch2 image_tag=ch2\n");

    // 入口标签 (Executor 默认找 init)
    buf.push_str("label init\n");

    for i in 0..lines {
        match i % 8 {
            0 => buf.push_str(&format!("scene bg{i}\n")),
            1 => buf.push_str(&format!("ch1: dialogue {i}\n")),
            2 => buf.push_str(&format!("show spr{i}\n")),
            3 => buf.push_str(&format!("play music bgm{i} volume=0.7 loop\n")),
            4 => buf.push_str(&format!("hide spr{i}\n")),
            5 => buf.push_str(&format!("ch2: dialogue {i}\n")),
            6 => buf.push_str("choice \"test\"\n \"zero\": call empty\n \"first\": call empty\nenco\n"),
            7 => buf.push_str("call empty\n"),
            _ => unreachable!(),
        }
    }
    // 结束标签
    buf.push_str("enlb\nlabel empty\nenlb\n");
    buf
}

struct NullRenderer;
impl Renderer for NullRenderer {
    fn run_event_loop(&mut self, ctx: &mut Ctx, manager: Arc<ScriptManager>) {
        let mut driver = ExecutorHandle::new(ctx, manager);

        loop {
            // 模拟驱动步进
            let _waiting = driver.step(ctx);

            // 模拟事件消耗
            for out in ctx.drain() {
                match out {
                    OutputEvent::ShowChoice { .. } => driver.feed(ctx, InputEvent::ChoiceMade {index: 0}),
                    OutputEvent::ShowDialogue {..} | OutputEvent::ShowNarration {..} => driver.feed(ctx, InputEvent::Continue),
                    OutputEvent::End => return,
                    _ => {}
                }
            }
        }
    }
}

static INIT: std::sync::Once = std::sync::Once::new();

fn bench_executor(c: &mut Criterion) {
    const LINES: usize = 10_000;
    let mut group = c.benchmark_group("executor");
    group.sample_size(10);

    // 1. 准备测试环境
    let dir = std::env::temp_dir().join("lumina_bench_env");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();

    let script_path = dir.join("bench.vivi");
    std::fs::write(&script_path, make_script_source(LINES)).unwrap();

    // 2. 初始化 Manager 并加载项目 (这是 Setup 阶段，不计入 Benchmark 循环)
    let mut manager = ScriptManager::new();
    manager.load_project(&dir).expect("Failed to load bench project");
    let manager_arc = Arc::new(manager);

    group.bench_function("step 10k stmts", |b| {
        INIT.call_once(|| {
            // 忽略配置加载错误，测试环境可能没有 config 文件
            let _ = lumina_shared::config::init("bench_dummy.toml");
        });

        b.iter(|| {
            let mut ctx = Ctx::default();
            let mut renderer = NullRenderer;
            renderer.run_event_loop(&mut ctx, manager_arc.clone());
        });
    });
    let _ = std::fs::remove_dir_all(dir);

    group.finish();
}

criterion_group!(benches, bench_executor);
criterion_main!(benches);