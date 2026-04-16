# G035 — Chat Application (Networking)

> Multi-room chat with users and message history.

```yaml
id: G035
title: Chat Application (Networking)
category: networking
requires: [G022-rss-feed, G025-journal, G033-event-system]
provides: [multi-party-choreography, pub-sub-routing, dynamic-membership]
```

## Insight: The First Multi-Party Choreography

Every pattern before this was **bilateral** — one producer, one consumer; one writer, one reader; one publisher, many subscribers. Chat breaks the symmetry: **N users, M rooms, messages fan out many-to-many**. The chat room IS a topic in the pub-sub sense, but membership is dynamic — agents join and leave mid-choreography, introducing **lifecycle** into what was previously a static dance.

This is the pattern that makes the noosphere feel *inhabited*.

## Insight: Message Routing Is Pub-Sub Evolved

G022 (RSS) gave us one-to-many broadcast. Chat extends this to **many-to-many** — every member of a room is simultaneously publisher and subscriber. The room itself is the topic, the membership list is the subscription registry, and `send_message` is `publish` scoped to a channel.

The chathud on Nathan's desktop IS this architecture — it reads and writes the `conversations` table via `dpn-ipc`, routing messages through rooms (channels). Same bones, six languages.

## Insight: History Is a Scoped Journal

`get_messages(room, since=None)` is exactly G025's journal query — `get_entries(since=None)` — but scoped to a room. Message history is journaling with a namespace. The room is the journal, the message is the entry, the timestamp is the cursor.

## Insight: Dynamic Membership as Lifecycle

`join_room` and `leave_room` introduce something new to our choreographies: **agents entering and exiting mid-dance**. Previous patterns assumed fixed participants. Chat forces us to handle:
- A user joining a room that already has history (do they see backlog?)
- A user leaving while messages continue (ghost presence)
- The empty room (does it persist or dissolve?)

This is the seed of agent lifecycle management in the noosphere.

## Choreographic Case: Executive Ghost Communication

Rooms map to projects and domains. Kathryn and Eliana coordinate in `#operations`. Sylvia and Lena exchange notes in `#editorial`. The executive ghosts don't need sockets — they need **message routing with membership semantics**. The in-process model here is the prototype; the dpn-api-client is the production instance.

## Structures

```innate
(defstruct message
  sender   : String
  room     : String
  text     : String
  timestamp : Instant)

(defstruct chat-room
  name     : String
  members  : Set<String>
  messages : List<Message>)

(defstruct chat-user
  name  : String
  rooms : Set<String>)

(defstruct chat-server
  rooms : Map<String, ChatRoom>
  users : Map<String, ChatUser>)
```

## Protocol

```innate
(defprotocol ChatOps
  (register-user [server name] -> ChatUser
    "Add a user to the server. Error if name is taken.")

  (create-room [server room-name] -> ChatRoom
    "Create a new room. Error if it already exists.")

  (join-room [server user-name room-name] -> ()
    "Add user to room's member set, room to user's room set.")

  (leave-room [server user-name room-name] -> ()
    "Remove user from room, room from user.")

  (send-message [server user-name room-name text] -> Message
    "Post a message. User must be a member of the room.")

  (get-messages [server room-name &optional since] -> List<Message>
    "Return messages from a room. If SINCE is provided, filter by timestamp.")

  (list-rooms [server] -> List<String>
    "Return all room names, sorted.")

  (list-users [server room-name] -> List<String>
    "Return sorted member names of a room."))
```

## Resolution Chain

```innate
;; The room IS a topic — send-message resolves through pub-sub routing
(resolve send-message [server user room text]
  (let [r (lookup-room server room)]
    (assert (member? user (room-members r))
            "User must be a member to send")
    (let [msg (make-message :sender user :room room :text text)]
      (append! (room-messages r) msg)
      msg)))

;; get-messages resolves as a scoped journal query
(resolve get-messages [server room &optional since]
  (let [msgs (room-messages (lookup-room server room))]
    (if since
      (filter (lambda [m] (>= (message-timestamp m) since)) msgs)
      msgs)))

;; join-room is a lifecycle event — mutual registration
(resolve join-room [server user-name room-name]
  (let [user (lookup-user server user-name)
        room (lookup-room server room-name)]
    (adjoin! (room-members room) user-name)
    (adjoin! (user-rooms user) room-name)))

;; leave-room is the inverse lifecycle event
(resolve leave-room [server user-name room-name]
  (let [user (lookup-user server user-name)
        room (lookup-room server room-name)]
    (discard! (room-members room) user-name)
    (discard! (user-rooms user) room-name)))
```

## Demo: Three Users, Two Rooms

```innate
(let [server (make-chat-server)]
  ;; Register
  (register-user server "alice")
  (register-user server "bob")
  (register-user server "carol")

  ;; Create rooms
  (create-room server "#general")
  (create-room server "#engineering")

  ;; Join
  (join-room server "alice" "#general")
  (join-room server "bob" "#general")
  (join-room server "carol" "#general")
  (join-room server "alice" "#engineering")
  (join-room server "bob" "#engineering")

  ;; Exchange messages
  (send-message server "alice" "#general" "Hello everyone!")
  (send-message server "bob" "#general" "Hey Alice!")
  (send-message server "carol" "#general" "Good morning!")
  (send-message server "alice" "#engineering" "Sprint planning at 2pm")
  (send-message server "bob" "#engineering" "Sounds good, I'll be there")

  ;; Query history
  (get-messages server "#general")        ;; => all 3 messages
  (get-messages server "#engineering")    ;; => 2 messages

  ;; Dynamic membership
  (leave-room server "carol" "#general")
  (list-users server "#general")          ;; => ("alice" "bob")
  )
```

## Cross-References

- **G022 RSS Feed**: One-to-many broadcast — chat extends to many-to-many
- **G025 Journal**: Message history is a room-scoped journal
- **G033 Event System**: `send-message` could emit events for cross-room notification
- **chathud** (`~/.config/quickshell/chathud/shell.qml`): The living instance of this pattern, reading/writing `conversations` via `dpn-ipc`
- **dpn-api-client**: Production message routing through Postgres `conversations` table
