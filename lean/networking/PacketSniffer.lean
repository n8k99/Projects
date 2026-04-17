-- Packet Sniffer — capture and analyze network packets with filtering and statistics.
-- Pure data model: packets, captures, filters, protocol breakdown, and anomaly detection.

namespace Networking

inductive NetProto where
  | tcp | udp | icmp | arp | dns | http | unknown
  deriving Repr, BEq, Hashable

instance : ToString NetProto where
  toString | .tcp => "TCP" | .udp => "UDP" | .icmp => "ICMP"
           | .arp => "ARP" | .dns => "DNS" | .http => "HTTP" | .unknown => "UNKNOWN"

structure NetPacket where
  timestampMs : Nat
  srcIp : String
  dstIp : String
  srcPort : Nat
  dstPort : Nat
  protocol : NetProto
  size : Nat
  flags : List String := []
  ttl : Nat := 64
  deriving Repr

structure NetCaptureFilter where
  srcIp : String := ""
  dstIp : String := ""
  protocol : Option NetProto := none
  port : Option Nat := none
  minSize : Nat := 0
  maxSize : Nat := 0
  deriving Repr

def NetCaptureFilter.matches (f : NetCaptureFilter) (p : NetPacket) : Bool :=
  (f.srcIp.isEmpty || f.srcIp == p.srcIp) &&
  (f.dstIp.isEmpty || f.dstIp == p.dstIp) &&
  (match f.protocol with | none => true | some pr => pr == p.protocol) &&
  (match f.port with | none => true | some pt => p.srcPort == pt || p.dstPort == pt) &&
  (f.minSize == 0 || p.size >= f.minSize) &&
  (f.maxSize == 0 || p.size <= f.maxSize)

structure NetCaptureState where
  filter : Option NetCaptureFilter := none
  packets : List NetPacket := []
  deriving Repr

namespace NetCaptureState

def ingest (c : NetCaptureState) (pkt : NetPacket) : NetCaptureState × Bool :=
  match c.filter with
  | some f => if f.matches pkt then ({ c with packets := c.packets ++ [pkt] }, true) else (c, false)
  | none => ({ c with packets := c.packets ++ [pkt] }, true)

def totalPackets (c : NetCaptureState) : Nat := c.packets.length

def totalBytes (c : NetCaptureState) : Nat :=
  c.packets.foldl (init := 0) fun acc p => acc + p.size

def uniqueIPs (c : NetCaptureState) : List String :=
  let ips := c.packets.foldl (init := []) fun acc p =>
    let acc := if acc.contains p.srcIp then acc else acc ++ [p.srcIp]
    if acc.contains p.dstIp then acc else acc ++ [p.dstIp]
  ips

def protocolBreakdown (c : NetCaptureState) : List (NetProto × Nat) :=
  let protos := c.packets.map (·.protocol) |>.eraseDups
  protos.map fun pr => (pr, c.packets.filter (·.protocol == pr) |>.length)

def topTalkers (c : NetCaptureState) (n : Nat := 10) : List (String × Nat) :=
  let ips := c.packets.map (·.srcIp) |>.eraseDups
  let counted := ips.map fun ip => (ip, c.packets.filter (·.srcIp == ip) |>.length)
  (counted.mergeSort (fun a b => a.2 > b.2)).take n

end NetCaptureState

-- SYN flood detection
def detectSynFlood (c : NetCaptureState) (threshold : Nat := 100) : List String :=
  let tcpPkts := c.packets.filter (·.protocol == .tcp)
  let synPkts := tcpPkts.filter fun p => p.flags.contains "SYN" && !p.flags.contains "ACK"
  let ackPkts := tcpPkts.filter fun p => p.flags.contains "ACK"
  let srcIps := synPkts.map (·.srcIp) |>.eraseDups
  srcIps.filter fun ip =>
    let syns := synPkts.filter (·.srcIp == ip) |>.length
    let acks := ackPkts.filter (·.srcIp == ip) |>.length
    syns > threshold && (acks == 0 || syns / acks > 10)

-- Port scan detection
def detectPortScan (c : NetCaptureState) (threshold : Nat := 20) : List String :=
  let synPkts := c.packets.filter fun p => p.protocol == .tcp && p.flags.contains "SYN"
  let srcIps := synPkts.map (·.srcIp) |>.eraseDups
  srcIps.filter fun ip =>
    let targets := synPkts.filter (·.srcIp == ip)
    let uniquePorts := targets.map (·.dstPort) |>.eraseDups
    uniquePorts.length >= threshold

-- Demo
def packetSnifferDemo : IO Unit := do
  let pkts : List NetPacket := [
    { timestampMs := 0, srcIp := "10.0.0.1", dstIp := "10.0.0.2",
      srcPort := 1234, dstPort := 80, protocol := .tcp, size := 100, flags := ["SYN"] },
    { timestampMs := 1, srcIp := "10.0.0.2", dstIp := "10.0.0.1",
      srcPort := 80, dstPort := 1234, protocol := .tcp, size := 60, flags := ["SYN", "ACK"] },
    { timestampMs := 2, srcIp := "10.0.0.3", dstIp := "10.0.0.2",
      srcPort := 5555, dstPort := 53, protocol := .udp, size := 50 }
  ]
  let cap := pkts.foldl (init := ({} : NetCaptureState)) fun c p => (c.ingest p).1
  IO.println s!"Packets: {cap.totalPackets}, Bytes: {cap.totalBytes}"
  IO.println s!"Unique IPs: {cap.uniqueIPs.length}"
  for (proto, count) in cap.protocolBreakdown do
    IO.println s!"  {proto}: {count}"

end Networking
