extern crate regex;
extern crate percent_encoding;
extern crate chrono;
use percent_encoding::percent_decode;
use std::collections::{BTreeMap, HashMap};
use chrono::prelude::*;
use regex::Regex;

pub struct Path {
    pub segments: Vec<String>,
    striped_str: String
}
impl Path {
    pub fn parse(raw: &str) -> Path {
        let strip_regex = Regex::new(r"^/?(.*)/?$").unwrap();
        let striped_str = strip_regex.replace(raw, "${1}").into_owned();
        let path = Path {
            segments: striped_str.split("/").map(|str| str.to_string()).collect(),
            striped_str: striped_str
        };
        return path;
    }
    pub fn compare(&self, template: &str) -> Option<HashMap<String, String>> {
        let mut params: HashMap<String, String> = HashMap::new();
        let strip_regex = Regex::new(r"^/?(.*)/?$").unwrap();
        let striped_str = strip_regex.replace(template, "${1}").into_owned();
        let template_regex = Regex::new(&striped_str).unwrap();
        if template_regex.is_match(&self.striped_str) {
            let captures = template_regex.captures(&self.striped_str);
            if captures.is_some() {
                let captures = captures.unwrap();
                for name in template_regex.capture_names() {
                if name.is_some() {
                    let name = name.unwrap();
                    let value = captures.name(name);
                    if value.is_some() {
                        params.insert(name.to_string(), value.unwrap().as_str().to_string());
                    }
                }
            }
            }
            return Some(params);
        }
        return None;
    }
}

