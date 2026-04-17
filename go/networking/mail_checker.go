// Mail Checker — monitor an inbox for new messages with filtering and notification.
package networking

import (
	"fmt"
	"sort"
	"strings"
	"sync"
	"time"
)

// MailAddress is a parsed email address.
type MailAddress struct {
	Name    string
	Address string
}

func (a MailAddress) String() string {
	if a.Name != "" {
		return fmt.Sprintf("%s <%s>", a.Name, a.Address)
	}
	return a.Address
}

// ParseMailAddress parses "Name <addr>" or plain "addr".
func ParseMailAddress(raw string) MailAddress {
	raw = strings.TrimSpace(raw)
	if lt := strings.Index(raw, "<"); lt >= 0 {
		if gt := strings.Index(raw, ">"); gt > lt {
			name := strings.TrimSpace(raw[:lt])
			name = strings.Trim(name, "\"")
			addr := strings.TrimSpace(raw[lt+1 : gt])
			return MailAddress{Name: name, Address: addr}
		}
	}
	return MailAddress{Address: raw}
}

// MailMsg is a single email message.
type MailMsg struct {
	UID        string
	Sender     MailAddress
	Recipients []MailAddress
	Subject    string
	Body       string
	Date       time.Time
	IsRead     bool
	IsFlagged  bool
	Headers    map[string]string
}

// Preview returns the first 80 chars of the body.
func (m MailMsg) Preview() string {
	body := strings.ReplaceAll(m.Body, "\n", " ")
	if len(body) > 80 {
		return body[:80] + "..."
	}
	return body
}

// MailFilter defines criteria for filtering messages.
type MailFilter struct {
	SenderContains  string
	SubjectContains string
	BodyContains    string
	IsUnreadOnly    bool
	Since           *time.Time
}

// Matches checks if a message matches the filter.
func (f *MailFilter) Matches(m *MailMsg) bool {
	if f.SenderContains != "" &&
		!strings.Contains(strings.ToLower(m.Sender.Address), strings.ToLower(f.SenderContains)) {
		return false
	}
	if f.SubjectContains != "" &&
		!strings.Contains(strings.ToLower(m.Subject), strings.ToLower(f.SubjectContains)) {
		return false
	}
	if f.BodyContains != "" &&
		!strings.Contains(strings.ToLower(m.Body), strings.ToLower(f.BodyContains)) {
		return false
	}
	if f.IsUnreadOnly && m.IsRead {
		return false
	}
	if f.Since != nil && m.Date.Before(*f.Since) {
		return false
	}
	return true
}

// MailCheckResult holds the result of a check operation.
type MailCheckResult struct {
	TotalMessages   int
	NewMessages     int
	MatchedMessages []MailMsg
	CheckedAt       time.Time
}

// Summary returns a human-readable summary.
func (r MailCheckResult) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "Mail check at %s\n", r.CheckedAt.Format("2006-01-02 15:04:05 UTC"))
	fmt.Fprintf(&b, "  Total: %d, New: %d, Matched: %d\n",
		r.TotalMessages, r.NewMessages, len(r.MatchedMessages))
	limit := 10
	if len(r.MatchedMessages) < limit {
		limit = len(r.MatchedMessages)
	}
	for _, msg := range r.MatchedMessages[:limit] {
		read := "●"
		if msg.IsRead {
			read = " "
		}
		flag := " "
		if msg.IsFlagged {
			flag = "★"
		}
		fmt.Fprintf(&b, "  %s%s %s %-30s %s\n",
			read, flag, msg.Date.Format("2006-01-02 15:04"), msg.Sender.Address, msg.Subject)
	}
	if len(r.MatchedMessages) > 10 {
		fmt.Fprintf(&b, "  ... and %d more\n", len(r.MatchedMessages)-10)
	}
	return b.String()
}

// MailboxStore is an in-process mailbox.
type MailboxStore struct {
	mu        sync.RWMutex
	Owner     string
	messages  map[string]*MailMsg
	seenUIDs  map[string]bool
	LastCheck *time.Time
}

// NewMailboxStore creates a mailbox for the given owner.
func NewMailboxStore(owner string) *MailboxStore {
	return &MailboxStore{
		Owner:    owner,
		messages: make(map[string]*MailMsg),
		seenUIDs: make(map[string]bool),
	}
}

