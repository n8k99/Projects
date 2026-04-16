-- G034 — Get Atomic Time from Internet Clock
-- Time synchronization protocol: in-process NTP-style client/server.

namespace Networking

/-- Authoritative time source. Wraps a base time with optional offset. -/
structure TimeServer where
  offsetSeconds : Float := 0.0
  deriving Repr

/-- Create a time server using the default clock. -/
def TimeServer.new : TimeServer :=
  { offsetSeconds := 0.0 }

/-- Create a time server shifted by `offset` seconds. -/
def TimeServer.withOffset (offset : Float) : TimeServer :=
  { offsetSeconds := offset }

/-- Return authoritative timestamp given a base time.
    Since Lean lacks direct system clock access, we pass `baseTime` explicitly. -/
def TimeServer.getTime (server : TimeServer) (baseTime : Float) : Float :=
  baseTime + server.offsetSeconds

/-- NTP-style client that queries a TimeServer and tracks synchronization state. -/
structure TimeClient where
  server      : TimeServer
  localOffset : Float := 0.0
  syncOffset  : Float := 0.0
  synced      : Bool  := false
  samples     : List Float := []
  deriving Repr

/-- Create a time client connected to a server. -/
def TimeClient.new (server : TimeServer) : TimeClient :=
  { server := server }

/-- Create a client whose local clock is off by `drift` seconds. -/
def TimeClient.withDrift (server : TimeServer) (drift : Float) : TimeClient :=
  { server := server, localOffset := drift }

/-- Return the client's uncorrected local time. -/
def TimeClient.localTime (client : TimeClient) (baseTime : Float) : Float :=
  baseTime + client.localOffset

/-- Query the server. Returns (serverTimestamp, updatedClient).
    In-process so round trip is effectively zero. -/
def TimeClient.queryTime (client : TimeClient) (baseTime : Float) : Float × TimeClient :=
  let tSend := client.localTime baseTime
  let serverTime := client.server.getTime baseTime
  let tRecv := client.localTime baseTime
  let roundTrip := tRecv - tSend
  let estimatedServerNow := serverTime + roundTrip / 2.0
  let sample := estimatedServerNow - tRecv
  (serverTime, { client with samples := sample :: client.samples })

/-- Compute average of a float list. -/
private def avgList (xs : List Float) : Float :=
  let sum := xs.foldl (· + ·) 0.0
  sum / Float.ofNat xs.length

/-- Return measured offset between local and server clock.
    Positive means local is behind server. Returns (offset, updatedClient). -/
def TimeClient.getOffset (client : TimeClient) (baseTime : Float) : Float × TimeClient :=
  if client.samples.isEmpty then
    let (_, client') := client.queryTime baseTime
    (avgList client'.samples, client')
  else
    (avgList client.samples, client)

/-- Adjust local time estimate by applying the measured offset. -/
def TimeClient.sync (client : TimeClient) (baseTime : Float) : TimeClient :=
  let (offset, client') := client.getOffset baseTime
  { client' with syncOffset := offset, synced := true }

/-- Return local time corrected by the synchronization offset. -/
def TimeClient.getSyncedTime (client : TimeClient) (baseTime : Float) : Float :=
  if client.synced then
    client.localTime baseTime + client.syncOffset
  else
    client.localTime baseTime

/-- Demo: simulate time synchronization. -/
def demo : IO Unit := do
  IO.println "=== G034: Atomic Time Synchronization ==="
  IO.println ""

  let baseTime : Float := 1700000000.0  -- fixed epoch for reproducibility
  let server := TimeServer.withOffset 0.0
  let drift : Float := -3.5
  let client := TimeClient.withDrift server drift

  IO.println s!"Server time:       {server.getTime baseTime}"
  IO.println s!"Client local time: {client.localTime baseTime}"
  IO.println s!"Deliberate drift:  {drift} s"
  IO.println ""

  -- Take 3 samples
  IO.println "Querying server (3 samples)..."
  let (ts1, c1) := client.queryTime baseTime
  IO.println s!"  Sample 1: server reports {ts1}"
  let (ts2, c2) := c1.queryTime baseTime
  IO.println s!"  Sample 2: server reports {ts2}"
  let (ts3, c3) := c2.queryTime baseTime
  IO.println s!"  Sample 3: server reports {ts3}"

  let (offset, c4) := c3.getOffset baseTime
  IO.println s!"\nMeasured offset: {offset} s"

  let c5 := c4.sync baseTime
  IO.println "\nAfter sync:"
  IO.println s!"  Server time:  {server.getTime baseTime}"
  IO.println s!"  Synced time:  {c5.getSyncedTime baseTime}"
  IO.println s!"  Local time:   {c5.localTime baseTime}"

  let syncError := Float.abs (c5.getSyncedTime baseTime - server.getTime baseTime)
  IO.println s!"\n  Sync error: {syncError * 1000.0} ms"
  IO.println s!"  Synced: {c5.synced}"

end Networking
