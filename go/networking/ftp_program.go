// Package networking provides simplified network protocol implementations.
//
// FTP Program — file transfer protocol client/server model.
package networking

import (
	"fmt"
	"sort"
	"strings"
)

// FtpServer holds a virtual filesystem and handles FTP commands.
type FtpServer struct {
	files map[string]string
}

// NewFtpServer creates an FtpServer with an empty virtual filesystem.
func NewFtpServer() *FtpServer {
	return &FtpServer{files: make(map[string]string)}
}

// AddFile seeds the virtual filesystem with a file.
func (s *FtpServer) AddFile(name, content string) {
	s.files[name] = content
}

// Handle processes a command string and returns a response string.
func (s *FtpServer) Handle(command string) string {
	trimmed := strings.TrimSpace(command)
	parts := strings.SplitN(trimmed, " ", 3)
	if len(parts) == 0 || parts[0] == "" {
		return "500 Syntax error"
	}

	cmd := strings.ToUpper(parts[0])

	switch cmd {
	case "LIST":
		if len(s.files) == 0 {
			return "226 (empty)"
		}
		names := make([]string, 0, len(s.files))
		for name := range s.files {
			names = append(names, name)
		}
		sort.Strings(names)
		return "226 File list:\n" + strings.Join(names, "\n")

	case "RETR":
		if len(parts) < 2 {
			return "501 Syntax error in parameters"
		}
		filename := parts[1]
		content, ok := s.files[filename]
		if !ok {
			return fmt.Sprintf("550 %s: No such file", filename)
		}
		return fmt.Sprintf("226 Transfer complete:\n%s", content)

	case "STOR":
		if len(parts) < 3 {
			return "501 Syntax error in parameters"
		}
		filename := parts[1]
		content := parts[2]
		s.files[filename] = content
		return fmt.Sprintf("226 %s stored (%d bytes)", filename, len(content))

	case "DELE":
		if len(parts) < 2 {
			return "501 Syntax error in parameters"
		}
		filename := parts[1]
		if _, ok := s.files[filename]; !ok {
			return fmt.Sprintf("550 %s: No such file", filename)
		}
		delete(s.files, filename)
		return fmt.Sprintf("250 %s deleted", filename)

	case "SIZE":
		if len(parts) < 2 {
			return "501 Syntax error in parameters"
		}
		filename := parts[1]
		content, ok := s.files[filename]
		if !ok {
			return fmt.Sprintf("550 %s: No such file", filename)
		}
		return fmt.Sprintf("213 %d", len(content))

	case "QUIT":
		return "221 Goodbye"

	default:
		return fmt.Sprintf("502 Command not implemented: %s", cmd)
	}
}

// FtpClient sends commands to an FtpServer instance.
type FtpClient struct {
	server    *FtpServer
	connected bool
}

// NewFtpClient creates a disconnected FTP client.
func NewFtpClient() *FtpClient {
	return &FtpClient{}
}

// Connect attaches the client to a server.
func (c *FtpClient) Connect(server *FtpServer) string {
	c.server = server
	c.connected = true
	return "220 Service ready"
}

// Send transmits a command to the connected server and returns the response.
func (c *FtpClient) Send(command string) string {
	if !c.connected || c.server == nil {
		return "421 Not connected"
	}
	response := c.server.Handle(command)
	if strings.EqualFold(strings.TrimSpace(command), "QUIT") {
		c.connected = false
		c.server = nil
	}
	return response
}

// ListFiles sends a LIST command.
func (c *FtpClient) ListFiles() string {
	return c.Send("LIST")
}

// Get sends a RETR command.
func (c *FtpClient) Get(filename string) string {
	return c.Send("RETR " + filename)
}

// Put sends a STOR command.
func (c *FtpClient) Put(filename, content string) string {
	return c.Send(fmt.Sprintf("STOR %s %s", filename, content))
}

// Delete sends a DELE command.
func (c *FtpClient) Delete(filename string) string {
	return c.Send("DELE " + filename)
}

// Size sends a SIZE command.
func (c *FtpClient) Size(filename string) string {
	return c.Send("SIZE " + filename)
}

// Quit sends a QUIT command and disconnects.
func (c *FtpClient) Quit() string {
	return c.Send("QUIT")
}
