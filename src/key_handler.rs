use super::app;

use termion::event::Key;

// view状態の入力ハンドラー
pub fn view_input_event(input: &Key, app: &mut app::App) -> bool {
    match input {
        Key::Char('q') => {
            return false;
        }
        Key::Char('\t') => {
            app.next_view();
        }
        Key::BackTab => {
            app.prev_view();
        }
        Key::Char('i') => {
            app.change_edit_mode();
        }

        Key::F(1) => app.jump_request_url(),

        Key::F(2) => app.jump_request_params(),

        Key::F(3) => app.jump_request_method(),

        Key::F(4) => app.jump_request_header(),

        Key::F(5) => app.jump_request_body(),

        Key::F(6) => app.jump_response_header(),

        Key::F(7) => app.jump_response_body(),

        Key::Char('\n') => app.request(),

        _ => {}
    }

    return true;
}

// edit状態の入力ハンドラー
pub fn edit_input_event(input: &Key, app: &mut app::App) {
    match input {
        Key::Char('\n') => {
            if app.is_request_url_edit() || app.is_request_method_edit() {
                app.request();
            } else {
                app.insert_text('\n');
            }
        }
        Key::Ctrl('r') => {
            app.request();
        }
        Key::Esc => {
            app.change_view_mode();
        }
        Key::Right => {
            if app.is_request_method_edit() {
                app.next_select_on_request_method();
            }
            // TODO: 2文字以上の移動+削除が想定外の動きをする
            // app.right_move_cursor();
        }
        Key::Left => {
            if app.is_request_method_edit() {
                app.prev_select_on_request_method();
            }
            // TODO: 2文字以上の移動+削除が想定外の動きをする
            // app.left_move_cursor();
        }
        Key::Backspace => {
            app.delete_text();
        }

        Key::Char(c) => {
            if c.is_ascii_graphic() {
                app.insert_text(*c)
            }
        }
        _ => {}
    }
}
