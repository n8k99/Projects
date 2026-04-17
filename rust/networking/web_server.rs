// Small Web Server — HTTP server with routing, middleware, and static file serving.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get, Post, Put, Delete, Head, Options,
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Self::Get, "POST" => Self::Post, "PUT" => Self::Put,
            "DELETE" => Self::Delete, "HEAD" => Self::Head, "OPTIONS" => Self::Options,
            _ => Self::Get,
        }
    }
    pub fn name(&self) -> &'static str {
        match self { Self::Get => "GET", Self::Post => "POST", Self::Put => "PUT",
                     Self::Delete => "DELETE", Self::Head => "HEAD", Self::Options => "OPTIONS" }
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: String,
    pub remote_addr: String,
}

impl HttpRequest {
    pub fn parse(raw: &str) -> Self {
        let lines: Vec<&str> = if raw.contains("\r\n") {
            raw.split("\r\n").collect()
        } else {
            raw.split('\n').collect()
        };
        let first = lines.first().unwrap_or(&"GET / HTTP/1.1");
        let parts: Vec<&str> = first.splitn(3, ' ').collect();
        let method = HttpMethod::from_str(parts.first().unwrap_or(&"GET"));
        let full_path = parts.get(1).unwrap_or(&"/").to_string();
        let (path, query_params) = if let Some(idx) = full_path.find('?') {
            let p = full_path[..idx].to_string();
            let qs = &full_path[idx + 1..];
            let mut qp = HashMap::new();
            for pair in qs.split('&') {
                if let Some(eq) = pair.find('=') {
                    qp.insert(pair[..eq].to_string(), pair[eq + 1..].to_string());
                }
            }
            (p, qp)
        } else {
            (full_path, HashMap::new())
        };
        let mut headers = HashMap::new();
        let mut body_start = lines.len();
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }
            if let Some(colon) = line.find(':') {
                headers.insert(
                    line[..colon].trim().to_lowercase(),
                    line[colon + 1..].trim().to_string(),
                );
            }
        }
        let body = if body_start < lines.len() {
            lines[body_start..].join("\n")
        } else {
            String::new()
        };
        HttpRequest { method, path, headers, query_params, body, remote_addr: String::new() }
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn new(code: u16, text: &str, body: &str) -> Self {
        HttpResponse {
            status_code: code, status_text: text.to_string(),
            headers: HashMap::new(), body: body.to_string(),
        }
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn serialize(&mut self) -> String {
        self.headers.entry("Content-Length".to_string())
            .or_insert_with(|| self.body.len().to_string());
        self.headers.entry("Content-Type".to_string())
            .or_insert_with(|| "text/html; charset=utf-8".to_string());
        let header_lines: Vec<String> = self.headers.iter()
            .map(|(k, v)| format!("{}: {}", k, v)).collect();
        format!("HTTP/1.1 {} {}\r\n{}\r\n\r\n{}",
                self.status_code, self.status_text, header_lines.join("\r\n"), self.body)
    }
}

pub fn response_200(body: &str) -> HttpResponse {
    let mut r = HttpResponse::new(200, "OK", body);
    r.set_header("Content-Type", "text/html; charset=utf-8");
    r
}

pub fn response_404(path: &str) -> HttpResponse {
    HttpResponse::new(404, "Not Found", &format!("404 Not Found: {}", path))
}

pub fn response_json(data: &str) -> HttpResponse {
    let mut r = HttpResponse::new(200, "OK", data);
    r.set_header("Content-Type", "application/json");
    r
}

pub fn response_redirect(location: &str, permanent: bool) -> HttpResponse {
    let (code, text) = if permanent { (301, "Moved Permanently") } else { (302, "Found") };
    let mut r = HttpResponse::new(code, text, "");
    r.set_header("Location", location);
    r
}

type HandlerFn = Box<dyn Fn(&HttpRequest) -> HttpResponse>;

struct RouteEntry {
    method: HttpMethod,
    path: String,
    handler: HandlerFn,
}

pub struct WebServer {
    pub name: String,
    routes: Vec<RouteEntry>,
    static_files: HashMap<String, (String, String)>,
    pub access_log: Vec<(String, String, u16)>, // (method, path, status)
}

impl WebServer {
    pub fn new(name: &str) -> Self {
        WebServer {
            name: name.to_string(), routes: Vec::new(),
            static_files: HashMap::new(), access_log: Vec::new(),
        }
    }

