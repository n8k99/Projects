// Image Gallery — external storage refs, content-addressed dedup, set-algebra tag queries.
package classes

import (
	"errors"
	"fmt"
)

// BlobRef is a pointer to externally-managed bytes.
type BlobRef struct {
	Path      string
	SizeBytes uint64
	Exists    bool
}

// GalleryImage is a metadata envelope around externally-stored bytes.
type GalleryImage struct {
	ID           string
	Filename     string
	Source       BlobRef
	Thumbnail    *BlobRef
	Preview      *BlobRef
	Width        uint32
	Height       uint32
	MimeType     string
	ContentHash  string
	Tags         map[string]struct{}
}

// NewGalleryImage builds an image with an empty tag set.
func NewGalleryImage(id, filename, hash string, source BlobRef) *GalleryImage {
	return &GalleryImage{
		ID: id, Filename: filename, Source: source, ContentHash: hash,
		MimeType: "image/jpeg", Tags: make(map[string]struct{}),
	}
}

// Album is a curated ordered collection of images.
type Album struct {
	ID          string
	Name        string
	Description string
	ImageIDs    []string
	Curator     string
}

// Gallery errors.
var (
	ErrUnknownImage = errors.New("unknown image")
	ErrUnknownAlbum = errors.New("unknown album")
)

// Gallery owns images and albums.
type Gallery struct {
	Images map[string]*GalleryImage
	Albums map[string]*Album
}

// NewGallery builds an empty gallery.
func NewGallery() *Gallery {
	return &Gallery{
		Images: make(map[string]*GalleryImage),
		Albums: make(map[string]*Album),
	}
}

// AddImage registers an image.
func (g *Gallery) AddImage(image *GalleryImage) error {
	if _, ok := g.Images[image.ID]; ok {
		return fmt.Errorf("%w: image %s", ErrDuplicate, image.ID)
	}
	if image.Tags == nil {
		image.Tags = make(map[string]struct{})
	}
	g.Images[image.ID] = image
	return nil
}

// RemoveImage deletes the image and orphans any album references to it.
func (g *Gallery) RemoveImage(imageID string) error {
	if _, ok := g.Images[imageID]; !ok {
		return fmt.Errorf("%w: %s", ErrUnknownImage, imageID)
	}
	delete(g.Images, imageID)
	for _, album := range g.Albums {
		filtered := album.ImageIDs[:0]
		for _, id := range album.ImageIDs {
			if id != imageID {
				filtered = append(filtered, id)
			}
		}
		album.ImageIDs = filtered
	}
	return nil
}

// Tag adds tags to an image.
func (g *Gallery) Tag(imageID string, tags ...string) error {
	image, ok := g.Images[imageID]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownImage, imageID)
	}
	for _, t := range tags {
		image.Tags[t] = struct{}{}
	}
	return nil
}

// Untag removes tags from an image.
func (g *Gallery) Untag(imageID string, tags ...string) error {
	image, ok := g.Images[imageID]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownImage, imageID)
	}
	for _, t := range tags {
		delete(image.Tags, t)
	}
	return nil
}

// AllTags returns every distinct tag across all images.
func (g *Gallery) AllTags() map[string]struct{} {
	out := make(map[string]struct{})
	for _, image := range g.Images {
		for t := range image.Tags {
			out[t] = struct{}{}
		}
	}
	return out
}

// TagCounts returns a histogram of tag usage.
func (g *Gallery) TagCounts() map[string]int {
	out := make(map[string]int)
	for _, image := range g.Images {
		for t := range image.Tags {
			out[t]++
		}
	}
	return out
}

// WithAllTags returns images tagged with every tag in `tags` (intersection).
func (g *Gallery) WithAllTags(tags []string) []*GalleryImage {
	if len(tags) == 0 {
		out := make([]*GalleryImage, 0, len(g.Images))
		for _, i := range g.Images {
			out = append(out, i)
		}
		return out
	}
	var out []*GalleryImage
	for _, image := range g.Images {
		ok := true
		for _, t := range tags {
			if _, has := image.Tags[t]; !has {
				ok = false
				break
			}
		}
		if ok {
			out = append(out, image)
		}
	}
	return out
}

