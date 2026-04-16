# G033 — FTP Program

> File transfer protocol client/server model.

FTP is the first **client-server protocol** in the Rosetta Stone — two distinct roles (client and server) communicating via a defined command language. This is a choreography with exactly two participants and a strict request-response pattern.

The command protocol (LIST, RETR, STOR, DELE) is a **command language between two parties** — like G021's text editor commands but external, crossing a boundary. InnateScript *is* this external command language for the noosphere.

The virtual filesystem is a **remote resource** — the client doesn't have direct access, it must go through the protocol. This is how agents access the vault: through the resolver's protocol, not directly.

Every `@reference` in InnateScript is a request-response. FTP makes that pattern explicit and named.

---

## The Server: A Resource Behind a Protocol

```innate
(define-role ftp-server
  "Holds a virtual filesystem. Responds to commands."

  (state files {})  ;; filename -> content

  (seed [name content]
    "Pre-populate the filesystem."
    (update files (assoc files name content)))

  (handle [command]
    "Parse and dispatch a command string. Return a response string."
    (let [parts (split command " " :max 3)
          cmd   (upper (first parts))]
      (match cmd
        "LIST" -> (if (empty? files)
                    "226 (empty)"
                    (str "226 File list:\n" (join "\n" (sort (keys files)))))
        "RETR" -> (let [name (second parts)]
                    (if-let [content (get files name)]
                      (str "226 Transfer complete:\n" content)
                      (str "550 " name ": No such file")))
        "STOR" -> (let [name    (second parts)
                        content (third parts)]
                    (update files (assoc files name content))
                    (str "226 " name " stored (" (length content) " bytes)"))
        "DELE" -> (let [name (second parts)]
                    (if (has? files name)
                      (do (update files (dissoc files name))
                          (str "250 " name " deleted"))
                      (str "550 " name ": No such file")))
        "SIZE" -> (let [name (second parts)]
                    (if-let [content (get files name)]
                      (str "213 " (length content))
                      (str "550 " name ": No such file")))
        "QUIT" -> "221 Goodbye"
        _      -> (str "502 Command not implemented: " cmd)))))
```

## The Client: A Role That Speaks the Protocol

```innate
(define-role ftp-client
  "Connects to an ftp-server, sends commands, receives responses."

  (state server nil)
  (state connected false)

  (connect [target]
    "Bind to a server instance."
    (set! server target)
    (set! connected true)
    "220 Service ready")

  (send [command]
    "Send a command string to the server. Return the response."
    (if (not connected)
      "421 Not connected"
      (let [response (@server/handle command)]
        (when (= (upper (trim command)) "QUIT")
          (set! connected false)
          (set! server nil))
        response)))

  ;; Convenience methods — named commands
  (list-files []       (send "LIST"))
  (get [name]          (send (str "RETR " name)))
  (put [name content]  (send (str "STOR " name " " content)))
  (delete [name]       (send (str "DELE " name)))
  (size [name]         (send (str "SIZE " name)))
  (quit []             (send "QUIT")))
```

## The Choreography: A Session

```innate
(define-choreography ftp-session
  "A complete FTP interaction between client and server."

  (roles
    (server ftp-server)
    (client ftp-client))

  (sequence
    ;; Seed the server with files
    (@server/seed "readme.txt" "Welcome to the FTP server.")
    (@server/seed "data.csv" "name,value\nalpha,1\nbeta,2")

    ;; Client connects
    (@client/connect server)

    ;; Browse
    (@client/list-files)         ;; -> 226 File list: data.csv, readme.txt

    ;; Download
    (@client/get "readme.txt")   ;; -> 226 Transfer complete: Welcome...

    ;; Upload
    (@client/put "notes.txt" "These are my notes.")

    ;; Browse again — three files now
    (@client/list-files)

    ;; Check size
    (@client/size "data.csv")    ;; -> 213 26

    ;; Delete
    (@client/delete "data.csv")

    ;; Disconnect
    (@client/quit)))             ;; -> 221 Goodbye
```

---

## Choreographic Insight: Vault Synchronization

The FTP pattern maps directly to vault synchronization between local and droplet:

```innate
(define-choreography vault-sync
  "Synchronize vault notes between local filesystem and remote Postgres."

  (roles
    (local   vault-local)    ;; ~/Documents/Droplet-Org/
    (remote  vault-remote))  ;; dpn-api-client -> Postgres

  (sequence
    ;; LIST: what does the remote have?
    (let [remote-notes (@remote/list "vault_notes")]

      ;; RETR: pull notes modified since last sync
      (for [note (changed-since remote-notes last-sync)]
        (@remote/get note)
        (@local/write note))

      ;; STOR: push local changes
      (for [note (local-changes-since last-sync)]
        (let [content (@local/read note)]
          (@remote/put note content))))))
```

The protocol is the same shape: LIST to discover, RETR to pull, STOR to push. FTP invented the choreography that every sync system reuses. The `@reference` syntax makes the request-response boundary visible — you always know when you're crossing from one role to another.
