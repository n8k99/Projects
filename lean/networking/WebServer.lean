-- Small Web Server — HTTP server with routing and request handling.

namespace Networking

structure HttpReq' where
  method : String := "GET"
  path : String := "/"
  headers : List (String × String) := []
  queryParams : List (String × String) := []
  body : String := ""
  deriving Repr

structure HttpResp' where
  statusCode : Nat := 200
  statusText : String := "OK"
  headers : List (String × String) := []
  body : String := ""
  deriving Repr

def resp200' (body : String) : HttpResp' :=
  { statusCode := 200, statusText := "OK", body, headers := [("Content-Type", "text/html")] }

def resp404' (path : String) : HttpResp' :=
  { statusCode := 404, statusText := "Not Found", body := s!"404 Not Found: {path}" }

def respJson' (data : String) : HttpResp' :=
  { statusCode := 200, statusText := "OK", body := data, headers := [("Content-Type", "application/json")] }

-- Simple request parsing
def parseHttpReq' (raw : String) : HttpReq' :=
  let lines := raw.splitOn "\n" |>.map (·.trimRight)
  match lines.head? with
  | none => {}
  | some firstLine =>
    let parts := firstLine.splitOn " "
    let method := parts[0]?.getD "GET"
    let fullPath := parts[1]?.getD "/"
    let (path, qp) := match fullPath.splitOn "?" with
      | [p] => (p, [])
      | [p, qs] => (p, qs.splitOn "&" |>.filterMap fun pair =>
        match pair.splitOn "=" with
        | [k, v] => some (k, v)
        | _ => none)
      | _ => (fullPath, [])
    { method, path, queryParams := qp }

-- Route and server
structure WebRoute where
  method : String
  path : String
  handlerName : String  -- symbolic reference
  deriving Repr

structure WebServerState where
  name : String := "rosetta-server"
  routes : List WebRoute := []
  staticFiles : List (String × String × String) := []  -- (path, content, contentType)
  accessLog : List (String × String × Nat) := []  -- (method, path, status)
  deriving Repr

namespace WebServerState

def addRoute (s : WebServerState) (method path handler : String) : WebServerState :=
  { s with routes := s.routes ++ [{ method, path, handlerName := handler }] }

def serveStatic (s : WebServerState) (path content contentType : String) : WebServerState :=
  { s with staticFiles := s.staticFiles ++ [(path, content, contentType)] }

-- Handle a request with static files and route matching
def handleRequest (s : WebServerState)
    (req : HttpReq')
    (handlers : List (String × (HttpReq' → HttpResp')))
    : WebServerState × HttpResp' :=
  -- Check static files
  if req.method == "GET" then
    match s.staticFiles.find? (·.1 == req.path) with
    | some (_, content, ct) =>
      let resp : HttpResp' := { statusCode := 200, statusText := "OK", body := content, headers := [("Content-Type", ct)] }
      ({ s with accessLog := s.accessLog ++ [(req.method, req.path, 200)] }, resp)
    | none => matchRoute s req handlers
  else matchRoute s req handlers
where
  matchRoute (s : WebServerState) (req : HttpReq') (handlers : List (String × (HttpReq' → HttpResp')))
      : WebServerState × HttpResp' :=
    match s.routes.find? (fun r => r.method == req.method && r.path == req.path) with
    | some route =>
      match handlers.find? (·.1 == route.handlerName) with
      | some (_, handler) =>
        let resp := handler req
        ({ s with accessLog := s.accessLog ++ [(req.method, req.path, resp.statusCode)] }, resp)
      | none => ({ s with accessLog := s.accessLog ++ [(req.method, req.path, 500)] },
                  { statusCode := 500, statusText := "No handler", body := "Handler not found" })
    | none =>
      ({ s with accessLog := s.accessLog ++ [(req.method, req.path, 404)] }, resp404' req.path)

end WebServerState

def webServerDemo : IO Unit := do
  let s := WebServerState.addRoute { name := "test" } "GET" "/" "home"
  let s := s.addRoute "GET" "/health" "health"
  let s := s.serveStatic "/style.css" "body{}" "text/css"
  let handlers := [
    ("home", fun _ => resp200' "<h1>Hello</h1>"),
    ("health", fun _ => respJson' "{\"ok\":true}")
  ]
  let req := parseHttpReq' "GET / HTTP/1.1"
  let (s, resp) := s.handleRequest req handlers
  IO.println s!"Status: {resp.statusCode}, Body: {resp.body}"
  IO.println s!"Access log: {s.accessLog.length} entries"

end Networking
