extern crate regex;

mod path;
mod http_request;
mod http_response;

pub use path::Path;
pub use http_request::HttpRequest;
pub use http_response::HttpResponse;

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
