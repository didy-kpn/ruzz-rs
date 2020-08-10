use std::{error::Error, io};
use termion::{
    event::Key, input::MouseTerminal, input::TermRead, raw::IntoRawMode, screen::AlternateScreen,
};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use unicode_width::UnicodeWidthStr;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    input_handle: thread::JoinHandle<()>,
    ignore_exit_key: Arc<AtomicBool>,
    tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let input_handle = {
            let tx = tx.clone();
            let ignore_exit_key = ignore_exit_key.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                        if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                            return;
                        }
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            })
        };
        Events {
            rx,
            ignore_exit_key,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}

#[derive(PartialEq, Eq, std::hash::Hash, Copy, Clone)]
enum InputMode {
    Url,
    Params,
    Method,
    Header,
    Body,
}

#[derive(PartialEq, Eq, std::hash::Hash)]
enum OutputMessage {
    Header,
    Body,
}

struct App {
    inputs: std::collections::HashMap<InputMode, String>,
    input_mode: InputMode,
    outputs: std::collections::HashMap<OutputMessage, String>,
}

impl Default for App {
    fn default() -> App {
        let mut input = std::collections::HashMap::new();
        input.insert(InputMode::Url, "".to_string());
        input.insert(InputMode::Params, "".to_string());
        input.insert(InputMode::Method, "GET".to_string());
        input.insert(InputMode::Header, "".to_string());
        input.insert(InputMode::Body, "".to_string());

        let mut output = std::collections::HashMap::new();
        output.insert(OutputMessage::Header, "".to_string());
        output.insert(OutputMessage::Body, "".to_string());

        App {
            inputs: input,
            input_mode: InputMode::Url,
            outputs: output,
        }
    }
}

impl App {
    fn next(&mut self) {
        let mode = match self.input_mode {
            InputMode::Url => InputMode::Params,
            InputMode::Params => InputMode::Method,
            InputMode::Method => InputMode::Body,
            InputMode::Body => InputMode::Header,
            InputMode::Header => InputMode::Url,
        };
        self.input_mode = mode;
    }

    fn prev(&mut self) {
        let mode = match self.input_mode {
            InputMode::Url => InputMode::Header,
            InputMode::Header => InputMode::Body,
            InputMode::Body => InputMode::Method,
            InputMode::Method => InputMode::Params,
            InputMode::Params => InputMode::Url,
        };
        self.input_mode = mode;
    }

    fn input_push(&mut self, c: char) {
        if self.input_mode == InputMode::Method {
            return;
        }
        let mut input = self.inputs.get(&self.input_mode).unwrap().clone();
        input.push(c);
        self.inputs.insert(self.input_mode, input);
    }

    fn input_pop(&mut self) {
        if self.input_mode == InputMode::Method {
            return;
        }
        let mut input = self.inputs.get(&self.input_mode).unwrap().clone();
        input.pop();
        self.inputs.insert(self.input_mode, input);
    }

    fn change_method_right(&mut self) {
        if self.input_mode != InputMode::Method {
            return;
        }
        let input = self.inputs.get(&self.input_mode).unwrap().clone();
        let value = if input == "GET".to_string() {
            "POST"
        } else if input == "POST".to_string() {
            "PUT"
        } else if input == "PUT".to_string() {
            "DELETE"
        } else {
            "GET"
        }
        .to_string();
        self.inputs.insert(self.input_mode, value);
    }

    fn change_method_left(&mut self) {
        if self.input_mode != InputMode::Method {
            return;
        }
        let input = self.inputs.get(&self.input_mode).unwrap().clone();
        let value = if input == "GET".to_string() {
            "DELETE"
        } else if input == "DELETE".to_string() {
            "PUT"
        } else if input == "PUT".to_string() {
            "POST"
        } else {
            "GET"
        }
        .to_string();
        self.inputs.insert(self.input_mode, value);
    }

