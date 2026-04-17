// Packet Sniffer — capture and analyze network packets with filtering and statistics.
package networking

import (
	"fmt"
	"sort"
	"strings"
	"sync"
)

// NetProtocol identifies a network protocol.
type NetProtocol string

const (
	ProtoTCP     NetProtocol = "TCP"
	ProtoUDP     NetProtocol = "UDP"
	ProtoICMP    NetProtocol = "ICMP"
	ProtoARP     NetProtocol = "ARP"
	ProtoDNS     NetProtocol = "DNS"
	ProtoHTTP    NetProtocol = "HTTP"
	ProtoUnknown NetProtocol = "UNKNOWN"
)

// NetPacket is a captured network packet.
type NetPacket struct {
	TimestampMs uint64
	SrcIP       string
	DstIP       string
	SrcPort     uint16
	DstPort     uint16
	Protocol    NetProtocol
	Size        int
	Flags       []string
	Payload     []byte
	TTL         uint8
}

func (p NetPacket) String() string {
	flags := ""
	if len(p.Flags) > 0 {
		flags = fmt.Sprintf(" [%s]", strings.Join(p.Flags, ","))
	}
	return fmt.Sprintf("%d %5s %s:%d -> %s:%d %dB%s",
		p.TimestampMs, p.Protocol, p.SrcIP, p.SrcPort, p.DstIP, p.DstPort, p.Size, flags)
}

// NetCaptureFilter defines criteria for packet filtering.
type NetCaptureFilter struct {
	SrcIP    string
	DstIP    string
	Protocol *NetProtocol
	Port     *uint16
	MinSize  int
	MaxSize  int
}

// Matches tests if a packet matches the filter.
func (f *NetCaptureFilter) Matches(pkt *NetPacket) bool {
	if f.SrcIP != "" && f.SrcIP != pkt.SrcIP {
		return false
	}
	if f.DstIP != "" && f.DstIP != pkt.DstIP {
		return false
	}
	if f.Protocol != nil && *f.Protocol != pkt.Protocol {
		return false
	}
	if f.Port != nil && pkt.SrcPort != *f.Port && pkt.DstPort != *f.Port {
		return false
	}
	if f.MinSize > 0 && pkt.Size < f.MinSize {
		return false
	}
	if f.MaxSize > 0 && pkt.Size > f.MaxSize {
		return false
	}
	return true
}

// NetConnKey identifies a bidirectional connection.
type NetConnKey struct {
	IPA      string
	PortA    uint16
	IPB      string
	PortB    uint16
	Protocol NetProtocol
}

// ConnKeyFromPacket creates a normalized connection key.
func ConnKeyFromPacket(pkt *NetPacket) NetConnKey {
	if pkt.SrcIP < pkt.DstIP || (pkt.SrcIP == pkt.DstIP && pkt.SrcPort <= pkt.DstPort) {
		return NetConnKey{pkt.SrcIP, pkt.SrcPort, pkt.DstIP, pkt.DstPort, pkt.Protocol}
	}
	return NetConnKey{pkt.DstIP, pkt.DstPort, pkt.SrcIP, pkt.SrcPort, pkt.Protocol}
}

// NetConnStats tracks statistics for a connection.
type NetConnStats struct {
	Key         NetConnKey
	PacketCount int
	TotalBytes  int
	FirstSeen   uint64
	LastSeen    uint64
}

// PacketCapture captures, filters, and analyzes packets.
type PacketCapture struct {
	mu          sync.RWMutex
	Filter      *NetCaptureFilter
	Packets     []NetPacket
	Connections map[NetConnKey]*NetConnStats
}

// NewPacketCapture creates a capture with an optional filter.
func NewPacketCapture(filter *NetCaptureFilter) *PacketCapture {
	return &PacketCapture{
		Filter:      filter,
		Connections: make(map[NetConnKey]*NetConnStats),
	}
}

// Ingest adds a packet to the capture. Returns true if it passed the filter.
func (c *PacketCapture) Ingest(pkt NetPacket) bool {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.Filter != nil && !c.Filter.Matches(&pkt) {
		return false
	}
	key := ConnKeyFromPacket(&pkt)
	if conn, ok := c.Connections[key]; ok {
		conn.PacketCount++
		conn.TotalBytes += pkt.Size
		if pkt.TimestampMs < conn.FirstSeen {
			conn.FirstSeen = pkt.TimestampMs
		}
		if pkt.TimestampMs > conn.LastSeen {
			conn.LastSeen = pkt.TimestampMs
		}
	} else {
		c.Connections[key] = &NetConnStats{
			Key: key, PacketCount: 1, TotalBytes: pkt.Size,
			FirstSeen: pkt.TimestampMs, LastSeen: pkt.TimestampMs,
		}
	}
	c.Packets = append(c.Packets, pkt)
	return true
}

