"""Image Gallery — external storage refs, content-addressed dedup, set-algebra tag queries.

The image bytes live OUTSIDE the entity. An `Image` record is a metadata
envelope: filename, dimensions, mime type, a hash, and a pointer to where the
actual bytes are. The database does not own the bytes — it describes them.

Tags form a many-to-many relation. Queries over tags are set operations:

  with_all_tags({"sunset", "beach"})    -> images tagged with BOTH
  with_any_tag({"sunset", "beach"})     -> images tagged with EITHER
  without_tags({"travel"}, {"private"}) -> travel-tagged, not private-tagged

Albums are *curated* collections (intentional, ordered). Tags are *intrinsic*
classifications (properties of the image itself). The distinction matters:
intrinsic properties can be auto-inferred; curated memberships require intent.
"""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional


@dataclass
class BlobRef:
    """A pointer to externally-managed bytes. May go stale if the source moves."""
    path: str                   # filesystem path, URL, s3://..., etc.
    size_bytes: int = 0
    exists: bool = True         # last-known availability; caller re-checks on fetch


@dataclass
class Image:
    """Metadata envelope around externally-stored image bytes."""
    id: str
    filename: str
    source: BlobRef
    thumbnail: Optional[BlobRef] = None
    preview: Optional[BlobRef] = None
    width: int = 0
    height: int = 0
    mime_type: str = "image/jpeg"
    content_hash: str = ""      # identity-by-content; dedup key
    taken_at: Optional[datetime] = None
    tags: set[str] = field(default_factory=set)


@dataclass
class Album:
    """A curated, ordered collection of images."""
    id: str
    name: str
    description: str = ""
    image_ids: list[str] = field(default_factory=list)
    curator: str = ""


class GalleryError(Exception):
    pass


class UnknownEntity(GalleryError): ...
class StaleReference(GalleryError): ...


class Gallery:
    """Images + albums + tags. Tag queries are set algebra; album queries are ordered retrieval."""

    def __init__(self) -> None:
        self.images: dict[str, Image] = {}
        self.albums: dict[str, Album] = {}

    # --- images ---
    def add_image(self, image: Image) -> None:
        if image.id in self.images:
            raise ValueError(f"image already registered: {image.id}")
        self.images[image.id] = image

    def remove_image(self, image_id: str) -> Image:
        if image_id not in self.images:
            raise UnknownEntity(f"unknown image: {image_id}")
        image = self.images.pop(image_id)
        # Orphan references in all albums.
        for album in self.albums.values():
            album.image_ids = [i for i in album.image_ids if i != image_id]
        return image

    def _require(self, image_id: str) -> Image:
        if image_id not in self.images:
            raise UnknownEntity(f"unknown image: {image_id}")
        return self.images[image_id]

    # --- tags ---
    def tag(self, image_id: str, *tags: str) -> Image:
        image = self._require(image_id)
        image.tags.update(tags)
        return image

    def untag(self, image_id: str, *tags: str) -> Image:
        image = self._require(image_id)
        image.tags.difference_update(tags)
        return image

    def all_tags(self) -> set[str]:
        result: set[str] = set()
        for image in self.images.values():
            result.update(image.tags)
        return result

    def tag_counts(self) -> dict[str, int]:
        counts: dict[str, int] = {}
        for image in self.images.values():
            for tag in image.tags:
                counts[tag] = counts.get(tag, 0) + 1
        return counts

    # --- set-algebra tag queries ---
    def with_all_tags(self, tags: set[str]) -> list[Image]:
        """Intersection: images whose tag set is a superset of `tags`."""
        if not tags:
            return list(self.images.values())
        return [i for i in self.images.values() if tags.issubset(i.tags)]

    def with_any_tag(self, tags: set[str]) -> list[Image]:
        """Union: images with at least one of the given tags."""
        if not tags:
            return []
        return [i for i in self.images.values() if i.tags & tags]

    def without_tags(self, include: set[str], exclude: set[str]) -> list[Image]:
        """Difference: match `include` (intersection semantics) and NOT match any of `exclude`."""
        base = self.with_all_tags(include) if include else list(self.images.values())
        return [i for i in base if not (i.tags & exclude)]

    # --- content-addressed identity ---
    def duplicates(self) -> dict[str, list[str]]:
        """Group image ids by content_hash. Hashes with >1 image are duplicates."""
        buckets: dict[str, list[str]] = {}
        for image in self.images.values():
            if not image.content_hash:
                continue
            buckets.setdefault(image.content_hash, []).append(image.id)
        return {h: ids for h, ids in buckets.items() if len(ids) > 1}

    def find_by_hash(self, content_hash: str) -> list[Image]:
        return [i for i in self.images.values() if i.content_hash == content_hash]

    # --- albums ---
    def create_album(self, album: Album) -> None:
        if album.id in self.albums:
            raise ValueError(f"album already registered: {album.id}")
        for iid in album.image_ids:
            if iid not in self.images:
                raise UnknownEntity(f"unknown image in album: {iid}")
        self.albums[album.id] = album

    def add_to_album(self, album_id: str, image_id: str,
                     position: Optional[int] = None) -> Album:
        if album_id not in self.albums:
            raise UnknownEntity(f"unknown album: {album_id}")
        self._require(image_id)
        album = self.albums[album_id]
        if image_id in album.image_ids:
            return album
        if position is None:
            album.image_ids.append(image_id)
        else:
            album.image_ids.insert(max(0, position), image_id)
        return album

    def remove_from_album(self, album_id: str, image_id: str) -> Album:
        if album_id not in self.albums:
            raise UnknownEntity(f"unknown album: {album_id}")
        album = self.albums[album_id]
        album.image_ids = [i for i in album.image_ids if i != image_id]
        return album

    def album_contents(self, album_id: str) -> list[Image]:
        if album_id not in self.albums:
            raise UnknownEntity(f"unknown album: {album_id}")
        return [self.images[iid] for iid in self.albums[album_id].image_ids
                if iid in self.images]

    def albums_containing(self, image_id: str) -> list[Album]:
        self._require(image_id)
        return [a for a in self.albums.values() if image_id in a.image_ids]

    # --- stats ---
    def total_bytes(self) -> int:
        total = 0
        for image in self.images.values():
            total += image.source.size_bytes
            if image.thumbnail:
                total += image.thumbnail.size_bytes
            if image.preview:
                total += image.preview.size_bytes
        return total

    def count_by_mime(self) -> dict[str, int]:
        counts: dict[str, int] = {}
        for image in self.images.values():
            counts[image.mime_type] = counts.get(image.mime_type, 0) + 1
        return counts


