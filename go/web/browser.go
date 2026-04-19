// Web Browser with Tabs — state model for tabbed navigation.
//
// Each tab has its own history timeline (back/forward stacks). The browser
// owns the tab list, an active-tab pointer, a closed-tab stack (for reopen),
// and browser-scoped bookmarks. Per-tab history and browser-level closed-tab
// recovery are two orthogonal undo-like stacks.
package web

// TabID identifies a browser tab.
type TabID int64

// Tab holds a tab's url and per-tab history.
type Tab struct {
	ID      TabID
	Title   string
	URL     string
	Back    []string
	Forward []string
}

type closedTab struct {
	URL, Title string
	Back, Forward []string
}

// Browser is a multi-tab browsing session.
type Browser struct {
	tabs      []*Tab
	active    TabID
	closed    []closedTab // newest first (prepend on close)
	bookmarks []Bookmark
	nextID    TabID
}

// Bookmark is a browser-scoped (title, url) pair.
type Bookmark struct{ Title, URL string }

// NewBrowser returns a new empty browser.
func NewBrowser() *Browser { return &Browser{nextID: 1} }

// OpenTab opens a new tab with the given URL and activates it.
func (b *Browser) OpenTab(url string) TabID {
	id := b.nextID
	b.nextID++
	b.tabs = append(b.tabs, &Tab{ID: id, Title: url, URL: url})
	b.active = id
	return id
}

// CloseTab closes a tab; pushes it onto the reopen stack.
func (b *Browser) CloseTab(id TabID) {
	pos := -1
	for i, t := range b.tabs {
		if t.ID == id {
			pos = i
			break
		}
	}
	if pos < 0 {
		return
	}
	t := b.tabs[pos]
	b.closed = append([]closedTab{{URL: t.URL, Title: t.Title,
		Back: t.Back, Forward: t.Forward}}, b.closed...)
	b.tabs = append(b.tabs[:pos], b.tabs[pos+1:]...)
	if b.active == id {
		switch {
		case pos < len(b.tabs):
			b.active = b.tabs[pos].ID
		case len(b.tabs) > 0:
			b.active = b.tabs[len(b.tabs)-1].ID
		default:
			b.active = 0
		}
	}
}

// SwitchTo activates the given tab; returns false if unknown.
func (b *Browser) SwitchTo(id TabID) bool {
	for _, t := range b.tabs {
		if t.ID == id {
			b.active = id
			return true
		}
	}
	return false
}

// ActiveTab returns the currently-active tab or nil.
func (b *Browser) ActiveTab() *Tab {
	if b.active == 0 {
		return nil
	}
	for _, t := range b.tabs {
		if t.ID == b.active {
			return t
		}
	}
	return nil
}

// Navigate the active tab to url.
func (b *Browser) Navigate(url string) bool {
	t := b.ActiveTab()
	if t == nil {
		return false
	}
	t.Back = append(t.Back, t.URL)
	t.Forward = nil
	t.URL = url
	t.Title = url
	return true
}

// Back moves the active tab one step back; returns the new URL or "".
func (b *Browser) Back() string {
	t := b.ActiveTab()
	if t == nil || len(t.Back) == 0 {
		return ""
	}
	prev := t.Back[len(t.Back)-1]
	t.Back = t.Back[:len(t.Back)-1]
	t.Forward = append(t.Forward, t.URL)
	t.URL = prev
	return prev
}

// Forward moves the active tab one step forward; returns the new URL or "".
func (b *Browser) Forward() string {
	t := b.ActiveTab()
	if t == nil || len(t.Forward) == 0 {
		return ""
	}
	next := t.Forward[len(t.Forward)-1]
	t.Forward = t.Forward[:len(t.Forward)-1]
	t.Back = append(t.Back, t.URL)
	t.URL = next
	return next
}

// ReopenClosed restores the most recently closed tab.
func (b *Browser) ReopenClosed() TabID {
	if len(b.closed) == 0 {
		return 0
	}
	ct := b.closed[0]
	b.closed = b.closed[1:]
	id := b.nextID
	b.nextID++
	b.tabs = append(b.tabs, &Tab{ID: id, Title: ct.Title, URL: ct.URL,
		Back: ct.Back, Forward: ct.Forward})
	b.active = id
	return id
}

// AddBookmark bookmarks the active tab.
func (b *Browser) AddBookmark() bool {
	t := b.ActiveTab()
	if t == nil {
		return false
	}
	entry := Bookmark{Title: t.Title, URL: t.URL}
	for _, bm := range b.bookmarks {
		if bm == entry {
			return true
		}
	}
	b.bookmarks = append(b.bookmarks, entry)
	return true
}

// Bookmarks returns a snapshot of the browser's bookmarks.
func (b *Browser) Bookmarks() []Bookmark { return append([]Bookmark{}, b.bookmarks...) }

// TabCount returns the number of open tabs.
func (b *Browser) TabCount() int { return len(b.tabs) }

// ClosedCount returns the number of tabs available for reopening.
func (b *Browser) ClosedCount() int { return len(b.closed) }