// WithAnyTag returns images having at least one of the given tags (union).
func (g *Gallery) WithAnyTag(tags []string) []*GalleryImage {
	if len(tags) == 0 {
		return nil
	}
	var out []*GalleryImage
	for _, image := range g.Images {
		for _, t := range tags {
			if _, has := image.Tags[t]; has {
				out = append(out, image)
				break
			}
		}
	}
	return out
}

// WithoutTags returns images matching all `include` tags and none of `exclude`.
func (g *Gallery) WithoutTags(include, exclude []string) []*GalleryImage {
	base := g.WithAllTags(include)
	if len(exclude) == 0 {
		return base
	}
	var out []*GalleryImage
	for _, image := range base {
		blocked := false
		for _, t := range exclude {
			if _, has := image.Tags[t]; has {
				blocked = true
				break
			}
		}
		if !blocked {
			out = append(out, image)
		}
	}
	return out
}

// Duplicates groups image ids by ContentHash; only hashes with >1 image appear.
func (g *Gallery) Duplicates() map[string][]string {
	buckets := make(map[string][]string)
	for _, image := range g.Images {
		if image.ContentHash == "" {
			continue
		}
		buckets[image.ContentHash] = append(buckets[image.ContentHash], image.ID)
	}
	for h, ids := range buckets {
		if len(ids) < 2 {
			delete(buckets, h)
		}
	}
	return buckets
}

// FindByHash returns images matching a content hash.
func (g *Gallery) FindByHash(hash string) []*GalleryImage {
	var out []*GalleryImage
	for _, image := range g.Images {
		if image.ContentHash == hash {
			out = append(out, image)
		}
	}
	return out
}

// CreateAlbum registers a new album.
func (g *Gallery) CreateAlbum(album *Album) error {
	if _, ok := g.Albums[album.ID]; ok {
		return fmt.Errorf("%w: album %s", ErrDuplicate, album.ID)
	}
	for _, iid := range album.ImageIDs {
		if _, ok := g.Images[iid]; !ok {
			return fmt.Errorf("%w: %s", ErrUnknownImage, iid)
		}
	}
	g.Albums[album.ID] = album
	return nil
}

// AddToAlbum appends an image to an album (no duplicates).
func (g *Gallery) AddToAlbum(albumID, imageID string) error {
	if _, ok := g.Images[imageID]; !ok {
		return fmt.Errorf("%w: %s", ErrUnknownImage, imageID)
	}
	album, ok := g.Albums[albumID]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownAlbum, albumID)
	}
	for _, id := range album.ImageIDs {
		if id == imageID {
			return nil
		}
	}
	album.ImageIDs = append(album.ImageIDs, imageID)
	return nil
}

// RemoveFromAlbum removes an image reference from an album.
func (g *Gallery) RemoveFromAlbum(albumID, imageID string) error {
	album, ok := g.Albums[albumID]
	if !ok {
		return fmt.Errorf("%w: %s", ErrUnknownAlbum, albumID)
	}
	filtered := album.ImageIDs[:0]
	for _, id := range album.ImageIDs {
		if id != imageID {
			filtered = append(filtered, id)
		}
	}
	album.ImageIDs = filtered
	return nil
}

// AlbumContents returns the album's images in curator order.
func (g *Gallery) AlbumContents(albumID string) ([]*GalleryImage, error) {
	album, ok := g.Albums[albumID]
	if !ok {
		return nil, fmt.Errorf("%w: %s", ErrUnknownAlbum, albumID)
	}
	var out []*GalleryImage
	for _, id := range album.ImageIDs {
		if img, ok := g.Images[id]; ok {
			out = append(out, img)
		}
	}
	return out, nil
}

// AlbumsContaining returns the albums that include an image.
func (g *Gallery) AlbumsContaining(imageID string) []*Album {
	var out []*Album
	for _, album := range g.Albums {
		for _, id := range album.ImageIDs {
			if id == imageID {
				out = append(out, album)
				break
			}
		}
	}
	return out
}

// TotalBytes sums source + thumbnail + preview sizes.
func (g *Gallery) TotalBytes() uint64 {
	var total uint64
	for _, image := range g.Images {
		total += image.Source.SizeBytes
		if image.Thumbnail != nil {
			total += image.Thumbnail.SizeBytes
		}
		if image.Preview != nil {
			total += image.Preview.SizeBytes
		}
	}
	return total
}
