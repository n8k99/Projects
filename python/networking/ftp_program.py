"""FTP Program — file transfer protocol client/server model."""


class FtpServer:
    """A simplified FTP server with a virtual filesystem."""

    def __init__(self):
        self._files: dict[str, str] = {}

    def add_file(self, name: str, content: str) -> None:
        """Seed the virtual filesystem with a file."""
        self._files[name] = content

    def handle(self, command: str) -> str:
        """Process a command string and return a response string."""
        parts = command.strip().split(None, 2)
        if not parts:
            return "500 Syntax error"

        cmd = parts[0].upper()

        if cmd == "LIST":
            if not self._files:
                return "226 (empty)"
            listing = "\n".join(sorted(self._files.keys()))
            return f"226 File list:\n{listing}"

        elif cmd == "RETR":
            if len(parts) < 2:
                return "501 Syntax error in parameters"
            filename = parts[1]
            if filename not in self._files:
                return f"550 {filename}: No such file"
            return f"226 Transfer complete:\n{self._files[filename]}"

        elif cmd == "STOR":
            if len(parts) < 3:
                return "501 Syntax error in parameters"
            filename = parts[1]
            content = parts[2]
            self._files[filename] = content
            return f"226 {filename} stored ({len(content)} bytes)"

        elif cmd == "DELE":
            if len(parts) < 2:
                return "501 Syntax error in parameters"
            filename = parts[1]
            if filename not in self._files:
                return f"550 {filename}: No such file"
            del self._files[filename]
            return f"250 {filename} deleted"

        elif cmd == "SIZE":
            if len(parts) < 2:
                return "501 Syntax error in parameters"
            filename = parts[1]
            if filename not in self._files:
                return f"550 {filename}: No such file"
            return f"213 {len(self._files[filename])}"

        elif cmd == "QUIT":
            return "221 Goodbye"

        else:
            return f"502 Command not implemented: {cmd}"


class FtpClient:
    """A simplified FTP client that sends commands to an FtpServer."""

    def __init__(self):
        self._server: FtpServer | None = None
        self._connected = False

    def connect(self, server: FtpServer) -> str:
        """Connect to an FTP server instance."""
        self._server = server
        self._connected = True
        return "220 Service ready"

    def send(self, command: str) -> str:
        """Send a command to the connected server and return the response."""
        if not self._connected or self._server is None:
            return "421 Not connected"
        response = self._server.handle(command)
        if command.strip().upper() == "QUIT":
            self._connected = False
            self._server = None
        return response

    def list_files(self) -> str:
        return self.send("LIST")

    def get(self, filename: str) -> str:
        return self.send(f"RETR {filename}")

    def put(self, filename: str, content: str) -> str:
        return self.send(f"STOR {filename} {content}")

    def delete(self, filename: str) -> str:
        return self.send(f"DELE {filename}")

    def size(self, filename: str) -> str:
        return self.send(f"SIZE {filename}")

    def quit(self) -> str:
        return self.send("QUIT")


if __name__ == "__main__":
    # Set up server with initial files
    server = FtpServer()
    server.add_file("readme.txt", "Welcome to the FTP server.")
    server.add_file("data.csv", "name,value\nalpha,1\nbeta,2")

    # Connect client
    client = FtpClient()
    print(client.connect(server))

    # List files
    print("\n--- LIST ---")
    print(client.list_files())

    # Download a file
    print("\n--- RETR readme.txt ---")
    print(client.get("readme.txt"))

    # Upload a new file
    print("\n--- STOR notes.txt ---")
    print(client.put("notes.txt", "These are my notes."))

    # List again to see the new file
    print("\n--- LIST ---")
    print(client.list_files())

    # Check file size
    print("\n--- SIZE data.csv ---")
    print(client.size("data.csv"))

    # Delete a file
    print("\n--- DELE data.csv ---")
    print(client.delete("data.csv"))

    # List after deletion
    print("\n--- LIST ---")
    print(client.list_files())

    # Quit
    print("\n--- QUIT ---")
    print(client.quit())

    # Try to send after quit
    print("\n--- After disconnect ---")
    print(client.list_files())
