use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use lumina_core::{config, runtime::Ctx, OutputEvent};
use lumina_core::event::InputEvent;
use lumina_core::renderer::driver::Driver;
use viviscript_core::{ast::Script, lexer::Lexer, parser::Parser};

fn make_script(lines: usize) -> Script {
    let mut buf = String::with_capacity(lines * 40);
    buf.push_str("character ch1 name=ch1 image_tag=ch1\n");
    buf.push_str("character ch2 name=ch2 image_tag=ch2\n");
    buf.push_str("label init\n");
    for i in 0..lines {
        match i % 8 {
            0 => buf.push_str(&format!("scene bg{i}\n")),
            1 => buf.push_str(&format!("ch1: dialogue {i}\n")),
            2 => buf.push_str(&format!("show spr{i}\n")),
            3 => buf.push_str(&format!("play music bgm{i} volume=0.7 loop\n")),
            4 => buf.push_str(&format!("hide spr{i}\n")),
            5 => buf.push_str(&format!("ch2: dialogue {i}\n")),
            6 => buf.push_str("choice test\n 0: call empty\n 1: call empty\nenco\n"),
            7 => buf.push_str(&format!("call empty {i}\n")),
            _ => unreachable!(),
        }
    }
    buf.push_str("enlb\nlabel empty\nenlb\n");

    let tokens = Lexer::new(&buf).run();
    Parser::new(&tokens).parse()
}

struct NullRenderer;
impl lumina_core::renderer::Renderer for NullRenderer {
    fn render(
        &mut self,
        out: &OutputEvent,
        _ctx: &mut Ctx,
    ) -> Option<InputEvent> {
        match out {
            OutputEvent::ShowChoice { .. } => Some(InputEvent::ChoiceMade { index: 0 }),
            OutputEvent::ShowDialogue {..} | OutputEvent::ShowNarration {..} => Some(InputEvent::Continue),
            _ => None,
        }
    }
}

static INIT: std::sync::Once = std::sync::Once::new();
fn bench_executor(c: &mut Criterion) {
    const LINES: usize = 10_000;
    let mut group = c.benchmark_group("executor");
    group.sample_size(10);

    group.bench_function("step 10k stmts", |b| {
        INIT.call_once(|| config::init_global("config.toml"));
        b.iter_batched(||make_script(LINES),
        |sc| {
            let mut ctx = Ctx::default();
            let mut drv = Driver::new(&mut ctx, sc, NullRenderer);
            drv.run(&mut ctx);
        },
        BatchSize::SmallInput);
    });
    group.finish();
}

criterion_group!(benches, bench_executor);
criterion_main!(benches);