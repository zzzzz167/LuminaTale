use crate::event::{EngineEvent, Mode};
use crate::renderer::Renderer;
use std::io::{stdin, stdout, Write};
use crate::runtime::Ctx;


pub struct TerminalRenderer;

impl Renderer for TerminalRenderer {
    fn handle(&mut self, ev: &EngineEvent) -> Option<EngineEvent> {
        match ev {
            EngineEvent::ShowNarration { lines } => {
                for l in lines { println!("[Narration] {}", l); }
                loop {
                    print!("> "); stdout().flush().unwrap();
                    let mut buf = String::new();
                    stdin().read_line(&mut buf).unwrap();
                    let trimmed = buf.trim_end();
                    if trimmed.is_empty(){
                        return Some(EngineEvent::InputMode {mode: Mode::Continue}); 
                    }
                    if trimmed.eq_ignore_ascii_case("exit") {
                        return Some(EngineEvent::InputMode {mode: Mode::Exit});
                    }
                    println!("invalid");
                }
            },
            EngineEvent::ShowDialogue {name, content} => {
                println!("[Dialogue] {}:{}", name,content);
                loop {
                    print!("> "); stdout().flush().unwrap();
                    let mut buf = String::new();
                    stdin().read_line(&mut buf).unwrap();
                    let trimmed = buf.trim_end();
                    if trimmed.is_empty(){
                        return Some(EngineEvent::InputMode {mode: Mode::Continue});
                    }
                    if trimmed.eq_ignore_ascii_case("exit") {
                        return Some(EngineEvent::InputMode {mode: Mode::Exit});
                    }
                    println!("invalid");
                }
            },
            EngineEvent::ShowChoice { title, options } => {
                if let Some(t) = title { println!("--- {} ---", t); }
                for (i, o) in options.iter().enumerate() {
                    println!("  [{}] {}", i + 1, o);
                }
                loop {
                    print!("Select> "); stdout().flush().unwrap();
                    let mut buf = String::new();
                    stdin().read_line(&mut buf).unwrap();
                    if let Ok(n) = buf.trim().parse::<usize>() {
                        if n >= 1 && n <= options.len() {
                            return Some(EngineEvent::ChoiceMade { index: n - 1 });
                        }
                    }
                    println!("invalid");
                }
            },
            EngineEvent::PlayAudio { channel, path, fade_in, volume, ..} => {
                println!("[PlayAudio] {}:{} fade_in:{} volume:{}",channel,path,fade_in,volume);
                None
            },
            EngineEvent::StopAudio {channel, fade_out} => {
                println!("[StopAudio] {} fade_out:{}", channel,fade_out);
                None
            }
            _ => None
        }
    }
}