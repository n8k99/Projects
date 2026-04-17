// Small Web Server — HTTP server with routing, static files, and request logging.
package networking

import (
	"fmt"
	"strings"
	"time"
)

// HttpReq represents a parsed HTTP request.
type HttpReq struct {
	Method      string
	Path        string
	Headers     map[string]string
	QueryParams map[string]string
	Body        string
	RemoteAddr  string
}

// ParseHttpReq parses a raw HTTP request string.
func ParseHttpReq(raw string) HttpReq {
	sep := "\r\n"
	if !strings.Contains(raw, "\r\n") {
		sep = "\n"
	}
	lines := strings.Split(raw, sep)
	req := HttpReq{Method: "GET", Path: "/", Headers: make(map[string]string), QueryParams: make(map[string]string)}
	if len(lines) == 0 {
		return req
	}
	parts := strings.SplitN(lines[0], " ", 3)
	if len(parts) >= 1 {
		req.Method = parts[0]
	}
	if len(parts) >= 2 {
		fullPath := parts[1]
		if idx := strings.Index(fullPath, "?"); idx >= 0 {
			req.Path = fullPath[:idx]
			for _, pair := range strings.Split(fullPath[idx+1:], "&") {
				if eq := strings.Index(pair, "="); eq >= 0 {
					req.QueryParams[pair[:eq]] = pair[eq+1:]
				}
			}
		} else {
			req.Path = fullPath
		}
	}
	bodyStart := len(lines)
	for i := 1; i < len(lines); i++ {
		if lines[i] == "" {
			bodyStart = i + 1
			break
		}
		if colon := strings.Index(lines[i], ":"); colon >= 0 {
			req.Headers[strings.ToLower(strings.TrimSpace(lines[i][:colon]))] = strings.TrimSpace(lines[i][colon+1:])
		}
	}
	if bodyStart < len(lines) {
		req.Body = strings.Join(lines[bodyStart:], "\n")
	}
	return req
}

// HttpResp represents an HTTP response.
type HttpResp struct {
	StatusCode int
	StatusText string
	Headers    map[string]string
	Body       string
}

// NewHttpResp creates a response.
func NewHttpResp(code int, text, body string) HttpResp {
	return HttpResp{code, text, make(map[string]string), body}
}

// Serialize produces the wire format.
func (r *HttpResp) Serialize() string {
	if _, ok := r.Headers["Content-Length"]; !ok {
		r.Headers["Content-Length"] = fmt.Sprintf("%d", len(r.Body))
	}
	if _, ok := r.Headers["Content-Type"]; !ok {
		r.Headers["Content-Type"] = "text/html; charset=utf-8"
	}
	var headerLines []string
	for k, v := range r.Headers {
		headerLines = append(headerLines, fmt.Sprintf("%s: %s", k, v))
	}
	return fmt.Sprintf("HTTP/1.1 %d %s\r\n%s\r\n\r\n%s",
		r.StatusCode, r.StatusText, strings.Join(headerLines, "\r\n"), r.Body)
}

// Resp200 creates a 200 response.
func Resp200(body string) HttpResp {
	r := NewHttpResp(200, "OK", body)
	r.Headers["Content-Type"] = "text/html; charset=utf-8"
	return r
}

// Resp404 creates a 404 response.
func Resp404(path string) HttpResp {
	return NewHttpResp(404, "Not Found", "404 Not Found: "+path)
}

// RespJSON creates a JSON response.
func RespJSON(data string) HttpResp {
	r := NewHttpResp(200, "OK", data)
	r.Headers["Content-Type"] = "application/json"
	return r
}

// WebHandlerFunc is an HTTP handler function.
type WebHandlerFunc func(HttpReq) HttpResp

type webRoute struct {
	Method  string
	Path    string
	Handler WebHandlerFunc
}

// AccessLogEntry records a request.
type AccessLogEntry struct {
	Timestamp time.Time
	Method    string
	Path      string
	Status    int
}

// SimpleWebServer is an HTTP server with routing and logging.
type SimpleWebServer struct {
	Name        string
	routes      []webRoute
	staticFiles map[string]struct{ Content, ContentType string }
	AccessLog   []AccessLogEntry
}

// NewSimpleWebServer creates a server.
func NewSimpleWebServer(name string) *SimpleWebServer {
	return &SimpleWebServer{
		Name:        name,
		staticFiles: make(map[string]struct{ Content, ContentType string }),
	}
}

// AddRoute registers a handler.
func (s *SimpleWebServer) AddRoute(method, path string, handler WebHandlerFunc) {
	s.routes = append(s.routes, webRoute{method, path, handler})
}

// ServeStatic registers a static file.
func (s *SimpleWebServer) ServeStatic(path, content, contentType string) {
	s.staticFiles[path] = struct{ Content, ContentType string }{content, contentType}
}

// HandleRequest processes a request and returns a response.
func (s *SimpleWebServer) HandleRequest(req HttpReq) HttpResp {
	// Static files
	if req.Method == "GET" {
		if sf, ok := s.staticFiles[req.Path]; ok {
			r := NewHttpResp(200, "OK", sf.Content)
			r.Headers["Content-Type"] = sf.ContentType
			s.log(req, 200)
			return r
		}
	}
	// Routes
	for _, route := range s.routes {
		if route.Method == req.Method && route.Path == req.Path {
			resp := route.Handler(req)
			s.log(req, resp.StatusCode)
			return resp
		}
	}
	resp := Resp404(req.Path)
	s.log(req, 404)
	return resp
}

// ProcessRaw handles a raw HTTP request string.
func (s *SimpleWebServer) ProcessRaw(raw string) string {
	req := ParseHttpReq(raw)
	resp := s.HandleRequest(req)
	return resp.Serialize()
}

func (s *SimpleWebServer) log(req HttpReq, status int) {
	s.AccessLog = append(s.AccessLog, AccessLogEntry{
		Timestamp: time.Now().UTC(), Method: req.Method, Path: req.Path, Status: status,
	})
}
