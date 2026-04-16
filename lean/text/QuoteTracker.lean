/- Quote Tracker — collect, categorize, and retrieve quotes. -/

namespace Text

/-- A single quote with attribution and categorization. -/
structure Quote where
  text   : String
  author : String
  source : Option String := none
  tags   : List String := []
  deriving Repr, BEq

instance : ToString Quote where
  toString q :=
    let base := s!"\"{q.text}\" — {q.author}"
    let withSrc := match q.source with
      | some s => base ++ s!" ({s})"
      | none   => base
    let withTags := if q.tags.isEmpty then withSrc
      else withSrc ++ s!" [{String.intercalate ", " q.tags}]"
    withTags

/-- A curated collection of quotes with search and random access. -/
structure QuoteTracker where
  quotes : Array Quote := #[]
  deriving Repr

namespace QuoteTracker

/-- Create an empty quote tracker. -/
def empty : QuoteTracker := {}

/-- Add a quote to the collection. -/
def addQuote (qt : QuoteTracker) (text author : String)
    (source : Option String := none) (tags : List String := []) : QuoteTracker :=
  { qt with quotes := qt.quotes.push { text, author, source, tags } }

/-- Return the number of quotes. -/
def count (qt : QuoteTracker) : Nat := qt.quotes.size

/-- Search for quotes by author (case-insensitive substring match). -/
def searchByAuthor (qt : QuoteTracker) (author : String) : Array Quote :=
  let needle := author.toLower
  qt.quotes.filter fun q => (q.author.toLower).containsSubstr needle

/-- Search for quotes by tag (case-insensitive). -/
def searchByTag (qt : QuoteTracker) (tag : String) : Array Quote :=
  let needle := tag.toLower
  qt.quotes.filter fun q => q.tags.any fun t => t.toLower == needle

/-- Return sorted list of unique authors. -/
def listAuthors (qt : QuoteTracker) : List String :=
  let authors := qt.quotes.toList.map Quote.author
  let unique := authors.eraseDups
  unique.mergeSort (· < ·)

/-- Return sorted list of unique tags. -/
def listTags (qt : QuoteTracker) : List String :=
  let allTags := qt.quotes.toList.bind Quote.tags
  let unique := allTags.eraseDups
  unique.mergeSort (· < ·)

/-- Return a quote by index (modular). Returns none if empty. -/
def quoteAt (qt : QuoteTracker) (idx : Nat) : Option Quote :=
  if qt.quotes.size == 0 then none
  else some (qt.quotes[idx % qt.quotes.size]!)

end QuoteTracker
end Text