    pub fn add_route<F>(&mut self, method: HttpMethod, path: &str, handler: F)
    where F: Fn(&HttpRequest) -> HttpResponse + 'static {
        self.routes.push(RouteEntry {
            method, path: path.to_string(), handler: Box::new(handler),
        });
    }

    pub fn serve_static(&mut self, path: &str, content: &str, content_type: &str) {
        self.static_files.insert(path.to_string(), (content.to_string(), content_type.to_string()));
    }

    fn resolve(&self, method: HttpMethod, path: &str) -> Option<&HandlerFn> {
        // Exact match
        for route in &self.routes {
            if route.method == method && route.path == path {
                return Some(&route.handler);
            }
        }
        // Prefix match
        for route in &self.routes {
            if route.method == method && route.path.ends_with("/*") &&
               path.starts_with(&route.path[..route.path.len() - 2]) {
                return Some(&route.handler);
            }
        }
        None
    }

    pub fn handle_request(&mut self, request: &HttpRequest) -> HttpResponse {
        // Check static files first
        if request.method == HttpMethod::Get {
            if let Some((content, ct)) = self.static_files.get(&request.path) {
                let mut r = HttpResponse::new(200, "OK", content);
                r.set_header("Content-Type", ct);
                self.access_log.push((request.method.name().to_string(), request.path.clone(), 200));
                return r;
            }
        }
        let response = match self.resolve(request.method, &request.path) {
            Some(handler) => handler(request),
            None => response_404(&request.path),
        };
        self.access_log.push((request.method.name().to_string(), request.path.clone(), response.status_code));
        response
    }

    pub fn process_raw(&mut self, raw: &str) -> String {
        let request = HttpRequest::parse(raw);
        let mut response = self.handle_request(&request);
        response.serialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let raw = "GET /hello?name=world HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = HttpRequest::parse(raw);
        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.path, "/hello");
        assert_eq!(req.query_params.get("name").map(|s| s.as_str()), Some("world"));
        assert_eq!(req.headers.get("host").map(|s| s.as_str()), Some("localhost"));
    }

    #[test]
    fn test_parse_post_with_body() {
        let raw = "POST /api/data HTTP/1.1\r\nContent-Type: application/json\r\n\r\n{\"key\":\"val\"}";
        let req = HttpRequest::parse(raw);
        assert_eq!(req.method, HttpMethod::Post);
        assert_eq!(req.body, "{\"key\":\"val\"}");
    }

    #[test]
    fn test_response_serialize() {
        let mut resp = response_200("Hello");
        let raw = resp.serialize();
        assert!(raw.starts_with("HTTP/1.1 200 OK"));
        assert!(raw.contains("Content-Length: 5"));
        assert!(raw.ends_with("Hello"));
    }

    #[test]
    fn test_routing() {
        let mut server = WebServer::new("test");
        server.add_route(HttpMethod::Get, "/", |_| response_200("home"));
        server.add_route(HttpMethod::Get, "/health", |_| response_json("{\"ok\":true}"));

        let req_home = HttpRequest::parse("GET / HTTP/1.1\r\n\r\n");
        let resp = server.handle_request(&req_home);
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.body, "home");

        let req_health = HttpRequest::parse("GET /health HTTP/1.1\r\n\r\n");
        let resp = server.handle_request(&req_health);
        assert_eq!(resp.status_code, 200);
        assert!(resp.body.contains("ok"));
    }

    #[test]
    fn test_404() {
        let mut server = WebServer::new("test");
        let req = HttpRequest::parse("GET /missing HTTP/1.1\r\n\r\n");
        let resp = server.handle_request(&req);
        assert_eq!(resp.status_code, 404);
    }

    #[test]
    fn test_static_files() {
        let mut server = WebServer::new("test");
        server.serve_static("/style.css", "body{}", "text/css");
        let req = HttpRequest::parse("GET /style.css HTTP/1.1\r\n\r\n");
        let resp = server.handle_request(&req);
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.body, "body{}");
    }

    #[test]
    fn test_access_log() {
        let mut server = WebServer::new("test");
        server.add_route(HttpMethod::Get, "/", |_| response_200("ok"));
        let req = HttpRequest::parse("GET / HTTP/1.1\r\n\r\n");
        server.handle_request(&req);
        server.handle_request(&req);
        assert_eq!(server.access_log.len(), 2);
    }

    #[test]
    fn test_json_response() {
        let resp = response_json("{\"time\":\"now\"}");
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.headers.get("Content-Type").map(|s| s.as_str()), Some("application/json"));
    }
}
