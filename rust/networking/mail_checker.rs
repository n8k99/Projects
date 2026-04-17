// Mail Checker — monitor an inbox for new messages with filtering and notification.

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct EmailAddress {
    pub name: String,
    pub address: String,
}

impl EmailAddress {
    pub fn new(address: &str) -> Self {
        EmailAddress { name: String::new(), address: address.to_string() }
    }

    pub fn with_name(name: &str, address: &str) -> Self {
        EmailAddress { name: name.to_string(), address: address.to_string() }
    }

    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim();
        if let (Some(lt), Some(gt)) = (raw.find('<'), raw.find('>')) {
            let name = raw[..lt].trim().trim_matches('"').to_string();
            let addr = raw[lt + 1..gt].trim().to_string();
            EmailAddress { name, address: addr }
        } else {
            EmailAddress { name: String::new(), address: raw.to_string() }
        }
    }
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_empty() {
            write!(f, "{}", self.address)
        } else {
            write!(f, "{} <{}>", self.name, self.address)
        }
    }
}

#[derive(Debug, Clone)]
pub struct MailMessage {
    pub uid: String,
    pub sender: EmailAddress,
    pub recipients: Vec<EmailAddress>,
    pub subject: String,
    pub body: String,
    pub date: u64,
    pub is_read: bool,
    pub is_flagged: bool,
    pub headers: HashMap<String, String>,
}

impl MailMessage {
    pub fn preview(&self) -> String {
        let clean: String = self.body.chars().take(80).map(|c| if c == '\n' { ' ' } else { c }).collect();
        if self.body.len() > 80 {
            format!("{}...", clean)
        } else {
            clean
        }
    }

    pub fn matches_filter(&self, f: &MailFilter) -> bool {
        if !f.sender_contains.is_empty()
            && !self.sender.address.to_lowercase().contains(&f.sender_contains.to_lowercase())
        {
            return false;
        }
        if !f.subject_contains.is_empty()
            && !self.subject.to_lowercase().contains(&f.subject_contains.to_lowercase())
        {
            return false;
        }
        if !f.body_contains.is_empty()
            && !self.body.to_lowercase().contains(&f.body_contains.to_lowercase())
        {
            return false;
        }
        if f.is_unread_only && self.is_read {
            return false;
        }
        if let Some(since) = f.since {
            if self.date < since {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Default)]
pub struct MailFilter {
    pub sender_contains: String,
    pub subject_contains: String,
    pub body_contains: String,
    pub is_unread_only: bool,
    pub since: Option<u64>,
}

#[derive(Debug)]
pub struct CheckResult {
    pub total_messages: usize,
    pub new_messages: usize,
    pub matched_messages: Vec<MailMessage>,
    pub checked_at: u64,
}

impl CheckResult {
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("Mail check at {}", self.checked_at),
            format!("  Total: {}, New: {}, Matched: {}",
                    self.total_messages, self.new_messages, self.matched_messages.len()),
        ];
        for msg in self.matched_messages.iter().take(10) {
            let read = if msg.is_read { " " } else { "●" };
            let flag = if msg.is_flagged { "★" } else { " " };
            lines.push(format!("  {}{} {} {:30} {}",
                               read, flag, msg.date, msg.sender.address, msg.subject));
        }
        if self.matched_messages.len() > 10 {
            lines.push(format!("  ... and {} more", self.matched_messages.len() - 10));
        }
        lines.join("\n")
    }
}

pub struct Mailbox {
    pub owner: String,
    messages: HashMap<String, MailMessage>,
    seen_uids: HashSet<String>,
    pub last_check: Option<u64>,
}

impl Mailbox {
    pub fn new(owner: &str) -> Self {
        Mailbox {
            owner: owner.to_string(),
            messages: HashMap::new(),
            seen_uids: HashSet::new(),
            last_check: None,
        }
    }

    pub fn deliver(&mut self, msg: MailMessage) {
        self.messages.insert(msg.uid.clone(), msg);
    }

    pub fn check(&mut self, filter: Option<&MailFilter>) -> CheckResult {
        let current_uids: HashSet<String> = self.messages.keys().cloned().collect();
        let new_count = current_uids.difference(&self.seen_uids).count();
        self.seen_uids = current_uids;
        let now = now_secs();
        self.last_check = Some(now);

        let mut all_msgs: Vec<MailMessage> = self.messages.values().cloned().collect();
        all_msgs.sort_by(|a, b| b.date.cmp(&a.date));

        let matched = match filter {
            Some(f) => all_msgs.into_iter().filter(|m| m.matches_filter(f)).collect(),
            None => all_msgs,
        };

        CheckResult {
            total_messages: self.messages.len(),
            new_messages: new_count,
            matched_messages: matched,
            checked_at: now,
        }
    }

    pub fn mark_read(&mut self, uid: &str) -> bool {
        if let Some(msg) = self.messages.get_mut(uid) {
            msg.is_read = true;
            true
        } else {
            false
        }
    }

