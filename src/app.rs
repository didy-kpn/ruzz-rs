// use unicode_width::UnicodeWidthStr;

use tui::widgets::ListState;
use std::collections::BTreeMap;
use reqwest::header::HeaderMap;

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
pub struct ListMethod {
    state: ListState,
    items: Vec<String>,
    value: HttpMethod,
}

impl ListMethod {
    pub fn new() -> ListMethod {
        let mut state = ListState::default();
        state.select(Some(0));
        ListMethod {
            state: state,
            items: vec![
                "GET".to_string(),
                "HEAD".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "CONNECT".to_string(),
                "OPTIONS".to_string(),
                "TRACE".to_string(),
                "PATCH".to_string(),
            ],
            value: HttpMethod::GET,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.value = match self.value {
            HttpMethod::GET => HttpMethod::HEAD,
            HttpMethod::HEAD => HttpMethod::POST,
            HttpMethod::POST => HttpMethod::PUT,
            HttpMethod::PUT => HttpMethod::DELETE,
            HttpMethod::DELETE => HttpMethod::CONNECT,
            HttpMethod::CONNECT => HttpMethod::OPTIONS,
            HttpMethod::OPTIONS => HttpMethod::TRACE,
            HttpMethod::TRACE => HttpMethod::PATCH,
            HttpMethod::PATCH => HttpMethod::GET,
        };
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.value = match self.value {
            HttpMethod::GET => HttpMethod::PATCH,
            HttpMethod::HEAD => HttpMethod::GET,
            HttpMethod::POST => HttpMethod::HEAD,
            HttpMethod::PUT => HttpMethod::POST,
            HttpMethod::DELETE => HttpMethod::PUT,
            HttpMethod::CONNECT => HttpMethod::DELETE,
            HttpMethod::OPTIONS => HttpMethod::CONNECT,
            HttpMethod::TRACE => HttpMethod::OPTIONS,
            HttpMethod::PATCH => HttpMethod::TRACE,
        };
    }
}

#[derive(Clone)]
struct Request {
    url: EditView,
    params: EditView,
    header: EditView,
    body: EditView,
    method: ListMethod,
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
                method: ListMethod::new(),
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

    pub fn request_method_items_vec(&self) -> &Vec<String> {
        &self.request.method.items
    }

    pub fn request_method_state(&self) -> &ListState {
        &self.request.method.state
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

    pub fn request_params_map(&self) -> BTreeMap<String, String> {
        let request_params_text = self.request_params_text();
        let request_params_text = request_params_text.replace("\n", "&");

        let mut params = BTreeMap::new();
        for query_str in request_params_text.split('&') {
            let q:Vec<String> = query_str.split('=').map(|s| s.to_string()).collect();
            let key = q.get(0).unwrap_or(&"".to_string()).to_string();
            let value = q.get(1).unwrap_or(&"".to_string()).to_string();
            if key.len() == 0 && value.len() == 0 {
                continue;
            }
            params.insert(key, value);
        }

        params
    }

    pub fn request_header_map(&self) -> HeaderMap {
        let request_header_text = self.request_header_text();

        let mut headers = HeaderMap::new();
        for header_str in request_header_text.split('\n') {
            let h:Vec<String> = header_str.split(':').map(|s| s.to_string()).collect();
            let key = h.get(0).unwrap_or(&"".to_string()).to_string();
            let value = h.get(1).unwrap_or(&"".to_string()).to_string();

            if key == "" || value == "" {
                break;
            }

            let key = Box::leak(key.into_boxed_str());

            headers.entry(&*key).or_insert(value.parse().unwrap());

            // if let Entry::Vacant(v) = headers.entry("x-hello") {
            //     let mut e = v.insert_entry("world".parse().unwrap());
            //     e.insert("world2".parse().unwrap());
            // }

            // let key = *key;
            // let key = HeaderName::from_str(key);

            // let value = h.get(1).unwrap_or(&"");
            // let value = HeaderValue::from_str(*value);
            // headers.append(key, value);
        }
        headers
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

    pub fn is_request_url_edit(&self) -> bool {
        self.mode.edit == EditMode::RequestUrl
    }

    pub fn is_request_method_edit(&self) -> bool {
        self.mode.edit == EditMode::RequestMethod
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

    fn reqwest_method(&self) -> reqwest::Method {
        match self.request.method.value {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::HEAD => reqwest::Method::HEAD,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::CONNECT => reqwest::Method::CONNECT,
            HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
            HttpMethod::TRACE => reqwest::Method::TRACE,
            HttpMethod::PATCH => reqwest::Method::PATCH,
        }
    }

    pub fn request(&mut self) {
        let client = reqwest::blocking::Client::new();

        let url = self.request_url_text();
        let params = self.request_params_map();
        let headers = self.request_header_map();
        let body = self.request_body_text().to_string();

        let response = client.request(self.reqwest_method(), url)
        .query(&params)
        .headers(headers)
        .body(body)
        .send();

        if let Ok(resp) = response {
            self.response.status.text = format!("{:?}", resp.status());
            self.response.header.text = format!("{:#?}", resp.headers());
            self.response.body.text = resp.text().unwrap_or("".to_string());
        }
    }

    pub fn next_select_on_request_method(&mut self) {
        self.request.method.next();
    }

    pub fn prev_select_on_request_method(&mut self) {
        self.request.method.previous();
    }
}