// TotalPackets returns the packet count.
func (c *PacketCapture) TotalPackets() int {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return len(c.Packets)
}

// TotalBytes returns the total captured bytes.
func (c *PacketCapture) TotalBytes() int {
	c.mu.RLock()
	defer c.mu.RUnlock()
	total := 0
	for _, p := range c.Packets {
		total += p.Size
	}
	return total
}

// ProtocolBreakdown returns packet counts per protocol.
func (c *PacketCapture) ProtocolBreakdown() map[NetProtocol]int {
	c.mu.RLock()
	defer c.mu.RUnlock()
	m := make(map[NetProtocol]int)
	for _, p := range c.Packets {
		m[p.Protocol]++
	}
	return m
}

// TopTalkers returns the top N source IPs by packet count.
func (c *PacketCapture) TopTalkers(n int) []struct {
	IP    string
	Count int
} {
	c.mu.RLock()
	defer c.mu.RUnlock()
	counts := make(map[string]int)
	for _, p := range c.Packets {
		counts[p.SrcIP]++
	}
	type ipCount struct {
		IP    string
		Count int
	}
	var list []ipCount
	for ip, cnt := range counts {
		list = append(list, ipCount{ip, cnt})
	}
	sort.Slice(list, func(i, j int) bool { return list[i].Count > list[j].Count })
	if len(list) > n {
		list = list[:n]
	}
	result := make([]struct {
		IP    string
		Count int
	}, len(list))
	for i, v := range list {
		result[i] = struct {
			IP    string
			Count int
		}{v.IP, v.Count}
	}
	return result
}

// UniqueIPs returns the set of all IPs seen.
func (c *PacketCapture) UniqueIPs() []string {
	c.mu.RLock()
	defer c.mu.RUnlock()
	set := make(map[string]bool)
	for _, p := range c.Packets {
		set[p.SrcIP] = true
		set[p.DstIP] = true
	}
	var ips []string
	for ip := range set {
		ips = append(ips, ip)
	}
	sort.Strings(ips)
	return ips
}

// Summary returns a human-readable capture summary.
func (c *PacketCapture) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "Capture: %d packets, %d bytes\n", c.TotalPackets(), c.TotalBytes())
	fmt.Fprintf(&b, "  Unique IPs: %d\n", len(c.UniqueIPs()))
	fmt.Fprintf(&b, "  Connections: %d\n", len(c.Connections))
	fmt.Fprintln(&b, "  Protocol breakdown:")
	for proto, count := range c.ProtocolBreakdown() {
		fmt.Fprintf(&b, "    %8s: %d\n", proto, count)
	}
	return b.String()
}

// DetectSynFlood finds IPs sending many SYN packets without completing handshakes.
func DetectSynFlood(c *PacketCapture, threshold int) []string {
	c.mu.RLock()
	defer c.mu.RUnlock()
	synCounts := make(map[string]int)
	ackCounts := make(map[string]int)
	for _, p := range c.Packets {
		if p.Protocol == ProtoTCP {
			hasSyn, hasAck := false, false
			for _, f := range p.Flags {
				if f == "SYN" { hasSyn = true }
				if f == "ACK" { hasAck = true }
			}
			if hasSyn && !hasAck { synCounts[p.SrcIP]++ }
			if hasAck { ackCounts[p.SrcIP]++ }
		}
	}
	var suspects []string
	for ip, syns := range synCounts {
		acks := ackCounts[ip]
		if syns > threshold && (acks == 0 || syns/acks > 10) {
			suspects = append(suspects, ip)
		}
	}
	sort.Strings(suspects)
	return suspects
}

// DetectPortScan finds IPs connecting to many ports on a single host.
func DetectPortScan(c *PacketCapture, threshold int) []string {
	c.mu.RLock()
	defer c.mu.RUnlock()
	type srcDst struct{ src, dst string }
	portSets := make(map[srcDst]map[uint16]bool)
	for _, p := range c.Packets {
		if p.Protocol == ProtoTCP {
			for _, f := range p.Flags {
				if f == "SYN" {
					k := srcDst{p.SrcIP, p.DstIP}
					if portSets[k] == nil {
						portSets[k] = make(map[uint16]bool)
					}
					portSets[k][p.DstPort] = true
					break
				}
			}
		}
	}
	seen := make(map[string]bool)
	for k, ports := range portSets {
		if len(ports) >= threshold {
			seen[k.src] = true
		}
	}
	var suspects []string
	for ip := range seen {
		suspects = append(suspects, ip)
	}
	sort.Strings(suspects)
	return suspects
}
