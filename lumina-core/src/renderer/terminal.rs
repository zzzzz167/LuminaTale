use crate::event::{InputEvent, OutputEvent};
use crate::renderer::Renderer;
use std::io::{stdin, stdout, Write};

pub struct TerminalRenderer;

impl Renderer for TerminalRenderer {
    fn render(&mut self, out: &OutputEvent) -> Option<InputEvent> {
        match out {
            OutputEvent::ShowNarration { lines } => {
                for l in lines {
                    println!("[Narration] {}", l);
                }
                self.wait_continue()
            }
            OutputEvent::ShowDialogue { name, content } => {
                println!("[Dialogue] {}: {}", name, content);
                self.wait_continue()
            }
            OutputEvent::ShowChoice { title, options } => {
                if let Some(t) = title {
                    println!("--- {} ---", t);
                }
                for (i, o) in options.iter().enumerate() {
                    println!("  [{}] {}", i + 1, o);
                }
                self.wait_choice(options.len())
            }
            OutputEvent::PlayAudio { channel, path, fade_in, volume, .. } => {
                println!("[PlayAudio] {}:{} fade_in:{} volume:{}", channel, path, fade_in, volume);
                None
            }
            OutputEvent::StopAudio { channel, fade_out } => {
                println!("[StopAudio] {} fade_out:{}", channel, fade_out);
                None
            }
            OutputEvent::End | OutputEvent::StepDone => None,
            _ => None,
        }
    }
}

impl TerminalRenderer {
    fn wait_continue(&mut self) -> Option<InputEvent> {
        loop {
            print!("> "); stdout().flush().unwrap();
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            let trimmed = buf.trim_end();
            if trimmed.is_empty() {
                return Some(InputEvent::Continue);
            }
            if trimmed.eq_ignore_ascii_case("exit") {
                return Some(InputEvent::Exit);
            }
            if trimmed.starts_with(":save") {
                if let Ok(slot) = trimmed[5..].trim().parse::<u32>() {
                    return Some(InputEvent::SaveRequest { slot });
                }
            }
            if trimmed.starts_with(":load") {
                if let Ok(slot) = trimmed[5..].trim().parse::<u32>() {
                    return Some(InputEvent::LoadRequest { slot });
                }
            }
            println!("invalid");
        }
    }

    fn wait_choice(&mut self, len: usize) -> Option<InputEvent> {
        loop {
            print!("Select> "); stdout().flush().unwrap();
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            if let Ok(n) = buf.trim().parse::<usize>() {
                if n >= 1 && n <= len {
                    return Some(InputEvent::ChoiceMade { index: n - 1 });
                }
            }
            println!("invalid");
        }
    }
}