// Web Bot — automated web interaction: crawling, scraping, and task automation.
package networking

import (
	"fmt"
	"regexp"
	"strings"
)

// BotPage is a fetched page.
type BotPage struct {
	URL        string
	StatusCode int
	Body       string
}

// Title extracts the <title> tag content.
func (p BotPage) Title() string {
	re := regexp.MustCompile(`(?i)<title>(.*?)</title>`)
	if m := re.FindStringSubmatch(p.Body); len(m) > 1 {
		return strings.TrimSpace(m[1])
	}
	return ""
}

// ExtractLinks finds all href values.
func (p BotPage) ExtractLinks() []string {
	re := regexp.MustCompile(`href="([^"]+)"`)
	matches := re.FindAllStringSubmatch(p.Body, -1)
	var links []string
	for _, m := range matches {
		if len(m) > 1 {
			links = append(links, m[1])
		}
	}
	return links
}

// ExtractText strips HTML tags.
func (p BotPage) ExtractText() string {
	re := regexp.MustCompile(`<[^>]+>`)
	text := re.ReplaceAllString(p.Body, " ")
	return strings.Join(strings.Fields(text), " ")
}

// BotCrawlResult holds crawl output.
type BotCrawlResult struct {
	Pages     map[string]BotPage
	Errors    map[string]string
	LinkGraph map[string][]string
}

// PageCount returns number of crawled pages.
func (r BotCrawlResult) PageCount() int { return len(r.Pages) }

// Summary returns a one-line crawl summary.
func (r BotCrawlResult) CrawlSummary() string {
	allLinks := make(map[string]bool)
	for _, links := range r.LinkGraph {
		for _, l := range links {
			allLinks[l] = true
		}
	}
	return fmt.Sprintf("Crawl: %d pages, %d errors, %d unique links",
		len(r.Pages), len(r.Errors), len(allLinks))
}

// BotActionResult holds the result of a bot action.
type BotActionResult struct {
	ActionType string
	Target     string
	Value      string
	Result     string
	Success    bool
	Error      string
}

// WebCrawlBot handles automated web interactions.
type WebCrawlBot struct {
	pageStore   map[string]BotPage
	Visited     map[string]bool
	History     []string
	CurrentPage *BotPage
	FormData    map[string]string
}

// NewWebCrawlBot creates a bot.
func NewWebCrawlBot() *WebCrawlBot {
	return &WebCrawlBot{
		pageStore: make(map[string]BotPage),
		Visited:   make(map[string]bool),
		FormData:  make(map[string]string),
	}
}

// AddPage adds a simulated page.
func (b *WebCrawlBot) AddPage(url, body string, status int) {
	b.pageStore[url] = BotPage{URL: url, StatusCode: status, Body: body}
}

// Fetch retrieves a page from the store.
func (b *WebCrawlBot) Fetch(url string) BotPage {
	b.Visited[url] = true
	b.History = append(b.History, url)
	if p, ok := b.pageStore[url]; ok {
		return p
	}
	return BotPage{URL: url, StatusCode: 404, Body: "Not Found"}
}

// Navigate fetches and sets the current page.
func (b *WebCrawlBot) Navigate(url string) BotPage {
	p := b.Fetch(url)
	b.CurrentPage = &p
	return p
}

// Crawl follows links from a start URL.
func (b *WebCrawlBot) Crawl(startURL string, maxPages int) BotCrawlResult {
	result := BotCrawlResult{
		Pages: make(map[string]BotPage), Errors: make(map[string]string),
		LinkGraph: make(map[string][]string),
	}
	queue := []string{startURL}
	domain := ""
	if parts := strings.SplitN(startURL, "/", 4); len(parts) >= 3 {
		domain = parts[2]
	}
	for len(queue) > 0 && len(result.Pages) < maxPages {
		url := queue[0]
		queue = queue[1:]
		if _, ok := result.Pages[url]; ok {
			continue
		}
		if _, ok := result.Errors[url]; ok {
			continue
		}
		page := b.Fetch(url)
		if page.StatusCode != 200 {
			result.Errors[url] = fmt.Sprintf("HTTP %d", page.StatusCode)
			continue
		}
		links := page.ExtractLinks()
		result.LinkGraph[url] = links
		result.Pages[url] = page
		for _, link := range links {
			if _, ok := result.Pages[link]; ok {
				continue
			}
			if domain != "" {
				if parts := strings.SplitN(link, "/", 4); len(parts) >= 3 {
					if parts[2] != domain {
						continue
					}
				}
			}
			queue = append(queue, link)
		}
	}
	return result
}

// ExecuteActions runs a list of bot actions.
func (b *WebCrawlBot) ExecuteActions(actions []BotActionResult) []BotActionResult {
	var results []BotActionResult
	for _, a := range actions {
		r := BotActionResult{ActionType: a.ActionType, Target: a.Target, Value: a.Value, Success: true}
		switch a.ActionType {
		case "navigate":
			p := b.Navigate(a.Target)
			r.Success = p.StatusCode == 200
			r.Result = fmt.Sprintf("HTTP %d", p.StatusCode)
		case "assert":
			if b.CurrentPage == nil {
				r.Success = false
				r.Error = "No page loaded"
			} else if strings.HasPrefix(a.Target, "status:") {
				var expected int
				fmt.Sscanf(a.Target[7:], "%d", &expected)
				r.Success = b.CurrentPage.StatusCode == expected
			} else if strings.HasPrefix(a.Target, "title:") {
				r.Success = b.CurrentPage.Title() == a.Target[6:]
			} else if strings.HasPrefix(a.Target, "contains:") {
				r.Success = strings.Contains(b.CurrentPage.Body, a.Target[9:])
			}
			if !r.Success && r.Error == "" {
				r.Error = "Assertion failed: " + a.Target
			}
		case "fill":
			b.FormData[a.Target] = a.Value
			r.Result = fmt.Sprintf("Set %s=%s", a.Target, a.Value)
		}
		results = append(results, r)
		if !r.Success {
			break
		}
	}
	return results
}
