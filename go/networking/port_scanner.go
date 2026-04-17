// Port Scanner — probe a host's ports to discover listening services.
package networking

import (
	"fmt"
	"net"
	"sort"
	"strings"
	"sync"
	"time"
)

// PortStatus represents the state of a scanned port.
type PortStatus int

const (
	PortOpen PortStatus = iota
	PortClosed
	PortFiltered
)

func (s PortStatus) String() string {
	switch s {
	case PortOpen:
		return "open"
	case PortClosed:
		return "closed"
	default:
		return "filtered"
	}
}

// PortResult holds the result of scanning a single port.
type PortResult struct {
	Port      int
	Status    PortStatus
	Service   string
	Banner    string
	LatencyMs float64
}

func (r PortResult) String() string {
	svc := ""
	if r.Service != "" {
		svc = fmt.Sprintf(" (%s)", r.Service)
	}
	return fmt.Sprintf("%5d/%s%s (%.1fms)", r.Port, r.Status, svc, r.LatencyMs)
}

// PortScanReport holds the full scan results.
type PortScanReport struct {
	Host          string
	PortsScanned  int
	OpenPorts     []PortResult
	ClosedPorts   int
	FilteredPorts int
	DurationMs    float64
}

// Summary returns a human-readable scan report.
func (r PortScanReport) Summary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "Scan report for %s\n", r.Host)
	fmt.Fprintf(&b, "  Scanned %d ports in %.0fms\n", r.PortsScanned, r.DurationMs)
	fmt.Fprintf(&b, "  Open: %d, Closed: %d, Filtered: %d\n",
		len(r.OpenPorts), r.ClosedPorts, r.FilteredPorts)
	if len(r.OpenPorts) > 0 {
		fmt.Fprintln(&b, "  Open ports:")
		for _, p := range r.OpenPorts {
			fmt.Fprintf(&b, "    %s\n", p)
		}
	}
	return b.String()
}

var wellKnownPorts = map[int]string{
	20: "ftp-data", 21: "ftp", 22: "ssh", 23: "telnet", 25: "smtp",
	53: "dns", 80: "http", 110: "pop3", 123: "ntp", 135: "msrpc",
	139: "netbios-ssn", 143: "imap", 443: "https", 445: "microsoft-ds",
	465: "smtps", 587: "submission", 993: "imaps", 995: "pop3s",
	1433: "mssql", 1521: "oracle", 3306: "mysql", 3389: "ms-wbt-server",
	5432: "postgresql", 5433: "postgresql-alt", 5672: "amqp",
	5900: "vnc", 6379: "redis", 8080: "http-proxy", 8443: "https-alt",
	9200: "elasticsearch", 27017: "mongodb",
}

// LookupService returns the well-known service name for a port.
func LookupService(port int) string {
	if svc, ok := wellKnownPorts[port]; ok {
		return svc
	}
	return ""
}

// PortScanner performs TCP port scans.
type PortScanner struct {
	Timeout    time.Duration
	MaxWorkers int
}

// NewPortScanner creates a scanner with the given timeout and concurrency.
func NewPortScanner(timeoutMs int, maxWorkers int) *PortScanner {
	return &PortScanner{
		Timeout:    time.Duration(timeoutMs) * time.Millisecond,
		MaxWorkers: maxWorkers,
	}
}

// ScanPort probes a single TCP port.
func (s *PortScanner) ScanPort(host string, port int) PortResult {
	addr := fmt.Sprintf("%s:%d", host, port)
	start := time.Now()
	conn, err := net.DialTimeout("tcp", addr, s.Timeout)
	elapsed := float64(time.Since(start).Microseconds()) / 1000.0

	if err != nil {
		status := PortFiltered
		if strings.Contains(err.Error(), "refused") {
			status = PortClosed
		}
		return PortResult{Port: port, Status: status, LatencyMs: elapsed}
	}
	conn.Close()
	return PortResult{
		Port:      port,
		Status:    PortOpen,
		Service:   LookupService(port),
		LatencyMs: elapsed,
	}
}

// ScanRange scans a contiguous range of ports.
func (s *PortScanner) ScanRange(host string, startPort, endPort int) PortScanReport {
	ports := make([]int, 0, endPort-startPort+1)
	for p := startPort; p <= endPort; p++ {
		ports = append(ports, p)
	}
	return s.ScanPorts(host, ports)
}

// ScanCommon scans well-known ports only.
func (s *PortScanner) ScanCommon(host string) PortScanReport {
	ports := make([]int, 0, len(wellKnownPorts))
	for p := range wellKnownPorts {
		ports = append(ports, p)
	}
	sort.Ints(ports)
	return s.ScanPorts(host, ports)
}

// ScanPorts scans a list of ports concurrently.
func (s *PortScanner) ScanPorts(host string, ports []int) PortScanReport {
	start := time.Now()
	var mu sync.Mutex
	var results []PortResult
	sem := make(chan struct{}, s.MaxWorkers)
	var wg sync.WaitGroup

	for _, port := range ports {
		wg.Add(1)
		sem <- struct{}{}
		go func(p int) {
			defer wg.Done()
			defer func() { <-sem }()
			r := s.ScanPort(host, p)
			mu.Lock()
			results = append(results, r)
			mu.Unlock()
		}(port)
	}
	wg.Wait()

	sort.Slice(results, func(i, j int) bool { return results[i].Port < results[j].Port })
	var openPorts []PortResult
	closed, filtered := 0, 0
	for _, r := range results {
		switch r.Status {
		case PortOpen:
			openPorts = append(openPorts, r)
		case PortClosed:
			closed++
		case PortFiltered:
			filtered++
		}
	}

	return PortScanReport{
		Host:          host,
		PortsScanned:  len(ports),
		OpenPorts:     openPorts,
		ClosedPorts:   closed,
		FilteredPorts: filtered,
		DurationMs:    float64(time.Since(start).Microseconds()) / 1000.0,
	}
}

// ComparePortScans compares two scan reports, returning newly opened, closed, and still-open ports.
func ComparePortScans(before, after PortScanReport) (newlyOpened, newlyClosed, stillOpen []int) {
	beforeSet := make(map[int]bool)
	afterSet := make(map[int]bool)
	for _, p := range before.OpenPorts {
		beforeSet[p.Port] = true
	}
	for _, p := range after.OpenPorts {
		afterSet[p.Port] = true
	}
	for p := range afterSet {
		if !beforeSet[p] {
			newlyOpened = append(newlyOpened, p)
		} else {
			stillOpen = append(stillOpen, p)
		}
	}
	for p := range beforeSet {
		if !afterSet[p] {
			newlyClosed = append(newlyClosed, p)
		}
	}
	sort.Ints(newlyOpened)
	sort.Ints(newlyClosed)
	sort.Ints(stillOpen)
	return
}
