use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        execute, event,
        event::{Event, KeyEventKind, KeyCode}
    },
    text::{Line, Text},
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Paragraph, ListItem, List},
    Terminal
};
use std::{io::Stdout, io};
use std::sync::Arc;
use viviscript_core::ast::Script;
use crate::{
    Ctx, OutputEvent,
    event::InputEvent,
    renderer::{Renderer, driver::ExecutorHandle}
};

pub struct TuiRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    input_buf: String,
    current_text: CurrentText,
}

#[derive(Debug, Clone, Default)]
enum CurrentText {
    #[default]
    Empty,
    Narration(String),
    Dialogue { name: String, content: String },
    Choice {
        title: Option<String>,
        options: Vec<String>,
    },
}

impl CurrentText {
    fn to_text(&self) -> Text<'static> {
        match self {
            CurrentText::Empty => Text::raw(""),
            CurrentText::Narration(lines) => Text::raw(lines.clone()),
            CurrentText::Dialogue { name, content } => {
                let mut lines = Vec::new();
                lines.push(Line::from(format!("「{}」", name)));
                lines.push(Line::from(content.clone()));
                Text::from(lines)
            }
            CurrentText::Choice { title, options } => {
                let mut lines = Vec::new();
                if let Some(t) = title {
                    lines.push(Line::from(t.clone()));
                }
                for (idx, opt) in options.iter().enumerate() {
                    lines.push(Line::from(format!("  {}. {}", idx + 1, opt)));
                }
                lines.push(Line::from(""));
                lines.push(Line::from("请输入数字进行选择"));
                Text::from(lines)
            }
        }
    }
}

impl TuiRenderer {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            input_buf: String::new(),
            current_text: CurrentText::Empty,
        })
    }

    fn try_read_key(&mut self) -> io::Result<Option<InputEvent>> {
        if !event::poll(std::time::Duration::from_millis(50))? {
            return Ok(None);
        }
        let Event::Key(key) = event::read()? else { return Ok(None) };
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }

        Ok(match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(InputEvent::Exit),
            KeyCode::Char(c) => {
                self.input_buf.push(c);
                None
            }
            KeyCode::Backspace => {
                self.input_buf.pop();
                None
            }
            KeyCode::Enter => {
                let line = std::mem::take(&mut self.input_buf);
                parse_command(&line)
            }
            _ => None,
        })
    }

    fn draw(&mut self, ctx: &Ctx) -> io::Result<()> {
        self.terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(60), Constraint::Min(0), Constraint::Length(3)])
                .split(size);
            let top = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(chunks[0]);
            let left = top[0];
            let right = top[1];
            let bottom_main = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(chunks[1]);

            let dialog_area = bottom_main[0];
            let hist_area = bottom_main[1];
            let cmd_area = chunks[2];

            let scene_block = Block::default()
                .borders(Borders::ALL)
                .title("Scene / Sprites");
            let mut scene_text = vec![Line::from("Layers:")];
            for (layer_name, sprites) in &ctx.layer_record.layer {
                scene_text.push(Line::from(format!("  [{}]:", layer_name)));
                for sp in sprites {
                    scene_text.push(Line::from(format!(
                        "    {}  {:?}",
                        sp.target,
                        sp.attrs.join(",")
                    )))
                }
            }
            let scene_paragraph = Paragraph::new(Text::from(scene_text)).block(scene_block);
            f.render_widget(scene_paragraph, left);

            let audio_block = Block::default()
                .borders(Borders::ALL)
                .title("Audio Queue");
            let mut audio_text = vec![];
            for (ch, aud_opt) in &ctx.audios {
                if let Some(audio) = aud_opt {
                    audio_text.push(Line::from(format!(
                        "{}: {} ▶ {}% {}",
                        ch,
                        audio.path,
                        (audio.volume * 100.0) as u8,
                        if audio.looping { "loop" } else { "once" }
                    )));
                } else {
                    audio_text.push(Line::from(format!("{}: --", ch)));
                }
            }
            let audio_paragraph = Paragraph::new(Text::from(audio_text)).block(audio_block);
            f.render_widget(audio_paragraph, right);

            let dialog_block = Block::default().borders(Borders::ALL).title("Current");
            f.render_widget(
                Paragraph::new(self.current_text.to_text())
                    .block(dialog_block)
                    .wrap(ratatui::widgets::Wrap { trim: false }),
                dialog_area,
            );

            let hist_block = Block::default()
                .borders(Borders::ALL)
                .title("History");
            let hist_items: Vec<ListItem> = ctx
                .dialogue_history
                .iter()
                .rev()
                .take(15)
                .map(|rec| {
                    let speaker = rec.speaker.as_deref().unwrap_or("Narrator");

                    ListItem::new(format!("{}: {}", speaker, rec.text))
                })
                .collect();
            let hist_list = List::new(hist_items).block(hist_block);
            f.render_widget(hist_list, hist_area);

            let cmd_block = Block::default().borders(Borders::ALL).title("Command");
            f.render_widget(
                Paragraph::new(format!("> {}", self.input_buf)).block(cmd_block),
                cmd_area,
            );
        })?;

        Ok(())
    }
}

impl Renderer for TuiRenderer {
    fn run_event_loop(&mut self, ctx: &mut Ctx, script: Arc<Script>) {
        let mut driver = ExecutorHandle::new(ctx, script);

        loop {
            let waiting = driver.step(ctx);

            for out in ctx.drain() {
                if matches!(out, OutputEvent::End) {
                    return;
                }
                self.current_text = match out {
                    OutputEvent::ShowNarration { lines } => {
                        CurrentText::Narration(lines.join("\n"))
                    }
                    OutputEvent::ShowDialogue { name, content } => {
                        CurrentText::Dialogue { name, content }
                    }
                    OutputEvent::ShowChoice { title, options } => {
                        CurrentText::Choice { title, options }
                    }
                    _ => continue,
                };
            }

            if let Err(e) = self.draw(ctx) {
                log::error!("TUI draw error: {}", e);
                return;
            }

            if waiting {
                match self.try_read_key() {
                    Ok(Some(ev)) => driver.feed(ctx, ev),
                    Ok(None) => {}
                    Err(e) => {
                        log::error!("TUI key read error: {}", e);
                        return;
                    }
                }
            }
        }
    }
}

impl Drop for TuiRenderer {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

fn parse_command(line: &str) -> Option<InputEvent> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.get(0).copied() {
        Some("save") => parts
            .get(1)
            .and_then(|s| s.parse::<u32>().ok())
            .map(|slot| InputEvent::SaveRequest { slot }),
        Some("load") => parts
            .get(1)
            .and_then(|s| s.parse::<u32>().ok())
            .map(|slot| InputEvent::LoadRequest { slot }),
        Some("exit") | Some("quit") => Some(InputEvent::Exit),
        Some("continue") | Some("c") | Some("") => Some(InputEvent::Continue),
        None => Some(InputEvent::Continue),
        _ => line
            .parse::<usize>()
            .ok()
            .map(|idx| InputEvent::ChoiceMade {
                index: idx.saturating_sub(1),
            }),
    }
}