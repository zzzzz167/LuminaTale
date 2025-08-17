use crate::event::{InputEvent, OutputEvent};
use crate::renderer::Renderer;
use crate::Ctx;
use ratatui::text::Text;
use ratatui::{
    backend::{CrosstermBackend},
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
    },
    layout::{Constraint, Direction, Layout}
    ,
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal
};
use std::io::{self, Stdout};

pub struct TuiRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    input_buffer: String,
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
    fn render(&self) -> Text<'static> {
        match self {
            CurrentText::Empty => Text::raw(""),
            CurrentText::Narration(lines) => Text::raw(lines.clone()),
            CurrentText::Dialogue { name, content } => {
                let mut lines: Vec<Line<'static>> = Vec::new();
                lines.push(Line::from(format!("「{}」", name)));
                lines.push(Line::from(content.clone()));
                Text::from(lines)
            }
            CurrentText::Choice { title, options } => {
                let mut lines: Vec<Line<'static>> = Vec::new();
                if let Some(tit) = title {
                    lines.push(Line::from(tit.clone()));
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
            input_buffer: String::new(),
            current_text: CurrentText::Empty,
        })
    }

    fn try_read_key(&mut self) -> io::Result<Option<InputEvent>> {
        if !event::poll(std::time::Duration::from_millis(50))? {
            return Ok(None);
        }
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(None);
            }
            return Ok(match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(InputEvent::Exit),
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                    None
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                    None
                }
                KeyCode::Enter => {
                    // 把整行输入解析成命令，然后清空缓冲区
                    let line = self.input_buffer.clone();
                    self.input_buffer.clear();
                    parse_command(&line)
                }
                _ => None,
            });
        }
        Ok(None)
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

            let dialog_block = Block::default()
                .borders(Borders::ALL)
                .title("Current");

            let dialog_paragraph = Paragraph::new(self.current_text.render()).block(dialog_block).wrap(ratatui::widgets::Wrap { trim: false });
            f.render_widget(dialog_paragraph, dialog_area);

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

            let cmd_block = Block::default()
                .borders(Borders::ALL)
                .title("Command");
            let cmd_paragraph = Paragraph::new(format!("> {}", &self.input_buffer)).block(cmd_block);
            f.render_widget(cmd_paragraph, cmd_area);
        })?;
        Ok(())
    }
}

fn parse_command(line: &str) -> Option<InputEvent> {
    if let Ok(idx) = line.parse::<usize>() {
        // 选项展示时是从 1 开始，所以减 1
        return Some(InputEvent::ChoiceMade { index: idx.saturating_sub(1) });
    }
    
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.get(0).map(|s| *s) {
        Some("save") => {
            if let Some(slot) = parts.get(1).and_then(|s| s.parse::<u32>().ok()) {
                Some(InputEvent::SaveRequest { slot })
            } else {
                None
            }
        }
        Some("load") => {
            if let Some(slot) = parts.get(1).and_then(|s| s.parse::<u32>().ok()) {
                Some(InputEvent::LoadRequest { slot })
            } else {
                None
            }
        }
        Some("exit") | Some("quit") => Some(InputEvent::Exit),
        Some("continue") | Some("c") |Some("")=> Some(InputEvent::Continue),
        None => Some(InputEvent::Continue),
        _ => None,
    }
}

impl Renderer for TuiRenderer {
    fn render(&mut self, out: &OutputEvent, ctx: &mut Ctx)  -> Option<InputEvent> {
        match out {
            OutputEvent::ShowNarration { lines } => {
                self.current_text = CurrentText::Narration(lines.join("\n"));
            }
            OutputEvent::ShowDialogue { name, content } => {
                self.current_text = CurrentText::Dialogue {
                    name: name.clone(),
                    content: content.clone(),
                };
            }
            OutputEvent::ShowChoice { title, options } => {
                self.current_text = CurrentText::Choice {
                    title: title.clone(),
                    options: options.clone(),
                };
            }
            _ => {}
        }
        
        if let Err(e) = self.draw(ctx) {
            eprintln!("draw error: {e}");
            return Some(InputEvent::Exit);
        }
        
        match self.try_read_key() {
            Ok(Some(ev)) => Some(ev),
            Ok(None) => None,
            Err(e) => {
                eprintln!("key read error: {e}");
                Some(InputEvent::Exit)
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