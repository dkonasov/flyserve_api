extern crate chrono;

use std::collections::BTreeMap;
use self::chrono::prelude::*;

pub struct HttpResponse<'a> {
    pub version: String,
    pub status_code: u16,
    pub status_msg: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub body: Option<String>,
    pub to_send: bool,
    response_handlers: Vec<Box<FnMut(&HttpResponse) + 'a>>,
}

impl<'a> HttpResponse<'a> {
    pub fn new() -> HttpResponse<'a> {
        let mut headers = BTreeMap::new();
        let formated_date = Utc::now().format("%a %d %b %Y %H:%M:%S GMT").to_string();
        headers.insert(String::from("Date"), formated_date);
        headers.insert("Server".to_string(), "Flyserve".to_string());
        HttpResponse {
            version: String::from("1.1"),
            status_code: 200,
            status_msg: Some(String::from("OK")),
            headers: headers,
            body: None,
            to_send: false,
            response_handlers: Vec::new()
        }
    }
    pub fn set_response_handler(&mut self, handler: Box<FnMut(&HttpResponse) + 'a>) {
        self.response_handlers.push(handler);
    }
    pub fn send(&mut self) {
        let current_response = self.clone();
        for handler in self.response_handlers.iter_mut() {
            handler(&current_response);
        }
    }
}

impl<'a> Clone for HttpResponse<'a> {
    fn clone(&self) -> HttpResponse<'a> {
        HttpResponse {
            version: self.version.clone(),
            status_code: self.status_code.clone(),
            status_msg: self.status_msg.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
            to_send: self.to_send.clone(),
            response_handlers: Vec::new()
        }
    }
}

impl<'a> ToString for HttpResponse<'a> {
    fn to_string(&self) -> String {
        let mut status_text = "".to_owned();
        let mut headers = String::from("");
        let mut body = "".to_owned();
        if self.status_msg.is_some() {
            status_text = format!(" {}", self.status_msg.as_ref().unwrap());
        }
        for (key, value) in self.headers.iter() {
            headers = headers + &format!("\r\n{}: {}", key, value);
        }
        if self.body.is_some() {
            body = format!("\r\n\r\n{}", self.body.as_ref().unwrap());
        }
        let result = format!(
            "HTTP/{} {}{}\
            {}\
            {}\
            ",
            self.version,
            self.status_code,
            status_text,
            headers,
            body
            );
        return result;
    }
}