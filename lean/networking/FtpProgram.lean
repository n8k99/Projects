/-
  FTP Program — file transfer protocol client/server model.
-/

namespace Networking

/-- A virtual filesystem mapping filenames to content. -/
abbrev FileSystem := List (String × String)

/-- A simplified FTP server with a virtual filesystem. -/
structure FtpServer where
  files : FileSystem := []

namespace FtpServer

/-- Seed the virtual filesystem with a file. -/
def addFile (server : FtpServer) (name content : String) : FtpServer :=
  { server with files := (name, content) :: server.files.filter (·.1 != name) }

/-- Look up a file by name. -/
private def lookup (files : FileSystem) (name : String) : Option String :=
  files.find? (·.1 == name) |>.map (·.2)

/-- Remove a file by name. -/
private def remove (files : FileSystem) (name : String) : FileSystem :=
  files.filter (·.1 != name)

/-- Get sorted list of filenames. -/
private def sortedNames (files : FileSystem) : List String :=
  (files.map (·.1)).mergeSort (· < ·)

/-- Split a command string into parts (up to 3: cmd, arg, rest). -/
private def splitCommand (command : String) : List String :=
  let trimmed := command.trim
  let parts := trimmed.splitOn " "
  let parts := parts.filter (· != "")
  match parts with
  | [] => []
  | [cmd] => [cmd]
  | [cmd, arg] => [cmd, arg]
  | cmd :: arg :: rest => [cmd, arg, " ".intercalate rest]

/-- Process a command string and return a (response, updated server) pair. -/
def handle (server : FtpServer) (command : String) : String × FtpServer :=
  let parts := splitCommand command
  match parts with
  | [] => ("500 Syntax error", server)
  | cmd :: args =>
    let ucmd := cmd.toUpper
    match ucmd, args with
    | "LIST", _ =>
      if server.files.isEmpty then
        ("226 (empty)", server)
      else
        let names := sortedNames server.files
        let listing := "\n".intercalate names
        (s!"226 File list:\n{listing}", server)
    | "RETR", [filename] =>
      match lookup server.files filename with
      | some content => (s!"226 Transfer complete:\n{content}", server)
      | none => (s!"550 {filename}: No such file", server)
    | "RETR", _ => ("501 Syntax error in parameters", server)
    | "STOR", [filename, content] =>
      let newServer := server.addFile filename content
      (s!"226 {filename} stored ({content.length} bytes)", newServer)
    | "STOR", _ => ("501 Syntax error in parameters", server)
    | "DELE", [filename] =>
      match lookup server.files filename with
      | some _ =>
        let newServer := { server with files := remove server.files filename }
        (s!"250 {filename} deleted", newServer)
      | none => (s!"550 {filename}: No such file", server)
    | "DELE", _ => ("501 Syntax error in parameters", server)
    | "SIZE", [filename] =>
      match lookup server.files filename with
      | some content => (s!"213 {content.length}", server)
      | none => (s!"550 {filename}: No such file", server)
    | "SIZE", _ => ("501 Syntax error in parameters", server)
    | "QUIT", _ => ("221 Goodbye", server)
    | _, _ => (s!"502 Command not implemented: {ucmd}", server)

end FtpServer

/-- A simplified FTP client that sends commands to an FtpServer. -/
structure FtpClient where
  server : Option FtpServer := none
  connected : Bool := false

namespace FtpClient

/-- Connect the client to a server. -/
def connect (client : FtpClient) (server : FtpServer) : String × FtpClient :=
  ("220 Service ready", { client with server := some server, connected := true })

/-- Send a command and return the response with updated client state. -/
def send (client : FtpClient) (command : String) : String × FtpClient :=
  if !client.connected then
    ("421 Not connected", client)
  else
    match client.server with
    | none => ("421 Not connected", client)
    | some server =>
      let (response, newServer) := server.handle command
      let isQuit := command.trim.toUpper == "QUIT"
      if isQuit then
        (response, { client with server := none, connected := false })
      else
        (response, { client with server := some newServer })

/-- Send LIST command. -/
def listFiles (client : FtpClient) : String × FtpClient :=
  client.send "LIST"

/-- Send RETR command. -/
def get (client : FtpClient) (filename : String) : String × FtpClient :=
  client.send s!"RETR {filename}"

/-- Send STOR command. -/
def put (client : FtpClient) (filename content : String) : String × FtpClient :=
  client.send s!"STOR {filename} {content}"

/-- Send DELE command. -/
def delete (client : FtpClient) (filename : String) : String × FtpClient :=
  client.send s!"DELE {filename}"

/-- Send SIZE command. -/
def size (client : FtpClient) (filename : String) : String × FtpClient :=
  client.send s!"SIZE {filename}"

/-- Send QUIT command. -/
def quit (client : FtpClient) : String × FtpClient :=
  client.send "QUIT"

end FtpClient

/-- Demo of the FTP client/server interaction. -/
def demo : IO Unit := do
  let server := FtpServer.mk []
  let server := server.addFile "readme.txt" "Welcome to the FTP server."
  let server := server.addFile "data.csv" "name,value\nalpha,1\nbeta,2"

  let client := FtpClient.mk none false
  let (resp, client) := client.connect server
  IO.println resp

  IO.println "\n--- LIST ---"
  let (resp, client) := client.listFiles
  IO.println resp

  IO.println "\n--- RETR readme.txt ---"
  let (resp, client) := client.get "readme.txt"
  IO.println resp

  IO.println "\n--- STOR notes.txt ---"
  let (resp, client) := client.put "notes.txt" "These are my notes."
  IO.println resp

  IO.println "\n--- LIST ---"
  let (resp, client) := client.listFiles
  IO.println resp

  IO.println "\n--- SIZE data.csv ---"
  let (resp, client) := client.size "data.csv"
  IO.println resp

  IO.println "\n--- DELE data.csv ---"
  let (resp, client) := client.delete "data.csv"
  IO.println resp

  IO.println "\n--- QUIT ---"
  let (resp, client) := client.quit
  IO.println resp

  IO.println "\n--- After disconnect ---"
  let (resp, _) := client.listFiles
  IO.println resp

end Networking