# --- demo ---
if __name__ == "__main__":
    g = Gallery()

    beach1 = Image("IMG-001", "beach-sunset.jpg",
                   source=BlobRef("/photos/2026/beach-sunset.jpg", 4_200_000),
                   thumbnail=BlobRef("/thumbs/IMG-001.jpg", 38_000),
                   width=4032, height=3024, content_hash="a1b2c3",
                   tags={"sunset", "beach", "travel"})
    beach2 = Image("IMG-002", "beach-dup.jpg",
                   source=BlobRef("/photos/2026/beach-dup.jpg", 4_200_000),
                   content_hash="a1b2c3",   # same content as beach1
                   tags={"beach"})
    forest = Image("IMG-003", "forest-path.jpg",
                   source=BlobRef("/photos/2026/forest-path.jpg", 3_800_000),
                   content_hash="d4e5f6",
                   tags={"forest", "travel"})
    private_note = Image("IMG-004", "receipt.jpg",
                         source=BlobRef("/photos/private/receipt.jpg", 500_000),
                         content_hash="deadbe",
                         tags={"travel", "private"})

    for img in (beach1, beach2, forest, private_note):
        g.add_image(img)

    # Set-algebra queries
    print("travel AND beach:",
          [i.id for i in g.with_all_tags({"travel", "beach"})])
    print("travel OR forest:",
          [i.id for i in g.with_any_tag({"travel", "forest"})])
    print("travel BUT NOT private:",
          [i.id for i in g.without_tags({"travel"}, {"private"})])

    # Content-addressed dedup
    print("duplicates by hash:", g.duplicates())

    # Curated album (ordered, intentional — separate from tags)
    g.create_album(Album("ALB-TRIP", "Summer Trip",
                         image_ids=["IMG-003", "IMG-001"]))
    print("Summer Trip album (curator order):",
          [i.id for i in g.album_contents("ALB-TRIP")])

    # Tag counts as a histogram
    print("tag counts:", g.tag_counts())
    print("total bytes:", g.total_bytes())
