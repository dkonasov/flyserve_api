extern crate percent_encoding;

use self::percent_encoding::percent_decode;
use path::Path;
use std::collections::BTreeMap;

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