pub struct HttpRequest {
    pub method: String,
    pub path: Path,
    pub query: BTreeMap<String, String>,
    pub version: String,
    pub headers: BTreeMap<String, String>,
    pub params: BTreeMap<String, String>,
    pub body: String
}
impl HttpRequest {
    pub fn parse(raw: &str) -> Result<HttpRequest, &str> {
        let lines: Vec<&str> = raw.lines().collect();
        let mut body = "".to_string();
        if lines.len() < 2 {
            return Err("Unable to parse request");
        }
        let info: Vec<&str> = lines[0].split_whitespace().collect();
        if info.len() < 3 {
            return Err("Unable to parse request");
        }
        let method = info[0];
        let path_str: Vec<&str> = info[1].split("?").collect();
        if path_str.len() < 1 {
            return Err("Unable to parse request");
        }
        let mut query = BTreeMap::new();
        if path_str.len() > 1 {
            let query_vec: Vec<&str> = path_str[1].split("&").collect();
            for query_elem_str in query_vec {
                let query_elem_vec: Vec<&str> = query_elem_str.split("=").collect();
                if query_elem_vec.len() < 2 || query_elem_vec[0] == "" {
                    return Err("Unable to parse request");
                }
                let mut value = "";
                if query_elem_vec.len() > 1 && query_elem_vec[1] != "" {
                    value = &query_elem_vec[1];
                }
                query.insert(percent_decode(query_elem_vec[0].as_bytes()).decode_utf8_lossy().into_owned(), percent_decode(value.as_bytes()).decode_utf8_lossy().into_owned());
            }
        }
        let protocol_info_vec: Vec<&str> = info[2].split("/").collect();
        if protocol_info_vec.len() < 2 || protocol_info_vec[0].to_uppercase() != "HTTP" ||  protocol_info_vec[1] == "" {
            return Err("Unable to parse request");
        }
        let mut cursor = 1;
        if lines[cursor] == "\r\n" {
            return Err("Unable to parse request: no headers was presented");
        }
        let mut headers = BTreeMap::new();
        while cursor < lines.len() && lines[cursor] != ""  {
            let mut headers_elem_vec: Vec<&str> = lines[cursor].split(":").collect();
            if headers_elem_vec.len() < 2 {
                return Err("Unable to parse request: invalid header");
            }
            let header_name = headers_elem_vec[0].trim().to_string();
            headers_elem_vec.remove(0);
            let header_value = headers_elem_vec.join(":").trim().to_string();
            if header_name.len() == 0 || header_value.len() == 0 {
                return Err("Unable to parse request: invalid header");
            }
            headers.insert(header_name, header_value);
            cursor += 1;
        }
        if (lines.len() - cursor) > 1 {
            cursor += 1;
            for i in cursor..lines.len() {
                body = body.to_owned() + lines[i];
            }
        }
        Ok(HttpRequest {
            method: String::from(method).to_uppercase(),
            path: Path::parse(path_str[0]),
            query: query,
            version: String::from(protocol_info_vec[1]),
            headers: headers,
            body: body.to_string(),
            params: BTreeMap::new()
        })
    }
}
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
        println!("Handlers len: " + self.response_handlers.len());
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

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn path_parse_and_strip() {
        let path = Path::parse("/foo/bar");
        assert_eq!(path.segments.len(), 2);
        assert_eq!(path.segments[0], "foo");
        assert_eq!(path.segments[1], "bar");
    }

    #[test]
    fn parse_http_request() {
        let request = HttpRequest::parse(
            "POST /foo/bar?foo=bar&foo%3F=bar%26 HTTP/1.1\r\n\
            Host: example.com\r\n\
            User-Agent: Mozilla/5.0 (X11; U; Linux i686; ru; rv:1.9b5) Gecko/2008050509 Firefox/3.0b5\r\n\
            Accept: text/html\r\n\
            \r\n\
            Lorem ipsum"
        );
        let request = request.unwrap();
        assert_eq!(request.method, "POST");
        assert_eq!(request.path.segments.len(), 2);
        assert_eq!(request.query.get("foo"), Some(&String::from("bar")));
        assert_eq!(request.query.get("foo?"), Some(&String::from("bar&")));
        assert_eq!(request.version, "1.1");
        assert_eq!(request.headers.get("Host"), Some(&String::from("example.com")));
        assert_eq!(request.headers.get("User-Agent"), Some(&String::from("Mozilla/5.0 (X11; U; Linux i686; ru; rv:1.9b5) Gecko/2008050509 Firefox/3.0b5")));
        assert_eq!(request.headers.get("Accept"), Some(&String::from("text/html")));
        assert_eq!(request.body, "Lorem ipsum");
    }

    #[test]
    fn stringify_default_http_response() {
        let response = HttpResponse::new();
        let test_regex = Regex::new(r"^HTTP/1\.1 200 OK\r\nDate: (Mon|Tue|Wed|Thu|Fri|Sat|Sun) [0-3][0-9] (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) [1-9][0-9][0-9][0-9] [0-2][0-9]:[0-5][0-9]:[0-5][0-9] GMT\r\nServer: Flyserve$").unwrap();
        assert!(test_regex.is_match(&response.to_string()));
    }

    #[test]
    fn stringify_http_response() {
        let mut response = HttpResponse::new();
        response.headers.insert(String::from("Content-type"), String::from("text/plain"));
        response.body = Some(String::from("Hello, world!"));
        let test_regex_for_header = Regex::new(r"\r\nContent-type: text/plain").unwrap();
        let test_regex_for_body = Regex::new(r"\r\n\r\nHello, world!$").unwrap();
        assert!(test_regex_for_header.is_match(&response.to_string()));
        assert!(test_regex_for_body.is_match(&response.to_string()));
    }

    #[test]
    fn compare_blank_paths() {
        let path = Path::parse("");
        assert!(path.compare("").is_some());
    }
    #[test]
    fn compare_simple_regex() {
        let path = Path::parse("/foo");
        assert!(path.compare("/.*").is_some());
    }
    #[test]
    fn compare_simple_negative() {
        let path = Path::parse("/foo");
        assert!(path.compare("/.*/bar").is_none());
    }
    #[test]
    fn compare_read_param() {
        let path = Path::parse("/user/42");
        let compare_result = path.compare("/user/(?P<id>.*)");
        assert!(compare_result.is_some());
        assert_eq!(compare_result.unwrap().get("id"), Some(&"42".to_string()));
    }

    #[test]
    fn send_response() {
        let mut response_sent = false;
        {
            let mut response = HttpResponse::new();
            response.set_response_handler(Box::new(|res| { response_sent = true; }));
            response.send();
        }
        assert!(response_sent);
    }
}
