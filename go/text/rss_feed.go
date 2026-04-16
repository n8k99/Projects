package text

import (
	"encoding/xml"
	"fmt"
)

// RssItem represents a single item in an RSS feed.
type RssItem struct {
	XMLName     xml.Name `xml:"item"`
	Title       string   `xml:"title"`
	Link        string   `xml:"link"`
	Description string   `xml:"description"`
	PubDate     string   `xml:"pubDate,omitempty"`
	GUID        string   `xml:"guid,omitempty"`
}

// RssChannel holds the channel metadata and items.
type RssChannel struct {
	XMLName     xml.Name  `xml:"channel"`
	Title       string    `xml:"title"`
	Link        string    `xml:"link"`
	Description string    `xml:"description"`
	Language    string    `xml:"language"`
	Items       []RssItem `xml:"item"`
}

// RssFeed is the top-level RSS 2.0 document.
type RssFeed struct {
	XMLName xml.Name   `xml:"rss"`
	Version string     `xml:"version,attr"`
	Channel RssChannel `xml:"channel"`
}

// NewRssFeed creates a new RSS 2.0 feed with channel metadata.
func NewRssFeed(title, link, description, language string) *RssFeed {
	return &RssFeed{
		Version: "2.0",
		Channel: RssChannel{
			Title:       title,
			Link:        link,
			Description: description,
			Language:    language,
		},
	}
}

// AddItem adds an item to the feed's channel.
func (f *RssFeed) AddItem(title, link, description, pubDate, guid string) {
	if guid == "" {
		guid = link
	}
	f.Channel.Items = append(f.Channel.Items, RssItem{
		Title:       title,
		Link:        link,
		Description: description,
		PubDate:     pubDate,
		GUID:        guid,
	})
}

// ToXML renders the feed as a valid RSS 2.0 XML string.
func (f *RssFeed) ToXML() (string, error) {
	data, err := xml.MarshalIndent(f, "", "  ")
	if err != nil {
		return "", fmt.Errorf("marshal RSS feed: %w", err)
	}
	return xml.Header + string(data), nil
}

// ParseRssFeed parses an RSS 2.0 XML string back into an RssFeed.
func ParseRssFeed(xmlStr string) (*RssFeed, error) {
	var feed RssFeed
	if err := xml.Unmarshal([]byte(xmlStr), &feed); err != nil {
		return nil, fmt.Errorf("unmarshal RSS feed: %w", err)
	}
	return &feed, nil
}
