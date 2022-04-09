use super::app;

use std::io::Stdout;

use termion::{input::MouseTerminal, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

// use unicode_width::UnicodeWidthStr;

pub fn draw(
    terminal: &mut Terminal<
        TermionBackend<AlternateScreen<MouseTerminal<termion::raw::RawTerminal<Stdout>>>>,
    >,
    app: &app::App,
) {
    let request_url_text = app.request_url_text();
    let request_params_text = app.request_params_text();
    let request_header_text = app.request_header_text();
    let request_body_text = app.request_body_text();

    let response_status_text = app.response_status_text();
    let response_header_text = app.response_header_text();
    let response_body_text = app.response_body_text();

    let view = app.view_mode();
    let edit = app.edit_mode();

    let mut request_method_state = app.request_method_state().clone();

    let _ = terminal.draw(|f| {
        let size = f.size();

        // 上下レイアウト
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(13), Constraint::Percentage(87)].as_ref())
            .split(size);

        // 下の左右のレイアウト
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)].as_ref())
            .split(main_chunks[1]);

        // 左右の左のレイアウト
        let request_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(25),
                    Constraint::Percentage(15),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ]
                .as_ref(),
            )
            .split(bottom_chunks[0]);

        // 左右の右のレイアウト
        let response_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(13),
                    Constraint::Percentage(40),
                    Constraint::Percentage(47),
                ]
                .as_ref(),
            )
            .split(bottom_chunks[1]);

        // Request URL
        let request_url = Paragraph::new(request_url_text.as_ref())
            .style(match view {
                app::ViewMode::RequestUrl => {
                    if *edit == app::EditMode::RequestUrl {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    }
                }
                _ => Style::default(),
            })
            .block(Block::default().borders(Borders::ALL).title("Request URL"));
        f.render_widget(request_url, main_chunks[0]);

        // Request Params
        let request_params = Paragraph::new(request_params_text.as_ref())
            .style(match view {
                app::ViewMode::RequestParams => {
                    if *edit == app::EditMode::RequestParams {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    }
                }
                _ => Style::default(),
            })
            .block(Block::default().borders(Borders::ALL).title("URL params"));
        f.render_widget(request_params, request_chunks[0]);

        // Request Method
        let items: Vec<ListItem> = app
            .request_method_items_vec()
            .iter()
            .map(|i| {
                let lines = vec![Spans::from(i.to_string())];
                ListItem::new(lines).style(Style::default())
            })
            .collect();
        let items = List::new(items)
            .style(match view {
                app::ViewMode::RequestMethod => {
                    if *edit == app::EditMode::RequestMethod {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    }
                }
                _ => Style::default(),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Request Method"),
            )
            .highlight_symbol(">> ");
        f.render_stateful_widget(items, request_chunks[1], &mut request_method_state);

        // Request Header
        let request_header = Paragraph::new(request_header_text.as_ref())
            .style(match view {
                app::ViewMode::RequestHeader => {
                    if *edit == app::EditMode::RequestHeader {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    }
                }
                _ => Style::default(),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Request Header"),
            );
        f.render_widget(request_header, request_chunks[2]);

        // Request Body
        let request_body = Paragraph::new(request_body_text.as_ref())
            .style(match view {
                app::ViewMode::RequestBody => {
                    if *edit == app::EditMode::RequestBody {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    }
                }
                _ => Style::default(),
            })
            .block(Block::default().borders(Borders::ALL).title("Request Body"));
        f.render_widget(request_body, request_chunks[3]);

        // Request Status
        let response_status = Paragraph::new(response_status_text.as_ref())
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Response Status"),
            );
        f.render_widget(response_status, response_chunks[0]);

        // Response Header
        let response_header = Paragraph::new(response_header_text.as_ref())
            .style(match view {
                app::ViewMode::ResponseHeader => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Response Header"),
            );
        f.render_widget(response_header, response_chunks[1]);

        // Response Body
        let response_body = Paragraph::new(response_body_text.as_ref())
            .style(match view {
                app::ViewMode::ResponseBody => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Response Body"),
            );
        f.render_widget(response_body, response_chunks[2]);

        match app.edit_mode() {
            app::EditMode::RequestUrl => f.set_cursor(
                main_chunks[0].x + app.request_url_cursor_x() + 1,
                main_chunks[0].y + 1,
            ),
            app::EditMode::RequestParams => f.set_cursor(
                request_chunks[0].x + app.request_params_cursor_x() + 1,
                request_chunks[0].y + 1,
            ),
            app::EditMode::RequestHeader => f.set_cursor(
                request_chunks[2].x + app.request_header_cursor_x() + 1,
                request_chunks[2].y + 1,
            ),
            app::EditMode::RequestBody => f.set_cursor(
                request_chunks[3].x + app.request_body_cursor_x() + 1,
                request_chunks[3].y + 1,
            ),
            _ => {}
        }
    });
}
