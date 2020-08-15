// use unicode_width::UnicodeWidthStr;

#[derive(Clone)]
struct Cursor {
    x: u16,
    y: u16,
}

#[derive(Clone)]
struct EditView {
    text: String,
    cursor: Cursor,
}

impl EditView {
    fn new() -> EditView {
        EditView {
            text: "".to_string(),
            cursor: Cursor { x: 0, y: 0 },
        }
    }
}

#[derive(Clone)]
enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

#[derive(Clone)]
struct Request {
    url: EditView,
    params: EditView,
    header: EditView,
    body: EditView,
    method: HttpMethod,
}

#[derive(Clone)]
struct Response {
    status: EditView,
    header: EditView,
    body: EditView,
}

#[derive(Clone)]
pub enum ViewMode {
    RequestUrl,
    RequestParams,
    RequestMethod,
    RequestHeader,
    RequestBody,

    ResponseHeader,
    ResponseBody,
}

#[derive(PartialEq, Eq, Clone)]
pub enum EditMode {
    NoMode,
    RequestUrl,
    RequestParams,
    RequestMethod,
    RequestHeader,
    RequestBody,
}

#[derive(Clone)]
struct Mode {
    view: ViewMode,
    edit: EditMode,
}

#[derive(Clone)]
pub struct App {
    request: Request,
    response: Response,
    mode: Mode,
}

impl App {
    pub fn default() -> App {
        App {
            request: Request {
                url: EditView::new(),
                params: EditView::new(),
                header: EditView::new(),
                body: EditView::new(),
                method: HttpMethod::GET,
            },
            response: Response {
                status: EditView::new(),
                header: EditView::new(),
                body: EditView::new(),
            },
            mode: Mode {
                view: ViewMode::RequestUrl,
                edit: EditMode::NoMode,
            },
        }
    }

    pub fn request_url_text(&self) -> &String {
        &self.request.url.text
    }

    pub fn request_params_text(&self) -> &String {
        &self.request.params.text
    }

    pub fn request_header_text(&self) -> &String {
        &self.request.header.text
    }

    pub fn request_body_text(&self) -> &String {
        &self.request.body.text
    }

    pub fn response_status_text(&self) -> &String {
        &self.response.status.text
    }

    pub fn response_header_text(&self) -> &String {
        &self.response.header.text
    }

    pub fn response_body_text(&self) -> &String {
        &self.response.body.text
    }

    pub fn request_url_cursor_x(&self) -> &u16 {
        &self.request.url.cursor.x
    }

    pub fn request_params_cursor_x(&self) -> &u16 {
        &self.request.params.cursor.x
    }

    pub fn request_header_cursor_x(&self) -> &u16 {
        &self.request.header.cursor.x
    }

    pub fn request_body_cursor_x(&self) -> &u16 {
        &self.request.body.cursor.x
    }

    pub fn view_mode(&self) -> &ViewMode {
        &self.mode.view
    }

    pub fn edit_mode(&self) -> &EditMode {
        &self.mode.edit
    }

    pub fn is_mode_view(&self) -> bool {
        self.mode.edit == EditMode::NoMode
    }

    pub fn is_mode_edit(&self) -> bool {
        !self.is_mode_view()
    }

    pub fn next_view(&mut self) {
        self.change_view_mode();
        self.mode.view = match self.mode.view {
            ViewMode::RequestUrl => ViewMode::RequestParams,
            ViewMode::RequestParams => ViewMode::RequestMethod,
            ViewMode::RequestMethod => ViewMode::RequestHeader,
            ViewMode::RequestHeader => ViewMode::RequestBody,
            ViewMode::RequestBody => ViewMode::ResponseHeader,

            ViewMode::ResponseHeader => ViewMode::ResponseBody,
            ViewMode::ResponseBody => ViewMode::RequestUrl,
        };
    }

    pub fn prev_view(&mut self) {
        self.change_view_mode();
        self.mode.view = match self.mode.view {
            ViewMode::ResponseBody => ViewMode::ResponseHeader,
            ViewMode::ResponseHeader => ViewMode::RequestBody,

            ViewMode::RequestBody => ViewMode::RequestHeader,
            ViewMode::RequestHeader => ViewMode::RequestMethod,
            ViewMode::RequestMethod => ViewMode::RequestParams,
            ViewMode::RequestParams => ViewMode::RequestUrl,
            ViewMode::RequestUrl => ViewMode::ResponseBody,
        };
    }

    pub fn change_edit_mode(&mut self) {
        self.mode.edit = match self.mode.view {
            ViewMode::RequestUrl => EditMode::RequestUrl,
            ViewMode::RequestParams => EditMode::RequestParams,
            ViewMode::RequestMethod => EditMode::RequestMethod,
            ViewMode::RequestHeader => EditMode::RequestHeader,
            ViewMode::RequestBody => EditMode::RequestBody,
            _ => EditMode::NoMode,
        };
    }

    pub fn change_view_mode(&mut self) {
        self.mode.edit = EditMode::NoMode;
    }

    fn jump_view(&mut self, view: ViewMode) {
        self.change_view_mode();
        self.mode.view = view;
    }

    pub fn jump_request_url(&mut self) {
        self.jump_view(ViewMode::RequestUrl);
    }

    pub fn jump_request_params(&mut self) {
        self.jump_view(ViewMode::RequestParams);
    }

    pub fn jump_request_method(&mut self) {
        self.jump_view(ViewMode::RequestMethod);
    }

