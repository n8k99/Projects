// Chat Application (Networking) — multi-room chat with users and history.

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub room: String,
    pub text: String,
    pub timestamp: u64, // milliseconds since epoch
}

#[derive(Debug)]
pub struct ChatRoom {
    pub name: String,
    pub members: HashSet<String>,
    pub messages: Vec<Message>,
}

#[derive(Debug)]
pub struct ChatUser {
    pub name: String,
    pub rooms: HashSet<String>,
}

#[derive(Debug)]
pub struct ChatServer {
    rooms: HashMap<String, ChatRoom>,
    users: HashMap<String, ChatUser>,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            rooms: HashMap::new(),
            users: HashMap::new(),
        }
    }

    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    pub fn register_user(&mut self, name: &str) -> Result<(), String> {
        if self.users.contains_key(name) {
            return Err(format!("User '{}' already exists", name));
        }
        self.users.insert(
            name.to_string(),
            ChatUser {
                name: name.to_string(),
                rooms: HashSet::new(),
            },
        );
        Ok(())
    }

    pub fn create_room(&mut self, room_name: &str) -> Result<(), String> {
        if self.rooms.contains_key(room_name) {
            return Err(format!("Room '{}' already exists", room_name));
        }
        self.rooms.insert(
            room_name.to_string(),
            ChatRoom {
                name: room_name.to_string(),
                members: HashSet::new(),
                messages: Vec::new(),
            },
        );
        Ok(())
    }

    pub fn join_room(&mut self, user_name: &str, room_name: &str) -> Result<(), String> {
        if !self.users.contains_key(user_name) {
            return Err(format!("No such user: '{}'", user_name));
        }
        if !self.rooms.contains_key(room_name) {
            return Err(format!("No such room: '{}'", room_name));
        }
        self.rooms
            .get_mut(room_name)
            .unwrap()
            .members
            .insert(user_name.to_string());
        self.users
            .get_mut(user_name)
            .unwrap()
            .rooms
            .insert(room_name.to_string());
        Ok(())
    }

    pub fn leave_room(&mut self, user_name: &str, room_name: &str) -> Result<(), String> {
        if !self.users.contains_key(user_name) {
            return Err(format!("No such user: '{}'", user_name));
        }
        if !self.rooms.contains_key(room_name) {
            return Err(format!("No such room: '{}'", room_name));
        }
        self.rooms
            .get_mut(room_name)
            .unwrap()
            .members
            .remove(user_name);
        self.users
            .get_mut(user_name)
            .unwrap()
            .rooms
            .remove(room_name);
        Ok(())
    }

    pub fn send_message(
        &mut self,
        user_name: &str,
        room_name: &str,
        text: &str,
    ) -> Result<Message, String> {
        if !self.users.contains_key(user_name) {
            return Err(format!("No such user: '{}'", user_name));
        }
        let room = self
            .rooms
            .get_mut(room_name)
            .ok_or_else(|| format!("No such room: '{}'", room_name))?;
        if !room.members.contains(user_name) {
            return Err(format!(
                "User '{}' is not a member of room '{}'",
                user_name, room_name
            ));
        }
        let msg = Message {
            sender: user_name.to_string(),
            room: room_name.to_string(),
            text: text.to_string(),
            timestamp: Self::now_millis(),
        };
        room.messages.push(msg.clone());
        Ok(msg)
    }

    pub fn get_messages(&self, room_name: &str, since: Option<u64>) -> Result<Vec<&Message>, String> {
        let room = self
            .rooms
            .get(room_name)
            .ok_or_else(|| format!("No such room: '{}'", room_name))?;
        match since {
            None => Ok(room.messages.iter().collect()),
            Some(ts) => Ok(room.messages.iter().filter(|m| m.timestamp >= ts).collect()),
        }
    }

    pub fn list_rooms(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.rooms.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    pub fn list_users(&self, room_name: &str) -> Result<Vec<&str>, String> {
        let room = self
            .rooms
            .get(room_name)
            .ok_or_else(|| format!("No such room: '{}'", room_name))?;
        let mut members: Vec<&str> = room.members.iter().map(|s| s.as_str()).collect();
        members.sort();
        Ok(members)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_server() -> ChatServer {
        let mut server = ChatServer::new();
        server.register_user("alice").unwrap();
        server.register_user("bob").unwrap();
        server.register_user("carol").unwrap();
        server.create_room("#general").unwrap();
        server.create_room("#engineering").unwrap();
        server
    }

    #[test]
    fn test_register_user() {
        let mut server = ChatServer::new();
        assert!(server.register_user("alice").is_ok());
        assert!(server.register_user("alice").is_err());
    }

    #[test]
    fn test_create_room() {
        let mut server = ChatServer::new();
        assert!(server.create_room("#general").is_ok());
        assert!(server.create_room("#general").is_err());
    }

    #[test]
    fn test_join_and_list_users() {
        let mut server = setup_server();
        server.join_room("alice", "#general").unwrap();
        server.join_room("bob", "#general").unwrap();
        let users = server.list_users("#general").unwrap();
        assert_eq!(users, vec!["alice", "bob"]);
    }

    #[test]
    fn test_leave_room() {
        let mut server = setup_server();
        server.join_room("alice", "#general").unwrap();
        server.join_room("bob", "#general").unwrap();
        server.leave_room("alice", "#general").unwrap();
        let users = server.list_users("#general").unwrap();
        assert_eq!(users, vec!["bob"]);
    }

    #[test]
    fn test_send_and_get_messages() {
        let mut server = setup_server();
        server.join_room("alice", "#general").unwrap();
        server.join_room("bob", "#general").unwrap();
        server.send_message("alice", "#general", "Hello!").unwrap();
        server.send_message("bob", "#general", "Hi Alice!").unwrap();
        let msgs = server.get_messages("#general", None).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].sender, "alice");
        assert_eq!(msgs[1].text, "Hi Alice!");
    }

    #[test]
    fn test_send_message_not_member() {
        let mut server = setup_server();
        let result = server.send_message("alice", "#general", "Hello!");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_rooms() {
        let server = setup_server();
        let rooms = server.list_rooms();
        assert_eq!(rooms.len(), 2);
        assert!(rooms.contains(&"#general"));
        assert!(rooms.contains(&"#engineering"));
    }

    #[test]
    fn test_get_messages_since() {
        let mut server = setup_server();
        server.join_room("alice", "#general").unwrap();
        server.send_message("alice", "#general", "Old message").unwrap();
        // Ensure a different millisecond for the cutoff
        std::thread::sleep(std::time::Duration::from_millis(2));
        let cutoff = ChatServer::now_millis();
        std::thread::sleep(std::time::Duration::from_millis(2));
        server.send_message("alice", "#general", "New message").unwrap();
        let msgs = server.get_messages("#general", Some(cutoff)).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].text, "New message");
    }

    #[test]
    fn test_multi_room_isolation() {
        let mut server = setup_server();
        server.join_room("alice", "#general").unwrap();
        server.join_room("alice", "#engineering").unwrap();
        server.send_message("alice", "#general", "General msg").unwrap();
        server.send_message("alice", "#engineering", "Eng msg").unwrap();
        assert_eq!(server.get_messages("#general", None).unwrap().len(), 1);
        assert_eq!(server.get_messages("#engineering", None).unwrap().len(), 1);
    }
}
