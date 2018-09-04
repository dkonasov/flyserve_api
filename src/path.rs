extern crate regex;

use regex::Regex;
use std::collections::HashMap;

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