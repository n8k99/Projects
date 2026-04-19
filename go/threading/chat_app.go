// Chat Application — bidirectional broadcast across N concurrent actors.
//
// Every actor is both a producer (sending messages) and a consumer (reading
// its inbox). One send fans out to every joined user's inbox. A monotonic
// sequence counter assigned under the room lock gives every message a total
// order consistent across all recipients — the standard trick for getting
// deterministic ordering from a concurrent system.
//
// Note: this file shares the package `threading` with `progress_bar.go` and
// `download_manager.go`. Types are prefixed with `Chat` to avoid collision
// with an unrelated networking chat application in the repo.
package threading

import "sync"

// ChatUserID identifies a chat room member.
type ChatUserID int64

// ChatMessage is one delivered chat message.
type ChatMessage struct {
	Seq      int64
	FromID   ChatUserID
	FromName string
	Text     string
}

// chatInbox is a per-user unread message queue.
type chatInbox struct {
	Name     string
	Messages []ChatMessage
}

// ChatRoom is a broadcast chat room with dynamic membership.
type ChatRoom struct {
	mu      sync.Mutex
	users   map[ChatUserID]*chatInbox
	log     []ChatMessage
	nextID  ChatUserID
	nextSeq int64
}

// NewChatRoom returns an empty room.
func NewChatRoom() *ChatRoom {
	return &ChatRoom{
		users:   make(map[ChatUserID]*chatInbox),
		nextID:  1,
		nextSeq: 1,
	}
}

// Join adds a user to the room and returns their assigned id.
func (r *ChatRoom) Join(name string) ChatUserID {
	r.mu.Lock()
	defer r.mu.Unlock()
	id := r.nextID
	r.nextID++
	r.users[id] = &chatInbox{Name: name}
	return id
}

// Leave removes a user. Future sends skip them; their history remains in the log.
func (r *ChatRoom) Leave(id ChatUserID) {
	r.mu.Lock()
	defer r.mu.Unlock()
	delete(r.users, id)
}

// Send broadcasts `text` from `sender` to every currently-joined inbox
// (sender included). Returns false if sender is not a member.
func (r *ChatRoom) Send(sender ChatUserID, text string) bool {
	r.mu.Lock()
	defer r.mu.Unlock()
	inbox, ok := r.users[sender]
	if !ok {
		return false
	}
	seq := r.nextSeq
	r.nextSeq++
	msg := ChatMessage{
		Seq: seq, FromID: sender, FromName: inbox.Name, Text: text,
	}
	r.log = append(r.log, msg)
	for _, target := range r.users {
		target.Messages = append(target.Messages, msg)
	}
	return true
}

// Read drains and returns the user's inbox; nil slice for unknown users.
func (r *ChatRoom) Read(id ChatUserID) []ChatMessage {
	r.mu.Lock()
	defer r.mu.Unlock()
	inbox, ok := r.users[id]
	if !ok {
		return nil
	}
	out := inbox.Messages
	inbox.Messages = nil
	return out
}

// History returns a copy of every message ever sent.
func (r *ChatRoom) History() []ChatMessage {
	r.mu.Lock()
	defer r.mu.Unlock()
	out := make([]ChatMessage, len(r.log))
	copy(out, r.log)
	return out
}

// UserCount returns the current number of joined users.
func (r *ChatRoom) UserCount() int {
	r.mu.Lock()
	defer r.mu.Unlock()
	return len(r.users)
}