    pub fn mark_flagged(&mut self, uid: &str, flagged: bool) -> bool {
        if let Some(msg) = self.messages.get_mut(uid) {
            msg.is_flagged = flagged;
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self, uid: &str) -> bool {
        if self.messages.remove(uid).is_some() {
            self.seen_uids.remove(uid);
            true
        } else {
            false
        }
    }

    pub fn unread_count(&self) -> usize {
        self.messages.values().filter(|m| !m.is_read).count()
    }

    pub fn flagged_count(&self) -> usize {
        self.messages.values().filter(|m| m.is_flagged).count()
    }

    pub fn total(&self) -> usize {
        self.messages.len()
    }
}

pub struct MailServer {
    mailboxes: HashMap<String, Mailbox>,
    next_uid: u64,
}

impl MailServer {
    pub fn new() -> Self {
        MailServer { mailboxes: HashMap::new(), next_uid: 0 }
    }

    pub fn create_mailbox(&mut self, owner: &str) -> &mut Mailbox {
        self.mailboxes.entry(owner.to_string())
            .or_insert_with(|| Mailbox::new(owner))
    }

    pub fn send(&mut self, sender: EmailAddress, recipients: Vec<EmailAddress>,
                subject: &str, body: &str) -> String {
        self.next_uid += 1;
        let uid = format!("msg-{:08x}", self.next_uid);
        let msg = MailMessage {
            uid: uid.clone(),
            sender,
            recipients: recipients.clone(),
            subject: subject.to_string(),
            body: body.to_string(),
            date: now_secs(),
            is_read: false,
            is_flagged: false,
            headers: HashMap::new(),
        };
        for recip in &recipients {
            if let Some(mb) = self.mailboxes.get_mut(&recip.address) {
                mb.deliver(msg.clone());
            }
        }
        uid
    }

    pub fn get_mailbox(&mut self, owner: &str) -> Option<&mut Mailbox> {
        self.mailboxes.get_mut(owner)
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_parse() {
        let addr = EmailAddress::parse("Kathryn <kathryn@em.com>");
        assert_eq!(addr.name, "Kathryn");
        assert_eq!(addr.address, "kathryn@em.com");

        let addr = EmailAddress::parse("plain@em.com");
        assert_eq!(addr.name, "");
        assert_eq!(addr.address, "plain@em.com");
    }

    #[test]
    fn test_deliver_and_check() {
        let mut mb = Mailbox::new("test@em.com");
        mb.deliver(MailMessage {
            uid: "001".into(), sender: EmailAddress::new("a@em.com"),
            recipients: vec![EmailAddress::new("test@em.com")],
            subject: "Hello".into(), body: "World".into(),
            date: 1000, is_read: false, is_flagged: false, headers: HashMap::new(),
        });
        let result = mb.check(None);
        assert_eq!(result.total_messages, 1);
        assert_eq!(result.new_messages, 1);
        assert_eq!(result.matched_messages.len(), 1);

        // Second check — no new messages
        let result = mb.check(None);
        assert_eq!(result.new_messages, 0);
    }

    #[test]
    fn test_filter() {
        let msg = MailMessage {
            uid: "001".into(), sender: EmailAddress::new("kathryn@em.com"),
            recipients: vec![], subject: "Finance Report".into(),
            body: "Q2 positions".into(), date: 1000,
            is_read: false, is_flagged: false, headers: HashMap::new(),
        };

        let f = MailFilter { sender_contains: "kathryn".into(), ..Default::default() };
        assert!(msg.matches_filter(&f));

        let f = MailFilter { sender_contains: "jay".into(), ..Default::default() };
        assert!(!msg.matches_filter(&f));

        let f = MailFilter { subject_contains: "finance".into(), ..Default::default() };
        assert!(msg.matches_filter(&f));

        let f = MailFilter { is_unread_only: true, ..Default::default() };
        assert!(msg.matches_filter(&f));
    }

    #[test]
    fn test_mark_read_and_delete() {
        let mut mb = Mailbox::new("test@em.com");
        mb.deliver(MailMessage {
            uid: "001".into(), sender: EmailAddress::new("a@em.com"),
            recipients: vec![], subject: "Test".into(), body: "".into(),
            date: 1000, is_read: false, is_flagged: false, headers: HashMap::new(),
        });
        assert_eq!(mb.unread_count(), 1);
        mb.mark_read("001");
        assert_eq!(mb.unread_count(), 0);
        mb.delete("001");
        assert_eq!(mb.total(), 0);
    }

    #[test]
    fn test_server_routing() {
        let mut server = MailServer::new();
        server.create_mailbox("a@em.com");
        server.create_mailbox("b@em.com");
        server.send(
            EmailAddress::new("a@em.com"),
            vec![EmailAddress::new("b@em.com")],
            "Hello", "Body",
        );
        let mb = server.get_mailbox("b@em.com").unwrap();
        assert_eq!(mb.total(), 1);
        let mb_a = server.get_mailbox("a@em.com").unwrap();
        assert_eq!(mb_a.total(), 0); // sender doesn't get their own message
    }
}
