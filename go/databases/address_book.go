// Address Book — contacts with normalised-identifier deduplication and merge.
package databases

import (
	"sort"
	"strings"
)

// ABContact is one address-book record.
type ABContact struct {
	ID        int64
	Name      string
	Emails    []string
	Phones    []string
	Tags      []string
	Address   string
	Notes     string
	CreatedMs int64
}

// CanonicalEmails returns lowercased, gmail-aware forms.
func (c *ABContact) CanonicalEmails() []string {
	out := make([]string, len(c.Emails))
	for i, e := range c.Emails {
		out[i] = CanonicalEmail(e)
	}
	return out
}

// CanonicalPhones returns digit-only, +1-stripped forms.
func (c *ABContact) CanonicalPhones() []string {
	out := make([]string, len(c.Phones))
	for i, p := range c.Phones {
		out[i] = CanonicalPhone(p)
	}
	return out
}

// CanonicalEmail normalises an email for duplicate-detection.
func CanonicalEmail(email string) string {
	lower := strings.ToLower(strings.TrimSpace(email))
	at := strings.Index(lower, "@")
	if at < 0 {
		return lower
	}
	local, domain := lower[:at], lower[at+1:]
	if plus := strings.Index(local, "+"); plus >= 0 {
		local = local[:plus]
	}
	if domain == "gmail.com" || domain == "googlemail.com" {
		local = strings.ReplaceAll(local, ".", "")
		domain = "gmail.com"
	}
	return local + "@" + domain
}

// CanonicalPhone keeps digits only, drops US leading 1.
func CanonicalPhone(phone string) string {
	var b strings.Builder
	for _, c := range phone {
		if c >= '0' && c <= '9' {
			b.WriteRune(c)
		}
	}
	digits := b.String()
	if len(digits) == 11 && digits[0] == '1' {
		return digits[1:]
	}
	return digits
}

// ABook is a map of ID → contact.
type ABook struct {
	Contacts map[int64]*ABContact
	nextID   int64
}

// NewABook creates an empty address book.
func NewABook() *ABook {
	return &ABook{Contacts: map[int64]*ABContact{}, nextID: 1}
}

// AddContact inserts a contact; bumps nextID if needed.
func (a *ABook) AddContact(c *ABContact) int64 {
	if c.ID >= a.nextID {
		a.nextID = c.ID + 1
	}
	a.Contacts[c.ID] = c
	return c.ID
}

// NewID returns a fresh ID.
func (a *ABook) NewID() int64 {
	id := a.nextID
	a.nextID++
	return id
}

// GetContact returns the contact at an ID, or nil.
func (a *ABook) GetContact(id int64) *ABContact { return a.Contacts[id] }

// FindDuplicates returns groups of IDs sharing at least one canonical identifier.
func (a *ABook) FindDuplicates() [][]int64 {
	byEmail := map[string][]int64{}
	byPhone := map[string][]int64{}
	for _, c := range a.Contacts {
		for _, e := range c.CanonicalEmails() {
			if e != "" {
				byEmail[e] = append(byEmail[e], c.ID)
			}
		}
		for _, p := range c.CanonicalPhones() {
			if p != "" {
				byPhone[p] = append(byPhone[p], c.ID)
			}
		}
	}
	parent := map[int64]int64{}
	for id := range a.Contacts {
		parent[id] = id
	}
	var find func(int64) int64
	find = func(x int64) int64 {
		if parent[x] != x {
			parent[x] = find(parent[x])
		}
		return parent[x]
	}
	unite := func(x, y int64) {
		rx, ry := find(x), find(y)
		if rx != ry {
			parent[rx] = ry
		}
	}
	for _, ids := range byEmail {
		for i := 1; i < len(ids); i++ {
			unite(ids[0], ids[i])
		}
	}
	for _, ids := range byPhone {
		for i := 1; i < len(ids); i++ {
			unite(ids[0], ids[i])
		}
	}
	groups := map[int64][]int64{}
	ids := make([]int64, 0, len(a.Contacts))
	for id := range a.Contacts {
		ids = append(ids, id)
	}
	sort.Slice(ids, func(i, j int) bool { return ids[i] < ids[j] })
	for _, id := range ids {
		r := find(id)
		groups[r] = append(groups[r], id)
	}
	var out [][]int64
	for _, g := range groups {
		if len(g) >= 2 {
			out = append(out, g)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i][0] < out[j][0] })
	return out
}

// MergeContacts combines the given IDs into a new contact.
// Returns the new ID or an error.
func (a *ABook) MergeContacts(ids []int64) (int64, error) {
	if len(ids) < 2 {
		return 0, errAB("need at least 2 ids to merge")
	}
	sources := make([]*ABContact, 0, len(ids))
	for _, id := range ids {
		c := a.Contacts[id]
		if c == nil {
			return 0, errAB("id not found")
		}
		sources = append(sources, c)
	}
	merged := &ABContact{ID: a.NewID()}
	merged.CreatedMs = sources[0].CreatedMs
	for _, s := range sources[1:] {
		if s.CreatedMs < merged.CreatedMs {
			merged.CreatedMs = s.CreatedMs
		}
	}
	for _, s := range sources {
		if s.Name != "" && len(s.Name) > len(merged.Name) {
			merged.Name = s.Name
		}
	}
	mergeUnique := func(a []string, src []string) []string {
		seen := map[string]bool{}
		for _, v := range a {
			seen[v] = true
		}
		for _, v := range src {
			if v != "" && !seen[v] {
				seen[v] = true
				a = append(a, v)
			}
		}
		return a
	}
	for _, s := range sources {
		merged.Emails = mergeUnique(merged.Emails, s.Emails)
		merged.Phones = mergeUnique(merged.Phones, s.Phones)
		merged.Tags = mergeUnique(merged.Tags, s.Tags)
	}
	sort.Strings(merged.Emails)
	sort.Strings(merged.Phones)
	sort.Strings(merged.Tags)
	var addrs, notes []string
	for _, s := range sources {
		if s.Address != "" {
			addrs = append(addrs, s.Address)
		}
		if s.Notes != "" {
			notes = append(notes, s.Notes)
		}
	}
	merged.Address = strings.Join(addrs, " | ")
	merged.Notes = strings.Join(notes, "\n---\n")
	for _, id := range ids {
		delete(a.Contacts, id)
	}
	a.Contacts[merged.ID] = merged
	return merged.ID, nil
}

// SearchByName returns IDs whose name contains the needle (case-insensitive).
func (a *ABook) SearchByName(needle string) []int64 {
	lc := strings.ToLower(needle)
	var out []int64
	for _, c := range a.Contacts {
		if strings.Contains(strings.ToLower(c.Name), lc) {
			out = append(out, c.ID)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i] < out[j] })
	return out
}

// SearchByEmail matches contacts via canonical form.
func (a *ABook) SearchByEmail(email string) []int64 {
	can := CanonicalEmail(email)
	var out []int64
	for _, c := range a.Contacts {
		for _, e := range c.CanonicalEmails() {
			if e == can {
				out = append(out, c.ID)
				break
			}
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i] < out[j] })
	return out
}

type abError string

func (e abError) Error() string { return string(e) }
func errAB(s string) error { return abError(s) }
