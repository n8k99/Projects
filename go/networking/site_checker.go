// Site Checker with Time Scheduling — periodic health monitoring of web endpoints.
package networking

import (
	"fmt"
	"sort"
	"strings"
	"time"
)

// SiteHealthStatus represents the health of a monitored site.
type SiteHealthStatus int

const (
	StatusUp SiteHealthStatus = iota
	StatusDown
	StatusDegraded
	StatusUnknown
)

func (s SiteHealthStatus) String() string {
	switch s {
	case StatusUp:
		return "up"
	case StatusDown:
		return "down"
	case StatusDegraded:
		return "degraded"
	default:
		return "unknown"
	}
}

// SiteCheckConfig configures monitoring for a single site.
type SiteCheckConfig struct {
	URL            string
	Name           string
	IntervalSecs   int
	TimeoutMs      int
	ExpectedStatus int
	MaxResponseMs  float64
}

// SiteCheckResult holds the result of a single check.
type SiteCheckResult struct {
	URL            string
	Status         SiteHealthStatus
	HTTPStatus     int
	ResponseTimeMs float64
	Error          string
	CheckedAt      time.Time
}

// SiteCheckHistory tracks health for one site.
type SiteCheckHistory struct {
	Config              SiteCheckConfig
	Checks              []SiteCheckResult
	ConsecutiveFailures int
}

// CurrentStatus returns the most recent status.
func (h *SiteCheckHistory) CurrentStatus() SiteHealthStatus {
	if len(h.Checks) == 0 {
		return StatusUnknown
	}
	return h.Checks[len(h.Checks)-1].Status
}

// UptimePct returns the percentage of checks that were UP.
func (h *SiteCheckHistory) UptimePct() float64 {
	if len(h.Checks) == 0 {
		return 0
	}
	up := 0
	for _, c := range h.Checks {
		if c.Status == StatusUp {
			up++
		}
	}
	return float64(up) / float64(len(h.Checks)) * 100
}

// AvgResponseMs returns average response time for UP checks.
func (h *SiteCheckHistory) AvgResponseMs() float64 {
	var total float64
	count := 0
	for _, c := range h.Checks {
		if c.Status == StatusUp {
			total += c.ResponseTimeMs
			count++
		}
	}
	if count == 0 {
		return 0
	}
	return total / float64(count)
}

// AddCheck records a check result. Returns a status change message or empty string.
func (h *SiteCheckHistory) AddCheck(r SiteCheckResult) string {
	prev := h.CurrentStatus()
	if r.Status == StatusDown || r.Status == StatusDegraded {
		h.ConsecutiveFailures++
	} else {
		h.ConsecutiveFailures = 0
	}
	h.Checks = append(h.Checks, r)
	if prev != r.Status && prev != StatusUnknown {
		return fmt.Sprintf("%s: %s -> %s", h.Config.Name, prev, r.Status)
	}
	return ""
}

// Summary returns a one-line status.
func (h *SiteCheckHistory) CheckSummary() string {
	return fmt.Sprintf("%s: %s | uptime %.1f%% | avg %.0fms | %d checks | %d consecutive failures",
		h.Config.Name, h.CurrentStatus(), h.UptimePct(), h.AvgResponseMs(),
		len(h.Checks), h.ConsecutiveFailures)
}

// SiteMonitor manages multiple monitored sites.
type SiteMonitor struct {
	Histories map[string]*SiteCheckHistory
}

// NewSiteMonitor creates a monitor.
func NewSiteMonitor() *SiteMonitor {
	return &SiteMonitor{Histories: make(map[string]*SiteCheckHistory)}
}

// AddSite registers a site for monitoring.
func (m *SiteMonitor) AddSite(config SiteCheckConfig) {
	if config.Name == "" {
		config.Name = config.URL
	}
	m.Histories[config.URL] = &SiteCheckHistory{Config: config}
}

// RecordCheck records a check result.
func (m *SiteMonitor) RecordCheck(r SiteCheckResult) string {
	if h, ok := m.Histories[r.URL]; ok {
		return h.AddCheck(r)
	}
	return ""
}

// SitesByStatus returns URLs with the given status.
func (m *SiteMonitor) SitesByStatus(status SiteHealthStatus) []string {
	var urls []string
	for url, h := range m.Histories {
		if h.CurrentStatus() == status {
			urls = append(urls, url)
		}
	}
	sort.Strings(urls)
	return urls
}

// Dashboard returns a text dashboard.
func (m *SiteMonitor) Dashboard() string {
	var b strings.Builder
	fmt.Fprintln(&b, "Site Monitor Dashboard")
	fmt.Fprintln(&b, strings.Repeat("=", 60))
	var urls []string
	for url := range m.Histories {
		urls = append(urls, url)
	}
	sort.Strings(urls)
	for _, url := range urls {
		fmt.Fprintln(&b, m.Histories[url].CheckSummary())
	}
	return b.String()
}
