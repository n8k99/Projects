-- Image Gallery — external storage refs, content-addressed dedup, set-algebra tag queries.

namespace Classes

structure BlobRef where
  path : String
  sizeBytes : Nat := 0
  exists : Bool := true
  deriving Repr

structure GalleryImage where
  id : String
  filename : String
  source : BlobRef
  thumbnail : Option BlobRef := none
  preview : Option BlobRef := none
  width : Nat := 0
  height : Nat := 0
  mimeType : String := "image/jpeg"
  contentHash : String := ""
  tags : List String := []       -- maintained as a set (no duplicates)
  deriving Repr

structure Album where
  id : String
  name : String
  description : String := ""
  imageIds : List String := []
  curator : String := ""
  deriving Repr

structure Gallery where
  images : List GalleryImage := []
  albums : List Album := []
  deriving Repr

def Gallery.empty : Gallery := { }

def Gallery.findImage (g : Gallery) (id : String) : Option GalleryImage :=
  g.images.find? (·.id == id)

def Gallery.findAlbum (g : Gallery) (id : String) : Option Album :=
  g.albums.find? (·.id == id)

def Gallery.addImage (g : Gallery) (image : GalleryImage) : Except String Gallery :=
  if g.images.any (·.id == image.id)
  then .error s!"duplicate image: {image.id}"
  else .ok { g with images := g.images ++ [image] }

def Gallery.removeImage (g : Gallery) (imageId : String) : Except String Gallery :=
  if (g.findImage imageId).isNone then
    .error s!"unknown image: {imageId}"
  else
    let filteredAlbums := g.albums.map fun a =>
      { a with imageIds := a.imageIds.filter (· ≠ imageId) }
    .ok { g with
      images := g.images.filter (·.id ≠ imageId),
      albums := filteredAlbums
    }

private def addTag (img : GalleryImage) (t : String) : GalleryImage :=
  if img.tags.contains t then img else { img with tags := img.tags ++ [t] }

private def removeTag (img : GalleryImage) (t : String) : GalleryImage :=
  { img with tags := img.tags.filter (· ≠ t) }

def Gallery.tagImage (g : Gallery) (imageId : String) (tags : List String)
    : Except String Gallery :=
  if (g.findImage imageId).isNone then
    .error s!"unknown image: {imageId}"
  else
    .ok { g with images := g.images.map fun img =>
      if img.id == imageId then tags.foldl addTag img else img }

def Gallery.untagImage (g : Gallery) (imageId : String) (tags : List String)
    : Except String Gallery :=
  if (g.findImage imageId).isNone then
    .error s!"unknown image: {imageId}"
  else
    .ok { g with images := g.images.map fun img =>
      if img.id == imageId then tags.foldl removeTag img else img }

def Gallery.allTags (g : Gallery) : List String :=
  (g.images.flatMap (·.tags)).eraseDups

/-- Intersection: images whose tag set is a superset of `tags`. -/
def Gallery.withAllTags (g : Gallery) (tags : List String) : List GalleryImage :=
  if tags.isEmpty then g.images
  else g.images.filter fun img => tags.all fun t => img.tags.contains t

/-- Union: images with at least one of the given tags. -/
def Gallery.withAnyTag (g : Gallery) (tags : List String) : List GalleryImage :=
  if tags.isEmpty then []
  else g.images.filter fun img => tags.any fun t => img.tags.contains t

/-- Difference: intersection of `include` minus any image touching `exclude`. -/
def Gallery.withoutTags (g : Gallery) (include : List String) (exclude : List String)
    : List GalleryImage :=
  (g.withAllTags include).filter fun img =>
    ¬ exclude.any fun t => img.tags.contains t

/-- Group images by contentHash. Only hashes with more than one image appear. -/
def Gallery.duplicates (g : Gallery) : List (String × List String) :=
  let hashes := g.images.map (·.contentHash) |>.filter (· ≠ "") |>.eraseDups
  hashes.filterMap fun h =>
    let ids := (g.images.filter (·.contentHash == h)).map (·.id)
    if ids.length > 1 then some (h, ids) else none

def Gallery.findByHash (g : Gallery) (h : String) : List GalleryImage :=
  g.images.filter (·.contentHash == h)

def Gallery.createAlbum (g : Gallery) (album : Album) : Except String Gallery :=
  if g.albums.any (·.id == album.id) then
    .error s!"duplicate album: {album.id}"
  else if album.imageIds.any fun iid => (g.findImage iid).isNone then
    .error "unknown image in album"
  else
    .ok { g with albums := g.albums ++ [album] }

def Gallery.addToAlbum (g : Gallery) (albumId imageId : String) : Except String Gallery :=
  if (g.findImage imageId).isNone then
    .error s!"unknown image: {imageId}"
  else if (g.findAlbum albumId).isNone then
    .error s!"unknown album: {albumId}"
  else
    .ok { g with albums := g.albums.map fun a =>
      if a.id == albumId && ¬ a.imageIds.contains imageId
        then { a with imageIds := a.imageIds ++ [imageId] }
        else a }

def Gallery.removeFromAlbum (g : Gallery) (albumId imageId : String) : Except String Gallery :=
  if (g.findAlbum albumId).isNone then
    .error s!"unknown album: {albumId}"
  else
    .ok { g with albums := g.albums.map fun a =>
      if a.id == albumId then { a with imageIds := a.imageIds.filter (· ≠ imageId) } else a }

def Gallery.albumContents (g : Gallery) (albumId : String)
    : Except String (List GalleryImage) :=
  match g.findAlbum albumId with
  | none => .error s!"unknown album: {albumId}"
  | some a => .ok <| a.imageIds.filterMap g.findImage

def Gallery.albumsContaining (g : Gallery) (imageId : String) : List Album :=
  g.albums.filter (·.imageIds.contains imageId)

def Gallery.totalBytes (g : Gallery) : Nat :=
  g.images.foldl (fun acc img =>
    acc + img.source.sizeBytes
      + img.thumbnail.map (·.sizeBytes) |>.getD 0
      + img.preview.map (·.sizeBytes) |>.getD 0) 0

end Classes