    fn request(&mut self) {
        let client = reqwest::blocking::Client::new();

        // let resp = reqwest::blocking::get("https://httpbin.org/ip");

        let method = self.inputs.get(&InputMode::Method).unwrap().clone();
        let url = self.inputs.get(&InputMode::Url).unwrap();

        let resp = if method == "GET".to_string() {
            client.get(url)
        } else if method == "DELETE".to_string() {
            client.delete(url)
        } else if method == "PUT".to_string() {
            client.put(url)
        } else {
            client.post(url)
        }.send();

        if let Ok(res) = resp {
            self.outputs
                .insert(OutputMessage::Header, format!("{:#?}", res.headers()));

            self.outputs
                .insert(OutputMessage::Body, res.text().unwrap_or("".to_string()));
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let events = Events::new();

    // Create default app state
    let mut app = App::default();

    loop {
        // Draw UI
        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(13), Constraint::Percentage(87)].as_ref())
                .split(size);

            let url_input = Paragraph::new(app.inputs.get(&InputMode::Url).unwrap().as_ref())
                .style(match app.input_mode {
                    InputMode::Url => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .block(Block::default().borders(Borders::ALL).title("Request URL"));
            f.render_widget(url_input, chunks[0]);

            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
                .split(chunks[1]);

            let request_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(25),
                        Constraint::Percentage(15),
                        Constraint::Percentage(25),
                        Constraint::Percentage(35),
                    ]
                    .as_ref(),
                )
                .split(bottom_chunks[0]);

            let params_input = Paragraph::new(app.inputs.get(&InputMode::Params).unwrap().as_ref())
                .style(match app.input_mode {
                    InputMode::Params => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .block(Block::default().borders(Borders::ALL).title("URL params"));
            f.render_widget(params_input, request_chunks[0]);

            let method_input = Paragraph::new(app.inputs.get(&InputMode::Method).unwrap().as_ref())
                .style(match app.input_mode {
                    InputMode::Method => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .block(Block::default().borders(Borders::ALL).title("Method"));
            f.render_widget(method_input, request_chunks[1]);

            let body_input = Paragraph::new(app.inputs.get(&InputMode::Body).unwrap().as_ref())
                .style(match app.input_mode {
                    InputMode::Body => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .block(Block::default().borders(Borders::ALL).title("Request Body"));
            f.render_widget(body_input, request_chunks[2]);

            let header_input = Paragraph::new(app.inputs.get(&InputMode::Header).unwrap().as_ref())
                .style(match app.input_mode {
                    InputMode::Header => Style::default().fg(Color::Yellow),
                    _ => Style::default(),
                })
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Request Headers"),
                );
            f.render_widget(header_input, request_chunks[3]);

            let response_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                .split(bottom_chunks[1]);

            let response_header =
                Paragraph::new(app.outputs.get(&OutputMessage::Header).unwrap().as_ref())
                    .style(Style::default())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Response Headers"),
                    );
            f.render_widget(response_header, response_chunks[0]);

            let response_body =
                Paragraph::new(app.outputs.get(&OutputMessage::Body).unwrap().as_ref())
                    .style(Style::default())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Response Body"),
                    );
            f.render_widget(response_body, response_chunks[1]);

            match app.input_mode {
                InputMode::Url => f.set_cursor(
                    chunks[0].x + app.inputs.get(&InputMode::Url).unwrap().width() as u16 + 1,
                    chunks[0].y + 1,
                ),
                InputMode::Params => f.set_cursor(
                    chunks[0].x + app.inputs.get(&InputMode::Params).unwrap().width() as u16 + 1,
                    chunks[0].y + 4,
                ),
                InputMode::Body => f.set_cursor(
                    chunks[0].x + app.inputs.get(&InputMode::Body).unwrap().width() as u16 + 1,
                    chunks[0].y + 13,
                ),
                InputMode::Header => f.set_cursor(
                    chunks[0].x + app.inputs.get(&InputMode::Header).unwrap().width() as u16 + 1,
                    chunks[0].y + 19,
                ),
                _ => {}
            }
        })?;

        // Handle input
        if let Event::Input(input) = events.next()? {
            match input {
                Key::Char('q') => {
                    break;
                }
                Key::Char('\t') => {
                    app.next();
                }
                Key::BackTab => {
                    app.prev();
                }
                Key::Down => {
                    app.next();
                }
                Key::Up => {
                    app.prev();
                }
                Key::Right => {
                    app.change_method_right();
                }
                Key::Left => {
                    app.change_method_left();
                }
                Key::Char('\n') => {
                    app.request();
                }
                Key::Char(c) => app.input_push(c),
                Key::Backspace => {
                    app.input_pop();
                }
                _ => {}
            }
        }
    }

    Ok(())
}
