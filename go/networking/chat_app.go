// Chat Application (Networking) — multi-room chat with users and history.
package networking

import (
	"fmt"
	"sort"
	"sync"
	"time"
)

// Message represents a single chat message.
type Message struct {
	Sender    string
	Room      string
	Text      string
	Timestamp time.Time
}

// ChatRoom holds messages, a name, and tracks members.
type ChatRoom struct {
	Name     string
	Members  map[string]bool
	Messages []Message
}

// ChatUser tracks a user's name and joined rooms.
type ChatUser struct {
	Name  string
	Rooms map[string]bool
}

// ChatServer manages rooms, users, and message routing.
type ChatServer struct {
	mu    sync.RWMutex
	rooms map[string]*ChatRoom
	users map[string]*ChatUser
}

// NewChatServer creates a new ChatServer.
func NewChatServer() *ChatServer {
	return &ChatServer{
		rooms: make(map[string]*ChatRoom),
		users: make(map[string]*ChatUser),
	}
}

// RegisterUser adds a new user to the server.
func (s *ChatServer) RegisterUser(name string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, exists := s.users[name]; exists {
		return fmt.Errorf("user '%s' already exists", name)
	}
	s.users[name] = &ChatUser{Name: name, Rooms: make(map[string]bool)}
	return nil
}

// CreateRoom creates a new chat room.
func (s *ChatServer) CreateRoom(roomName string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, exists := s.rooms[roomName]; exists {
		return fmt.Errorf("room '%s' already exists", roomName)
	}
	s.rooms[roomName] = &ChatRoom{
		Name:    roomName,
		Members: make(map[string]bool),
	}
	return nil
}

// JoinRoom adds a user to a room.
func (s *ChatServer) JoinRoom(userName, roomName string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	user, ok := s.users[userName]
	if !ok {
		return fmt.Errorf("no such user: '%s'", userName)
	}
	room, ok := s.rooms[roomName]
	if !ok {
		return fmt.Errorf("no such room: '%s'", roomName)
	}
	room.Members[userName] = true
	user.Rooms[roomName] = true
	return nil
}

// LeaveRoom removes a user from a room.
func (s *ChatServer) LeaveRoom(userName, roomName string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	user, ok := s.users[userName]
	if !ok {
		return fmt.Errorf("no such user: '%s'", userName)
	}
	room, ok := s.rooms[roomName]
	if !ok {
		return fmt.Errorf("no such room: '%s'", roomName)
	}
	delete(room.Members, userName)
	delete(user.Rooms, roomName)
	return nil
}

// SendMessage posts a message from a user to a room.
func (s *ChatServer) SendMessage(userName, roomName, text string) (Message, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, ok := s.users[userName]; !ok {
		return Message{}, fmt.Errorf("no such user: '%s'", userName)
	}
	room, ok := s.rooms[roomName]
	if !ok {
		return Message{}, fmt.Errorf("no such room: '%s'", roomName)
	}
	if !room.Members[userName] {
		return Message{}, fmt.Errorf("user '%s' is not a member of room '%s'", userName, roomName)
	}
	msg := Message{
		Sender:    userName,
		Room:      roomName,
		Text:      text,
		Timestamp: time.Now().UTC(),
	}
	room.Messages = append(room.Messages, msg)
	return msg, nil
}

// GetMessages returns messages from a room, optionally filtered by a since timestamp.
// If since is nil, all messages are returned.
func (s *ChatServer) GetMessages(roomName string, since *time.Time) ([]Message, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	room, ok := s.rooms[roomName]
	if !ok {
		return nil, fmt.Errorf("no such room: '%s'", roomName)
	}
	if since == nil {
		result := make([]Message, len(room.Messages))
		copy(result, room.Messages)
		return result, nil
	}
	var result []Message
	for _, m := range room.Messages {
		if !m.Timestamp.Before(*since) {
			result = append(result, m)
		}
	}
	return result, nil
}

// ListRooms returns the names of all rooms, sorted.
func (s *ChatServer) ListRooms() []string {
	s.mu.RLock()
	defer s.mu.RUnlock()
	names := make([]string, 0, len(s.rooms))
	for name := range s.rooms {
		names = append(names, name)
	}
	sort.Strings(names)
	return names
}

// ListUsers returns the sorted member names of a room.
func (s *ChatServer) ListUsers(roomName string) ([]string, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	room, ok := s.rooms[roomName]
	if !ok {
		return nil, fmt.Errorf("no such room: '%s'", roomName)
	}
	members := make([]string, 0, len(room.Members))
	for name := range room.Members {
		members = append(members, name)
	}
	sort.Strings(members)
	return members, nil
}
