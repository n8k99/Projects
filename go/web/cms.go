// Content Management System — lifecycle FSM + revisions + scheduled publish.
package web

import (
	"errors"
	"fmt"
)

// ContentState is the article's FSM state.
type ContentState int

const (
	StateDraft ContentState = iota
	StateInReview
	StateScheduled
	StatePublished
	StateArchived
)

func (s ContentState) String() string {
	switch s {
	case StateDraft:
		return "draft"
	case StateInReview:
		return "in_review"
	case StateScheduled:
		return "scheduled"
	case StatePublished:
		return "published"
	case StateArchived:
		return "archived"
	}
	return "unknown"
}

// CMSArticle is one article with its lifecycle state.
type CMSArticle struct {
	ID                     int64
	Slug                   string
	Title                  string
	Body                   string
	Author                 string
	State                  ContentState
	Tags                   []string
	CreatedAtMs            int64
	ModifiedAtMs           int64
	ScheduledPublishAtMs   int64 // 0 if none
	PublishedAtMs          int64 // 0 if never
}

// CMSRevision is one saved version of an article.
type CMSRevision struct {
	ArticleID  int64
	Version    int
	Title      string
	Body       string
	SavedAtMs  int64
	SavedBy    string
}

// CMS errors.
var (
	ErrCMSUnknownArticle    = errors.New("unknown article")
	ErrCMSUnknownRevision   = errors.New("unknown revision")
	ErrCMSIllegalTransition = errors.New("illegal transition")
	ErrCMSArchivedIsFrozen  = errors.New("archived articles are frozen")
	ErrCMSSlugTaken         = errors.New("slug already in use")
)

// CMS holds articles and revision history.
type CMS struct {
	articles  []*CMSArticle
	revisions map[int64][]CMSRevision
	nextID    int64
}

// NewCMS returns an empty CMS.
func NewCMS() *CMS {
	return &CMS{
		revisions: make(map[int64][]CMSRevision),
		nextID:    1,
	}
}

// CreateArticle creates a new article in Draft state; slug must be unique.
func (c *CMS) CreateArticle(author, slug, title, body string, nowMs int64) (int64, error) {
	for _, a := range c.articles {
		if a.Slug == slug {
			return 0, fmt.Errorf("%w: %s", ErrCMSSlugTaken, slug)
		}
	}
	id := c.nextID
	c.nextID++
	c.articles = append(c.articles, &CMSArticle{
		ID: id, Slug: slug, Title: title, Body: body, Author: author,
		State: StateDraft, CreatedAtMs: nowMs, ModifiedAtMs: nowMs,
	})
	c.revisions[id] = []CMSRevision{{
		ArticleID: id, Version: 1, Title: title, Body: body,
		SavedAtMs: nowMs, SavedBy: author,
	}}
	return id, nil
}

func (c *CMS) require(id int64) (*CMSArticle, error) {
	for _, a := range c.articles {
		if a.ID == id {
			return a, nil
		}
	}
	return nil, fmt.Errorf("%w: %d", ErrCMSUnknownArticle, id)
}

// SaveRevision updates article content and appends a revision.
func (c *CMS) SaveRevision(id int64, user, title, body string, nowMs int64) (int, error) {
	a, err := c.require(id)
	if err != nil {
		return 0, err
	}
	if a.State == StateArchived {
		return 0, ErrCMSArchivedIsFrozen
	}
	a.Title = title
	a.Body = body
	a.ModifiedAtMs = nowMs
	revs := c.revisions[id]
	version := 1
	if len(revs) > 0 {
		version = revs[len(revs)-1].Version + 1
	}
	c.revisions[id] = append(revs, CMSRevision{
		ArticleID: id, Version: version,
		Title: title, Body: body,
		SavedAtMs: nowMs, SavedBy: user,
	})
	return version, nil
}

// SubmitForReview transitions Draft → InReview.
func (c *CMS) SubmitForReview(id int64, nowMs int64) error {
	a, err := c.require(id)
	if err != nil {
		return err
	}
	if a.State != StateDraft {
		return fmt.Errorf("%w: submit_for_review from %s", ErrCMSIllegalTransition, a.State)
	}
	a.State = StateInReview
	a.ModifiedAtMs = nowMs
	return nil
}

// ApproveAndPublish transitions Draft/InReview/Scheduled → Published.
func (c *CMS) ApproveAndPublish(id int64, nowMs int64) error {
	a, err := c.require(id)
	if err != nil {
		return err
	}
	if a.State != StateDraft && a.State != StateInReview && a.State != StateScheduled {
		return fmt.Errorf("%w: approve_and_publish from %s", ErrCMSIllegalTransition, a.State)
	}
	a.State = StatePublished
	a.PublishedAtMs = nowMs
	a.ModifiedAtMs = nowMs
	a.ScheduledPublishAtMs = 0
	return nil
}

