//! Chat Application — bidirectional broadcast across N concurrent actors.
//!
//! Every actor is both a producer (sending messages) and a consumer (reading
//! messages from its inbox). One send from Alice fans out to all other
//! joined users' inboxes simultaneously — broadcast, not point-to-point.
//!
//! The room is the shared synchronisation point. Join/leave is dynamic:
//! users enter and exit at runtime while other users keep sending. A send
//! iterates a snapshot of the user list under the room lock, then pushes
//! to each target's inbox (each inbox has its own lock, so the snapshot
//! iteration is released before any inbox push waits).
//!
//! A monotonic sequence counter assigned at send time under the room lock
//! gives every message a total order that is consistent across all recipients.
//! This is what "message ordering" means in a concurrent broadcast system.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

pub type UserId = u64;

#[derive(Debug, Clone)]
pub struct Message {
    pub seq: u64,
    pub from: UserId,
    pub from_name: String,
    pub text: String,
}

struct Inbox {
    name: String,
    messages: Mutex<VecDeque<Message>>,
}

struct RoomState {
    users: HashMap<UserId, Inbox>,
    log: Vec<Message>,
    next_id: u64,
    next_seq: u64,
}

pub struct ChatRoom {
    state: Mutex<RoomState>,
}

impl ChatRoom {
    pub fn new() -> Self {
        Self { state: Mutex::new(RoomState {
            users: HashMap::new(), log: Vec::new(),
            next_id: 1, next_seq: 1,
        }) }
    }

    pub fn join(&self, name: &str) -> UserId {
        let mut st = self.state.lock().unwrap();
        let id = st.next_id;
        st.next_id += 1;
        st.users.insert(id, Inbox {
            name: name.to_string(),
            messages: Mutex::new(VecDeque::new()),
        });
        id
    }

    pub fn leave(&self, id: UserId) {
        self.state.lock().unwrap().users.remove(&id);
    }

    /// Broadcast `text` from `from` to every currently-joined user (including
    /// sender). Assigns a monotonic sequence number under the room lock so
    /// every recipient sees messages in the same total order.
    ///
    /// Returns false if the sender is not a member.
    pub fn send(&self, from: UserId, text: &str) -> bool {
        let mut st = self.state.lock().unwrap();
        let Some(sender) = st.users.get(&from) else { return false; };
        let sender_name = sender.name.clone();
        let seq = st.next_seq;
        st.next_seq += 1;
        let msg = Message {
            seq, from, from_name: sender_name, text: text.to_string(),
        };
        st.log.push(msg.clone());
        // Push into every inbox. All inboxes are held inside `st`, so we
        // can iterate and push while holding the room lock.
        for inbox in st.users.values() {
            inbox.messages.lock().unwrap().push_back(msg.clone());
        }
        true
    }

    /// Drain and return everything currently in this user's inbox.
    pub fn read(&self, id: UserId) -> Vec<Message> {
        let st = self.state.lock().unwrap();
        let Some(inbox) = st.users.get(&id) else { return Vec::new(); };
        let mut queue = inbox.messages.lock().unwrap();
        queue.drain(..).collect()
    }

    pub fn history(&self) -> Vec<Message> {
        self.state.lock().unwrap().log.clone()
    }

    pub fn user_count(&self) -> usize {
        self.state.lock().unwrap().users.len()
    }
}

impl Default for ChatRoom {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn two_users_send_and_receive() {
        let room = ChatRoom::new();
        let alice = room.join("Alice");
        let bob = room.join("Bob");
        assert!(room.send(alice, "hi bob"));
        let bobs = room.read(bob);
        assert_eq!(bobs.len(), 1);
        assert_eq!(bobs[0].text, "hi bob");
        assert_eq!(bobs[0].from_name, "Alice");
        // Sender sees their own message too.
        let alices = room.read(alice);
        assert_eq!(alices.len(), 1);
    }

    #[test]
    fn broadcast_reaches_every_joined_user() {
        let room = ChatRoom::new();
        let a = room.join("Alice");
        let b = room.join("Bob");
        let c = room.join("Carol");
        let d = room.join("Dave");
        room.send(a, "hello everyone");
        for u in [a, b, c, d] {
            assert_eq!(room.read(u).len(), 1, "user {u} missed the broadcast");
        }
    }

    #[test]
    fn leaving_stops_future_delivery_but_keeps_history() {
        let room = ChatRoom::new();
        let a = room.join("Alice");
        let b = room.join("Bob");
        room.send(a, "round 1");
        // Bob reads round 1, then leaves.
        assert_eq!(room.read(b).len(), 1);
        room.leave(b);
        room.send(a, "round 2");
        // Alice receives round 2; the global log has both rounds.
        assert_eq!(room.read(a).len(), 2);  // a received both
        assert_eq!(room.history().len(), 2);
        assert_eq!(room.user_count(), 1);
    }

    #[test]
    fn messages_from_one_sender_arrive_in_order() {
        let room = ChatRoom::new();
        let a = room.join("Alice");
        let b = room.join("Bob");
        for i in 0..10 {
            room.send(a, &format!("msg-{i}"));
        }
        let bobs = room.read(b);
        assert_eq!(bobs.len(), 10);
        for (i, m) in bobs.iter().enumerate() {
            assert_eq!(m.text, format!("msg-{i}"));
            assert_eq!(m.seq as usize, i + 1);
        }
    }

    #[test]
    fn concurrent_senders_produce_total_order() {
        // Two senders each push 50 messages concurrently. The recipient sees
        // a valid interleaving with strictly increasing sequence numbers.
        let room = Arc::new(ChatRoom::new());
        let alice = room.join("Alice");
        let bob = room.join("Bob");
        let carol = room.join("Carol");

        let r1 = Arc::clone(&room);
        let r2 = Arc::clone(&room);
        let h1 = thread::spawn(move || {
            for i in 0..50 { r1.send(alice, &format!("A{i}")); }
        });
        let h2 = thread::spawn(move || {
            for i in 0..50 { r2.send(bob, &format!("B{i}")); }
        });
        h1.join().unwrap();
        h2.join().unwrap();

        let carols = room.read(carol);
        assert_eq!(carols.len(), 100);
        // Sequence numbers strictly increasing.
        for w in carols.windows(2) {
            assert!(w[0].seq < w[1].seq, "seq broke monotonicity");
        }
        // Within each sender, relative order preserved.
        let alice_msgs: Vec<_> = carols.iter().filter(|m| m.from == alice).collect();
        let bob_msgs: Vec<_> = carols.iter().filter(|m| m.from == bob).collect();
        assert_eq!(alice_msgs.len(), 50);
        assert_eq!(bob_msgs.len(), 50);
        for (i, m) in alice_msgs.iter().enumerate() {
            assert_eq!(m.text, format!("A{i}"));
        }
        for (i, m) in bob_msgs.iter().enumerate() {
            assert_eq!(m.text, format!("B{i}"));
        }
    }

    #[test]
    fn send_from_unknown_user_is_rejected() {
        let room = ChatRoom::new();
        let alice = room.join("Alice");
        room.leave(alice);
        assert!(!room.send(alice, "ghost message"));
        assert_eq!(room.history().len(), 0);
    }

    #[test]
    fn read_from_unknown_user_returns_empty() {
        let room = ChatRoom::new();
        let msgs = room.read(9999);
        assert!(msgs.is_empty());
    }
}
