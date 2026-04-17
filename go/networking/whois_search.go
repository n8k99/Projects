// Whois Search Tool — query and parse domain registration data.
package networking

import (
	"fmt"
	"strings"
)

// WhoisContact holds contact info from a whois record.
type WhoisContact struct {
	Name         string
	Organization string
	Email        string
	City         string
	State        string
	Country      string
}

// Summary returns a one-line contact summary.
func (c WhoisContact) Summary() string {
	var parts []string
	for _, s := range []string{c.Name, c.Organization, c.City, c.Country} {
		if s != "" {
			parts = append(parts, s)
		}
	}
	if len(parts) == 0 {
		return "(redacted)"
	}
	return strings.Join(parts, ", ")
}

// WhoisRecord holds parsed whois data.
type WhoisRecord struct {
	Domain      string
	Registrar   string
	Registrant  *WhoisContact
	NameServers []string
	Status      []string
	CreatedDate string
	UpdatedDate string
	ExpiryDate  string
	DNSSEC      string
	RawText     string
}

// TLD returns the top-level domain.
func (r WhoisRecord) TLD() string {
	parts := strings.Split(r.Domain, ".")
	if len(parts) > 1 {
		return parts[len(parts)-1]
	}
	return ""
}

// Summary returns a multi-line summary.
func (r WhoisRecord) WhoIsSummary() string {
	var b strings.Builder
	fmt.Fprintf(&b, "Domain: %s\n", r.Domain)
	if r.Registrar != "" {
		fmt.Fprintf(&b, "  Registrar: %s\n", r.Registrar)
	}
	if r.Registrant != nil {
		fmt.Fprintf(&b, "  Registrant: %s\n", r.Registrant.Summary())
	}
	if len(r.NameServers) > 0 {
		fmt.Fprintf(&b, "  Name Servers: %s\n", strings.Join(r.NameServers, ", "))
	}
	if r.CreatedDate != "" {
		fmt.Fprintf(&b, "  Created: %s\n", r.CreatedDate)
	}
	if r.ExpiryDate != "" {
		fmt.Fprintf(&b, "  Expires: %s\n", r.ExpiryDate)
	}
	if len(r.Status) > 0 {
		fmt.Fprintf(&b, "  Status: %s\n", strings.Join(r.Status, ", "))
	}
	return b.String()
}

func findWhoisField(raw, key string) string {
	keyLower := strings.ToLower(key)
	for _, line := range strings.Split(raw, "\n") {
		if strings.HasPrefix(strings.ToLower(strings.TrimSpace(line)), keyLower) {
			if idx := strings.Index(line, ":"); idx >= 0 {
				return strings.TrimSpace(line[idx+1:])
			}
		}
	}
	return ""
}

func findAllWhoisFields(raw, key string) []string {
	var results []string
	keyLower := strings.ToLower(key)
	for _, line := range strings.Split(raw, "\n") {
		if strings.HasPrefix(strings.ToLower(strings.TrimSpace(line)), keyLower) {
			if idx := strings.Index(line, ":"); idx >= 0 {
				v := strings.TrimSpace(line[idx+1:])
				if v != "" {
					results = append(results, v)
				}
			}
		}
	}
	return results
}

// ParseWhoisText parses raw whois text into a WhoisRecord.
func ParseWhoisText(domain, raw string) WhoisRecord {
	r := WhoisRecord{
		Domain:  strings.ToLower(domain),
		RawText: raw,
	}
	r.Registrar = findWhoisField(raw, "Registrar")
	r.DNSSEC = findWhoisField(raw, "DNSSEC")
	r.CreatedDate = findWhoisField(raw, "Creation Date")
	if r.CreatedDate == "" {
		r.CreatedDate = findWhoisField(raw, "Created Date")
	}
	r.UpdatedDate = findWhoisField(raw, "Updated Date")
	r.ExpiryDate = findWhoisField(raw, "Registry Expiry Date")
	if r.ExpiryDate == "" {
		r.ExpiryDate = findWhoisField(raw, "Expiration Date")
	}
	for _, ns := range findAllWhoisFields(raw, "Name Server") {
		r.NameServers = append(r.NameServers, strings.ToLower(ns))
	}
	r.Status = findAllWhoisFields(raw, "Domain Status")
	if len(r.Status) == 0 {
		r.Status = findAllWhoisFields(raw, "Status")
	}
	reg := WhoisContact{
		Name:         findWhoisField(raw, "Registrant Name"),
		Organization: findWhoisField(raw, "Registrant Organization"),
		Email:        findWhoisField(raw, "Registrant Email"),
		Country:      findWhoisField(raw, "Registrant Country"),
		City:         findWhoisField(raw, "Registrant City"),
		State:        findWhoisField(raw, "Registrant State"),
	}
	if reg.Name != "" || reg.Organization != "" || reg.Email != "" {
		r.Registrant = &reg
	}
	return r
}

// WhoisSearchClient queries and caches whois data.
type WhoisSearchClient struct {
	cache     map[string]WhoisRecord
	simulated map[string]string
}

// NewWhoisSearchClient creates a new client.
func NewWhoisSearchClient() *WhoisSearchClient {
	return &WhoisSearchClient{
		cache:     make(map[string]WhoisRecord),
		simulated: make(map[string]string),
	}
}

// AddSimulated adds a mock whois response.
func (c *WhoisSearchClient) AddSimulated(domain, raw string) {
	c.simulated[strings.ToLower(domain)] = raw
}

// Query fetches whois data for a domain.
func (c *WhoisSearchClient) Query(domain string) WhoisRecord {
	domain = strings.ToLower(domain)
	if r, ok := c.cache[domain]; ok {
		return r
	}
	raw := c.simulated[domain]
	r := ParseWhoisText(domain, raw)
	c.cache[domain] = r
	return r
}

// RegistrarBreakdown counts domains per registrar.
func (c *WhoisSearchClient) RegistrarBreakdown(domains []string) map[string]int {
	counts := make(map[string]int)
	for _, d := range domains {
		r := c.Query(d)
		key := r.Registrar
		if key == "" {
			key = "unknown"
		}
		counts[key]++
	}
	return counts
}

// CompareWhoisRecords returns changed fields between two records.
func CompareWhoisRecords(before, after WhoisRecord) map[string][2]string {
	changes := make(map[string][2]string)
	if before.Registrar != after.Registrar {
		changes["registrar"] = [2]string{before.Registrar, after.Registrar}
	}
	if strings.Join(before.NameServers, ",") != strings.Join(after.NameServers, ",") {
		changes["name_servers"] = [2]string{
			strings.Join(before.NameServers, ","),
			strings.Join(after.NameServers, ","),
		}
	}
	if before.ExpiryDate != after.ExpiryDate {
		changes["expiry_date"] = [2]string{before.ExpiryDate, after.ExpiryDate}
	}
	return changes
}
