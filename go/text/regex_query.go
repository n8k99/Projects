package text

import (
	"regexp"
	"strings"
)

// Pre-built patterns for common structured data.
var (
	PatternEmail     = regexp.MustCompile(`[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}`)
	PatternURL       = regexp.MustCompile(`https?://[^\s<>"']+`)
	PatternPhone     = regexp.MustCompile(`\+?1?[-.\s]?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}`)
	PatternDate      = regexp.MustCompile(`\d{4}-\d{2}-\d{2}`)
	PatternIPAddress = regexp.MustCompile(`\b(?:\d{1,3}\.){3}\d{1,3}\b`)
)

// RegexMatch holds a matched substring and its byte-offset span.
type RegexMatch struct {
	Text  string
	Start int
	End   int
}

// FindMatches returns all non-overlapping matches of pattern in text.
func FindMatches(text, pattern string) []string {
	re := regexp.MustCompile(pattern)
	return re.FindAllString(text, -1)
}

// FindWithPositions returns matches with their byte-offset positions.
func FindWithPositions(text, pattern string) []RegexMatch {
	re := regexp.MustCompile(pattern)
	locs := re.FindAllStringIndex(text, -1)
	matches := make([]RegexMatch, 0, len(locs))
	for _, loc := range locs {
		matches = append(matches, RegexMatch{
			Text:  text[loc[0]:loc[1]],
			Start: loc[0],
			End:   loc[1],
		})
	}
	return matches
}

// RegexReplace replaces all occurrences of pattern with replacement.
func RegexReplace(text, pattern, replacement string) string {
	re := regexp.MustCompile(pattern)
	return re.ReplaceAllString(text, replacement)
}

// RegexSplit splits text on every occurrence of pattern.
func RegexSplit(text, pattern string) []string {
	re := regexp.MustCompile(pattern)
	return re.Split(text, -1)
}

// IsMatch returns true if pattern matches anywhere in text.
func IsMatch(text, pattern string) bool {
	re := regexp.MustCompile(pattern)
	return re.MatchString(text)
}

// ExtractGroups returns capture-group slices for every match of pattern.
// Each inner slice contains the captured groups (excluding the full match).
func ExtractGroups(text, pattern string) [][]string {
	re := regexp.MustCompile(pattern)
	all := re.FindAllStringSubmatch(text, -1)
	groups := make([][]string, 0, len(all))
	for _, match := range all {
		if len(match) > 1 {
			groups = append(groups, match[1:])
		}
	}
	return groups
}

// FindEmails extracts email addresses from text using the pre-built pattern.
func FindEmails(text string) []string {
	return PatternEmail.FindAllString(text, -1)
}

// FindURLs extracts URLs from text using the pre-built pattern.
func FindURLs(text string) []string {
	return PatternURL.FindAllString(text, -1)
}

// FindDates extracts YYYY-MM-DD dates from text using the pre-built pattern.
func FindDates(text string) []string {
	return PatternDate.FindAllString(text, -1)
}

// FindPhones extracts phone numbers from text using the pre-built pattern.
func FindPhones(text string) []string {
	return PatternPhone.FindAllString(text, -1)
}

// FindIPAddresses extracts IPv4 addresses from text using the pre-built pattern.
func FindIPAddresses(text string) []string {
	return PatternIPAddress.FindAllString(text, -1)
}

// IsEmail checks whether s matches the email pattern.
func IsEmail(s string) bool {
	return PatternEmail.MatchString(s)
}

// IsIPAddress checks whether s matches the IPv4 pattern (simple heuristic).
func IsIPAddress(s string) bool {
	if !PatternIPAddress.MatchString(s) {
		return false
	}
	parts := strings.Split(s, ".")
	if len(parts) != 4 {
		return false
	}
	for _, p := range parts {
		n := 0
		for _, c := range p {
			if c < '0' || c > '9' {
				return false
			}
			n = n*10 + int(c-'0')
		}
		if n > 255 {
			return false
		}
	}
	return true
}