// SchedulePublish transitions non-terminal → Scheduled with a future publish time.
func (c *CMS) SchedulePublish(id int64, atMs, nowMs int64) error {
	a, err := c.require(id)
	if err != nil {
		return err
	}
	if a.State != StateDraft && a.State != StateInReview && a.State != StateScheduled {
		return fmt.Errorf("%w: schedule_publish from %s", ErrCMSIllegalTransition, a.State)
	}
	a.State = StateScheduled
	a.ScheduledPublishAtMs = atMs
	a.ModifiedAtMs = nowMs
	return nil
}

// Archive transitions any non-Archived → Archived.
func (c *CMS) Archive(id int64, nowMs int64) error {
	a, err := c.require(id)
	if err != nil {
		return err
	}
	if a.State == StateArchived {
		return fmt.Errorf("%w: archive from archived", ErrCMSIllegalTransition)
	}
	a.State = StateArchived
	a.ModifiedAtMs = nowMs
	return nil
}

// Unarchive transitions Archived → Draft.
func (c *CMS) Unarchive(id int64, nowMs int64) error {
	a, err := c.require(id)
	if err != nil {
		return err
	}
	if a.State != StateArchived {
		return fmt.Errorf("%w: unarchive from %s", ErrCMSIllegalTransition, a.State)
	}
	a.State = StateDraft
	a.ModifiedAtMs = nowMs
	return nil
}

// Tick transitions Scheduled articles whose time has come to Published.
func (c *CMS) Tick(nowMs int64) []int64 {
	var published []int64
	for _, a := range c.articles {
		if a.State == StateScheduled && a.ScheduledPublishAtMs > 0 && a.ScheduledPublishAtMs <= nowMs {
			a.State = StatePublished
			a.PublishedAtMs = nowMs
			a.ModifiedAtMs = nowMs
			a.ScheduledPublishAtMs = 0
			published = append(published, a.ID)
		}
	}
	return published
}

// RevertToRevision restores an old title/body by APPENDING a new revision.
func (c *CMS) RevertToRevision(id int64, version int, user string, nowMs int64) (int, error) {
	a, err := c.require(id)
	if err != nil {
		return 0, err
	}
	if a.State == StateArchived {
		return 0, ErrCMSArchivedIsFrozen
	}
	revs := c.revisions[id]
	var target *CMSRevision
	for i := range revs {
		if revs[i].Version == version {
			target = &revs[i]
			break
		}
	}
	if target == nil {
		return 0, fmt.Errorf("%w: %d v%d", ErrCMSUnknownRevision, id, version)
	}
	a.Title = target.Title
	a.Body = target.Body
	a.ModifiedAtMs = nowMs
	newVersion := revs[len(revs)-1].Version + 1
	c.revisions[id] = append(revs, CMSRevision{
		ArticleID: id, Version: newVersion,
		Title: target.Title, Body: target.Body,
		SavedAtMs: nowMs, SavedBy: user,
	})
	return newVersion, nil
}

// SetTags replaces the article's tag set.
func (c *CMS) SetTags(id int64, tags []string, nowMs int64) error {
	a, err := c.require(id)
	if err != nil {
		return err
	}
	a.Tags = append([]string{}, tags...)
	a.ModifiedAtMs = nowMs
	return nil
}

// Lookups.
func (c *CMS) Article(id int64) *CMSArticle {
	for _, a := range c.articles {
		if a.ID == id {
			return a
		}
	}
	return nil
}

func (c *CMS) ArticleBySlug(slug string) *CMSArticle {
	for _, a := range c.articles {
		if a.Slug == slug {
			return a
		}
	}
	return nil
}

func (c *CMS) Articles() []*CMSArticle { return c.articles }

func (c *CMS) ByState(state ContentState) []*CMSArticle {
	var out []*CMSArticle
	for _, a := range c.articles {
		if a.State == state {
			out = append(out, a)
		}
	}
	return out
}

func (c *CMS) ByTag(tag string) []*CMSArticle {
	var out []*CMSArticle
	for _, a := range c.articles {
		for _, t := range a.Tags {
			if t == tag {
				out = append(out, a)
				break
			}
		}
	}
	return out
}

func (c *CMS) ByAuthor(author string) []*CMSArticle {
	var out []*CMSArticle
	for _, a := range c.articles {
		if a.Author == author {
			out = append(out, a)
		}
	}
	return out
}

func (c *CMS) RevisionsOf(id int64) []CMSRevision {
	return c.revisions[id]
}