// Deliver adds a message to the mailbox.
func (mb *MailboxStore) Deliver(msg MailMsg) {
	mb.mu.Lock()
	defer mb.mu.Unlock()
	mb.messages[msg.UID] = &msg
}

// Check performs a mail check, returning results with optional filter.
func (mb *MailboxStore) Check(filter *MailFilter) MailCheckResult {
	mb.mu.Lock()
	defer mb.mu.Unlock()

	newCount := 0
	for uid := range mb.messages {
		if !mb.seenUIDs[uid] {
			newCount++
			mb.seenUIDs[uid] = true
		}
	}

	now := time.Now().UTC()
	mb.LastCheck = &now

	msgs := make([]MailMsg, 0, len(mb.messages))
	for _, m := range mb.messages {
		msgs = append(msgs, *m)
	}
	sort.Slice(msgs, func(i, j int) bool { return msgs[i].Date.After(msgs[j].Date) })

	var matched []MailMsg
	if filter != nil {
		for _, m := range msgs {
			if filter.Matches(&m) {
				matched = append(matched, m)
			}
		}
	} else {
		matched = msgs
	}

	return MailCheckResult{
		TotalMessages:   len(mb.messages),
		NewMessages:     newCount,
		MatchedMessages: matched,
		CheckedAt:       now,
	}
}

// MarkRead marks a message as read.
func (mb *MailboxStore) MarkRead(uid string) bool {
	mb.mu.Lock()
	defer mb.mu.Unlock()
	if m, ok := mb.messages[uid]; ok {
		m.IsRead = true
		return true
	}
	return false
}

// MarkFlagged flags or unflags a message.
func (mb *MailboxStore) MarkFlagged(uid string, flagged bool) bool {
	mb.mu.Lock()
	defer mb.mu.Unlock()
	if m, ok := mb.messages[uid]; ok {
		m.IsFlagged = flagged
		return true
	}
	return false
}

// Delete removes a message.
func (mb *MailboxStore) Delete(uid string) bool {
	mb.mu.Lock()
	defer mb.mu.Unlock()
	if _, ok := mb.messages[uid]; ok {
		delete(mb.messages, uid)
		delete(mb.seenUIDs, uid)
		return true
	}
	return false
}

// UnreadCount returns the number of unread messages.
func (mb *MailboxStore) UnreadCount() int {
	mb.mu.RLock()
	defer mb.mu.RUnlock()
	count := 0
	for _, m := range mb.messages {
		if !m.IsRead {
			count++
		}
	}
	return count
}

// Total returns the total number of messages.
func (mb *MailboxStore) Total() int {
	mb.mu.RLock()
	defer mb.mu.RUnlock()
	return len(mb.messages)
}

// MailServerStore routes messages between mailboxes.
type MailServerStore struct {
	mu        sync.Mutex
	mailboxes map[string]*MailboxStore
	nextUID   int
}

// NewMailServerStore creates a mail server.
func NewMailServerStore() *MailServerStore {
	return &MailServerStore{
		mailboxes: make(map[string]*MailboxStore),
	}
}

// CreateMailbox creates a mailbox for the given owner address.
func (s *MailServerStore) CreateMailbox(owner string) *MailboxStore {
	s.mu.Lock()
	defer s.mu.Unlock()
	mb := NewMailboxStore(owner)
	s.mailboxes[owner] = mb
	return mb
}

// Send routes a message to all local recipients.
func (s *MailServerStore) Send(sender MailAddress, recipients []MailAddress,
	subject, body string) string {
	s.mu.Lock()
	s.nextUID++
	uid := fmt.Sprintf("msg-%08x", s.nextUID)
	s.mu.Unlock()

	msg := MailMsg{
		UID:        uid,
		Sender:     sender,
		Recipients: recipients,
		Subject:    subject,
		Body:       body,
		Date:       time.Now().UTC(),
		Headers:    make(map[string]string),
	}
	for _, r := range recipients {
		s.mu.Lock()
		mb, ok := s.mailboxes[r.Address]
		s.mu.Unlock()
		if ok {
			mb.Deliver(msg)
		}
	}
	return uid
}
