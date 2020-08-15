mod app;
mod event;
mod key_handler;
mod ui;

use std::{error::Error, io};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

fn main() -> Result<(), Box<dyn Error>> {
    // ターミナル初期化
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // イベントハンドラーのセットアップ
    let events = event::Events::new();

    // エディタ等の初期情報
    let mut app = app::App::default();

    loop {
        // UIを描画する
        ui::draw(&mut terminal, &app);

        // キー入力のハンドラー
        if let event::Event::Input(input) = events.next()? {
            // editモードのキー入力
            if app.is_mode_edit() {
                key_handler::edit_input_event(&input, &mut app)
            }

            // viewモードのキー入力
            if app.is_mode_view() {
                let result = key_handler::view_input_event(&input, &mut app);
                if result == false {
                    break;
                }
            }
        }
    }

    Ok(())
}
