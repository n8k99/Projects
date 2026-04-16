//! FTP Program — file transfer protocol client/server model.

use std::collections::BTreeMap;

/// A simplified FTP server with a virtual filesystem.
pub struct FtpServer {
    files: BTreeMap<String, String>,
}

impl FtpServer {
    pub fn new() -> Self {
        FtpServer {
            files: BTreeMap::new(),
        }
    }

    /// Seed the virtual filesystem with a file.
    pub fn add_file(&mut self, name: &str, content: &str) {
        self.files.insert(name.to_string(), content.to_string());
    }

    /// Process a command string and return a response string.
    pub fn handle(&mut self, command: &str) -> String {
        let trimmed = command.trim();
        let mut parts = trimmed.splitn(3, ' ');
        let cmd = match parts.next() {
            Some(c) => c.to_uppercase(),
            None => return "500 Syntax error".to_string(),
        };

        match cmd.as_str() {
            "LIST" => {
                if self.files.is_empty() {
                    "226 (empty)".to_string()
                } else {
                    let listing: Vec<&str> = self.files.keys().map(|s| s.as_str()).collect();
                    format!("226 File list:\n{}", listing.join("\n"))
                }
            }
            "RETR" => {
                let filename = match parts.next() {
                    Some(f) => f,
                    None => return "501 Syntax error in parameters".to_string(),
                };
                match self.files.get(filename) {
                    Some(content) => format!("226 Transfer complete:\n{}", content),
                    None => format!("550 {}: No such file", filename),
                }
            }
            "STOR" => {
                let filename = match parts.next() {
                    Some(f) => f,
                    None => return "501 Syntax error in parameters".to_string(),
                };
                let content = match parts.next() {
                    Some(c) => c,
                    None => return "501 Syntax error in parameters".to_string(),
                };
                let len = content.len();
                self.files.insert(filename.to_string(), content.to_string());
                format!("226 {} stored ({} bytes)", filename, len)
            }
            "DELE" => {
                let filename = match parts.next() {
                    Some(f) => f,
                    None => return "501 Syntax error in parameters".to_string(),
                };
                if self.files.remove(filename).is_some() {
                    format!("250 {} deleted", filename)
                } else {
                    format!("550 {}: No such file", filename)
                }
            }
            "SIZE" => {
                let filename = match parts.next() {
                    Some(f) => f,
                    None => return "501 Syntax error in parameters".to_string(),
                };
                match self.files.get(filename) {
                    Some(content) => format!("213 {}", content.len()),
                    None => format!("550 {}: No such file", filename),
                }
            }
            "QUIT" => "221 Goodbye".to_string(),
            _ => format!("502 Command not implemented: {}", cmd),
        }
    }
}

/// A simplified FTP client that sends commands to an FtpServer.
pub struct FtpClient<'a> {
    server: Option<&'a mut FtpServer>,
}

impl<'a> FtpClient<'a> {
    pub fn new() -> Self {
        FtpClient { server: None }
    }

    pub fn connect(&mut self, server: &'a mut FtpServer) -> String {
        self.server = Some(server);
        "220 Service ready".to_string()
    }

    pub fn send(&mut self, command: &str) -> String {
        let server = match self.server.as_mut() {
            Some(s) => s,
            None => return "421 Not connected".to_string(),
        };
        let response = server.handle(command);
        if command.trim().eq_ignore_ascii_case("QUIT") {
            self.server = None;
        }
        response
    }

    pub fn list_files(&mut self) -> String {
        self.send("LIST")
    }

    pub fn get(&mut self, filename: &str) -> String {
        self.send(&format!("RETR {}", filename))
    }

    pub fn put(&mut self, filename: &str, content: &str) -> String {
        self.send(&format!("STOR {} {}", filename, content))
    }

    pub fn delete(&mut self, filename: &str) -> String {
        self.send(&format!("DELE {}", filename))
    }

    pub fn size(&mut self, filename: &str) -> String {
        self.send(&format!("SIZE {}", filename))
    }

    pub fn quit(&mut self) -> String {
        self.send("QUIT")
    }
}

fn main() {
    let mut server = FtpServer::new();
    server.add_file("readme.txt", "Welcome to the FTP server.");
    server.add_file("data.csv", "name,value\nalpha,1\nbeta,2");

    let mut client = FtpClient::new();
    println!("{}", client.connect(&mut server));

    println!("\n--- LIST ---");
    println!("{}", client.list_files());

    println!("\n--- RETR readme.txt ---");
    println!("{}", client.get("readme.txt"));

    println!("\n--- STOR notes.txt ---");
    println!("{}", client.put("notes.txt", "These are my notes."));

    println!("\n--- LIST ---");
    println!("{}", client.list_files());

    println!("\n--- SIZE data.csv ---");
    println!("{}", client.size("data.csv"));

    println!("\n--- DELE data.csv ---");
    println!("{}", client.delete("data.csv"));

    println!("\n--- QUIT ---");
    println!("{}", client.quit());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_empty() {
        let mut server = FtpServer::new();
        assert_eq!(server.handle("LIST"), "226 (empty)");
    }

    #[test]
    fn test_list_with_files() {
        let mut server = FtpServer::new();
        server.add_file("b.txt", "bee");
        server.add_file("a.txt", "ay");
        let response = server.handle("LIST");
        assert!(response.starts_with("226 File list:"));
        assert!(response.contains("a.txt"));
        assert!(response.contains("b.txt"));
    }

    #[test]
    fn test_stor_and_retr() {
        let mut server = FtpServer::new();
        let store_resp = server.handle("STOR hello.txt Hello world");
        assert!(store_resp.starts_with("226"));
        let retr_resp = server.handle("RETR hello.txt");
        assert!(retr_resp.contains("Hello world"));
    }

    #[test]
    fn test_retr_missing() {
        let mut server = FtpServer::new();
        let resp = server.handle("RETR ghost.txt");
        assert!(resp.starts_with("550"));
    }

    #[test]
    fn test_dele() {
        let mut server = FtpServer::new();
        server.add_file("temp.txt", "data");
        let resp = server.handle("DELE temp.txt");
        assert!(resp.starts_with("250"));
        let resp2 = server.handle("RETR temp.txt");
        assert!(resp2.starts_with("550"));
    }

    #[test]
    fn test_dele_missing() {
        let mut server = FtpServer::new();
        let resp = server.handle("DELE nope.txt");
        assert!(resp.starts_with("550"));
    }

    #[test]
    fn test_size() {
        let mut server = FtpServer::new();
        server.add_file("f.txt", "12345");
        assert_eq!(server.handle("SIZE f.txt"), "213 5");
    }

    #[test]
    fn test_quit() {
        let mut server = FtpServer::new();
        assert_eq!(server.handle("QUIT"), "221 Goodbye");
    }

    #[test]
    fn test_unknown_command() {
        let mut server = FtpServer::new();
        let resp = server.handle("NOOP");
        assert!(resp.starts_with("502"));
    }

    #[test]
    fn test_client_not_connected() {
        let mut client = FtpClient::new();
        assert_eq!(client.list_files(), "421 Not connected");
    }

    #[test]
    fn test_client_disconnect_on_quit() {
        let mut server = FtpServer::new();
        let mut client = FtpClient::new();
        client.connect(&mut server);
        client.quit();
        assert_eq!(client.list_files(), "421 Not connected");
    }
}