    pub fn jump_request_header(&mut self) {
        self.jump_view(ViewMode::RequestHeader);
    }

    pub fn jump_request_body(&mut self) {
        self.jump_view(ViewMode::RequestBody);
    }

    pub fn jump_response_header(&mut self) {
        self.jump_view(ViewMode::ResponseHeader);
    }

    pub fn jump_response_body(&mut self) {
        self.jump_view(ViewMode::ResponseBody);
    }

    pub fn insert_text(&mut self, c: char) {
        match self.mode.edit {
            EditMode::RequestUrl => {
                self.request
                    .url
                    .text
                    .insert(self.request.url.cursor.x as usize, c);
                self.request.url.cursor.x += 1;
            }
            EditMode::RequestParams => {
                self.request
                    .params
                    .text
                    .insert(self.request.params.cursor.x as usize, c);
                self.request.params.cursor.x += 1;
            }
            EditMode::RequestHeader => {
                self.request
                    .header
                    .text
                    .insert(self.request.header.cursor.x as usize, c);
                self.request.header.cursor.x += 1;
            }
            EditMode::RequestBody => {
                self.request
                    .body
                    .text
                    .insert(self.request.body.cursor.x as usize, c);
                self.request.body.cursor.x += 1;
            }
            _ => {}
        }
    }

    // TODO: 2文字以上の移動+削除が想定外の動きをする
    // pub fn right_move_cursor(&mut self) {
    //     match self.mode.edit {
    //         EditMode::RequestUrl => {
    //             if self.request.url.cursor.x < self.request.url.text.width() as u16 {
    //                 self.request.url.cursor.x += 1;
    //             }
    //         }
    //         EditMode::RequestParams => {
    //             if self.request.params.cursor.x < self.request.params.text.width() as u16 {
    //                 self.request.params.cursor.x += 1;
    //             }
    //         }
    //         EditMode::RequestHeader => {
    //             if self.request.header.cursor.x < self.request.header.text.width() as u16 {
    //                 self.request.header.cursor.x += 1;
    //             }
    //         }
    //         EditMode::RequestBody => {
    //             if self.request.body.cursor.x < self.request.body.text.width() as u16 {
    //                 self.request.body.cursor.x += 1;
    //             }
    //         }
    //         _ => {}
    //     }
    // }

    // pub fn left_move_cursor(&mut self) {
    //     match self.mode.edit {
    //         EditMode::RequestUrl => {
    //             if 0 < self.request.url.cursor.x {
    //                 self.request.url.cursor.x -= 1;
    //             }
    //         }
    //         EditMode::RequestParams => {
    //             if 0 < self.request.params.cursor.x {
    //                 self.request.params.cursor.x -= 1;
    //             }
    //         }
    //         EditMode::RequestHeader => {
    //             if 0 < self.request.header.cursor.x {
    //                 self.request.header.cursor.x -= 1;
    //             }
    //         }
    //         EditMode::RequestBody => {
    //             if 0 < self.request.body.cursor.x {
    //                 self.request.body.cursor.x -= 1;
    //             }
    //         }
    //         _ => {}
    //     }
    // }

    pub fn delete_text(&mut self) {
        match self.mode.edit {
            EditMode::RequestUrl => {
                if self.request.url.text.len() == 0 || self.request.url.cursor.x == 0 {
                    return;
                }

                self.request
                    .url
                    .text
                    .remove(self.request.url.cursor.x as usize - 1);

                if self.request.url.cursor.x < self.request.url.text.len() as u16 {
                    return;
                }

                self.request.url.cursor.x -= 1;
            }
            EditMode::RequestParams => {
                if self.request.params.text.len() == 0 || self.request.params.cursor.x == 0 {
                    return;
                }

                self.request
                    .params
                    .text
                    .remove(self.request.params.cursor.x as usize - 1);

                if self.request.params.cursor.x < self.request.params.text.len() as u16 {
                    return;
                }

                self.request.params.cursor.x -= 1;
            }
            EditMode::RequestHeader => {
                if self.request.header.text.len() == 0 || self.request.header.cursor.x == 0 {
                    return;
                }

                self.request
                    .header
                    .text
                    .remove(self.request.header.cursor.x as usize - 1);

                if self.request.header.cursor.x < self.request.params.text.len() as u16 {
                    return;
                }

                self.request.header.cursor.x -= 1;
            }
            EditMode::RequestBody => {
                if self.request.body.text.len() == 0 || self.request.body.cursor.x == 0 {
                    return;
                }

                self.request
                    .body
                    .text
                    .remove(self.request.body.cursor.x as usize - 1);

                if self.request.body.cursor.x < self.request.params.text.len() as u16 {
                    return;
                }

                self.request.body.cursor.x -= 1;
            }
            _ => {}
        }
    }

    pub fn request(&mut self) {
        let client = reqwest::blocking::Client::new();

        let url = self.request_url_text();

        let response = match self.request.method {
            HttpMethod::GET => client.get(url),
            HttpMethod::POST => client.post(url),
            HttpMethod::PUT => client.put(url),
            HttpMethod::DELETE => client.delete(url),
            _ => client.get(url),
        }
        .send();

        if let Ok(resp) = response {
            self.response.status.text = format!("{:?}", resp.status());
            self.response.header.text = format!("{:#?}", resp.headers());
            self.response.body.text = resp.text().unwrap_or("".to_string());
        }
    }
}
