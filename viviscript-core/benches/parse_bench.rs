use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use viviscript_core::{lexer::Lexer, parser::Parser};

fn make_script(lines:usize) -> String {
    let mut buf = String::with_capacity(lines * 40);
    
    for i in 0..lines {
        match i % 7 {
            0 => buf.push_str(&format!("character c{i} name=\"角色{i}\" image_tag=\"c{i}\"\n")),
            1 => buf.push_str(&format!("scene bg{i} with fade_in\n")),
            2 => buf.push_str(&format!("show spr{i} at center with fade_in\n")),
            3 => buf.push_str(&format!("hide spr{i}\n")),
            4 => buf.push_str(&format!("play music bgm{i} volume=0.8 loop\n")),
            5 => buf.push_str(&format!("c{i}: \"Hello world {i}\"\n")),
            6 => {
                buf.push_str(&format!("choice \"第{i}个选择\"\n"));
                buf.push_str(&format!("\"选项A\": jump label_a_{i}\n"));
                buf.push_str(&format!("\"选项B\": jump label_b_{i}\n"));
                buf.push_str("enco\n");
            }
            _ => unreachable!(),
        }
    }

    buf.push_str("label end\nstop bgm0 fade_out=1.0\nenlb\n");
    buf
}


fn bench_full(c: &mut Criterion) {
    let src = make_script(10_000);
    let mut group = c.benchmark_group("parse");
    group.sample_size(10);
    group.bench_function("lex+parse 10k lines", |b| {
        b.iter(|| {
            let tokens = Lexer::new(black_box(&src)).run();
            let _ast = Parser::new(black_box(&tokens)).parse();
        })
    });
    group.finish();
}

criterion_group!(benches, bench_full);
criterion_main!(benches);