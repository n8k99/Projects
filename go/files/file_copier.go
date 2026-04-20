// File Copy Utility — recursive tree copy with policy + filter + progress events.
package files

import (
	"fmt"
	"sort"
	"strings"
)

// FcItemKind tags a filesystem item.
type FcItemKind int

const (
	FcFile FcItemKind = 0
	FcDir  FcItemKind = 1
)

// FcItem is a file or directory.
type FcItem struct {
	Kind FcItemKind
	Size int64
}

// FcFile factory.
func NewFcFile(size int64) FcItem { return FcItem{Kind: FcFile, Size: size} }

// FcDir factory.
func NewFcDir() FcItem { return FcItem{Kind: FcDir} }

// FcOverwrite is the collision policy.
type FcOverwrite int

const (
	FcSkip              FcOverwrite = 0
	FcOverwriteReplace  FcOverwrite = 1
	FcRenameWithSuffix  FcOverwrite = 2
)

// FcFilter accepts include / rejects exclude by substring match.
type FcFilter struct {
	Include []string
	Exclude []string
}

// FcAllowAll returns a filter that accepts everything.
func FcAllowAll() FcFilter { return FcFilter{} }

// Accepts returns true if the path passes the filter.
func (f FcFilter) Accepts(path string) bool {
	if len(f.Include) > 0 {
		ok := false
		for _, pat := range f.Include {
			if strings.Contains(path, pat) {
				ok = true
				break
			}
		}
		if !ok {
			return false
		}
	}
	for _, pat := range f.Exclude {
		if strings.Contains(path, pat) {
			return false
		}
	}
	return true
}

// FcEventKind tags a copy event.
type FcEventKind int

const (
	FcCopied      FcEventKind = 0
	FcSkipped     FcEventKind = 1
	FcRenamed     FcEventKind = 2
	FcCreatedDir  FcEventKind = 3
)

// FcCopyEvent describes one operation.
type FcCopyEvent struct {
	Kind   FcEventKind
	Source string
	Dest   string
	Bytes  int64
}

// FcFs is a virtual filesystem.
type FcFs struct {
	Items map[string]FcItem
}

// NewFcFs creates an empty fs.
func NewFcFs() *FcFs { return &FcFs{Items: map[string]FcItem{}} }

// FcFromItems builds a fs from a pre-populated list.
func FcFromItems(entries map[string]FcItem) *FcFs {
	fs := NewFcFs()
	for k, v := range entries {
		fs.Items[k] = v
	}
	return fs
}

// Paths returns sorted paths.
func (fs *FcFs) Paths() []string {
	out := make([]string, 0, len(fs.Items))
	for k := range fs.Items {
		out = append(out, k)
	}
	sort.Strings(out)
	return out
}

// CopyTree recursively copies src into dest applying policy + filter.
func (fs *FcFs) CopyTree(srcRoot, destRoot string, policy FcOverwrite, filter FcFilter) ([]FcCopyEvent, error) {
	var srcPaths []string
	for p := range fs.Items {
		if p == srcRoot || strings.HasPrefix(p, srcRoot+"/") {
			srcPaths = append(srcPaths, p)
		}
	}
	if len(srcPaths) == 0 {
		return nil, fmt.Errorf("source not found: %s", srcRoot)
	}
	sort.Strings(srcPaths)

	var events []FcCopyEvent
	for _, srcPath := range srcPaths {
		item := fs.Items[srcPath]
		if !filter.Accepts(srcPath) {
			continue
		}
		var rel string
		if srcPath != srcRoot {
			rel = srcPath[len(srcRoot)+1:]
		}
		destPath := destRoot
		if rel != "" {
			destPath = destRoot + "/" + rel
		}

		finalDest := destPath
		if _, exists := fs.Items[destPath]; exists {
			switch policy {
			case FcSkip:
				events = append(events, FcCopyEvent{
					Kind: FcSkipped, Source: srcPath, Dest: destPath,
				})
				continue
			case FcRenameWithSuffix:
				renamed := fcNextAvailable(fs.Items, destPath)
				events = append(events, FcCopyEvent{
					Kind: FcRenamed, Source: srcPath, Dest: renamed,
				})
				finalDest = renamed
			}
		}

		if item.Kind == FcDir {
			fs.Items[finalDest] = NewFcDir()
			events = append(events, FcCopyEvent{
				Kind: FcCreatedDir, Source: srcPath, Dest: finalDest,
			})
		} else {
			fs.Items[finalDest] = NewFcFile(item.Size)
			events = append(events, FcCopyEvent{
				Kind: FcCopied, Source: srcPath, Dest: finalDest, Bytes: item.Size,
			})
		}
	}
	return events, nil
}

// FcTotalSizeCopied sums bytes across `Copied` events.
func FcTotalSizeCopied(events []FcCopyEvent) int64 {
	var sum int64
	for _, e := range events {
		if e.Kind == FcCopied {
			sum += e.Bytes
		}
	}
	return sum
}

func fcNextAvailable(items map[string]FcItem, base string) string {
	n := 1
	for {
		candidate := fmt.Sprintf("%s.%d", base, n)
		if _, exists := items[candidate]; !exists {
			return candidate
		}
		n++
	}
}
