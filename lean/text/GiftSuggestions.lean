/- Random Gift Suggestions — attribute-based gift recommendation. -/

namespace Text

/-- A gift with category, price range, and compatible traits. -/
structure Gift where
  name        : String
  category    : String
  priceRange  : String   -- "budget", "mid", "premium"
  suitableFor : List String := []
  deriving Repr, BEq

instance : ToString Gift where
  toString g :=
    let traits := String.intercalate ", " g.suitableFor
    s!"{g.name} [{g.category}] ({g.priceRange}) — suits: {traits}"

/-- Attribute-based gift recommendation engine. -/
structure GiftSuggester where
  gifts : Array Gift := #[]
  deriving Repr

namespace GiftSuggester

/-- The default catalog of 24 gifts across categories. -/
private def defaultGifts : Array Gift := #[
  -- Tech
  { name := "Mechanical Keyboard", category := "tech", priceRange := "mid", suitableFor := ["techie", "creative"] },
  { name := "Raspberry Pi Kit", category := "tech", priceRange := "budget", suitableFor := ["techie", "creative"] },
  { name := "Noise-Cancelling Headphones", category := "tech", priceRange := "premium", suitableFor := ["techie", "music-lover"] },
  { name := "Smart Home Starter Kit", category := "tech", priceRange := "mid", suitableFor := ["techie"] },
  -- Books
  { name := "Leather-Bound Journal", category := "books", priceRange := "mid", suitableFor := ["bookworm", "creative"] },
  { name := "Complete Tolkien Collection", category := "books", priceRange := "premium", suitableFor := ["bookworm", "adventurer"] },
  { name := "Pocket Poetry Anthology", category := "books", priceRange := "budget", suitableFor := ["bookworm", "creative"] },
  { name := "Cookbook: World Cuisines", category := "books", priceRange := "mid", suitableFor := ["bookworm", "foodie"] },
  -- Outdoor
  { name := "Hammock", category := "outdoor", priceRange := "budget", suitableFor := ["adventurer"] },
  { name := "Hiking Backpack", category := "outdoor", priceRange := "mid", suitableFor := ["adventurer"] },
  { name := "Camping Cookset", category := "outdoor", priceRange := "mid", suitableFor := ["adventurer", "foodie"] },
  { name := "Trail Running Shoes", category := "outdoor", priceRange := "premium", suitableFor := ["adventurer"] },
  -- Cooking
  { name := "Cast Iron Skillet", category := "cooking", priceRange := "budget", suitableFor := ["foodie"] },
  { name := "Spice Collection Box", category := "cooking", priceRange := "mid", suitableFor := ["foodie", "adventurer"] },
  { name := "Chef's Knife Set", category := "cooking", priceRange := "premium", suitableFor := ["foodie"] },
  { name := "Pasta Maker", category := "cooking", priceRange := "mid", suitableFor := ["foodie", "creative"] },
  -- Music
  { name := "Vinyl Record Starter Pack", category := "music", priceRange := "budget", suitableFor := ["music-lover"] },
  { name := "Concert Tickets", category := "music", priceRange := "mid", suitableFor := ["music-lover", "adventurer"] },
  { name := "MIDI Controller", category := "music", priceRange := "mid", suitableFor := ["music-lover", "techie", "creative"] },
  { name := "Turntable", category := "music", priceRange := "premium", suitableFor := ["music-lover"] },
  -- Art
  { name := "Watercolor Set", category := "art", priceRange := "budget", suitableFor := ["creative"] },
  { name := "Drawing Tablet", category := "art", priceRange := "mid", suitableFor := ["creative", "techie"] },
  { name := "Museum Membership", category := "art", priceRange := "mid", suitableFor := ["creative", "bookworm"] },
  { name := "Oil Paint Master Set", category := "art", priceRange := "premium", suitableFor := ["creative"] }
]

/-- Create a new suggester pre-loaded with default gifts. -/
def new : GiftSuggester := { gifts := defaultGifts }

/-- Create an empty suggester. -/
def empty : GiftSuggester := {}

/-- Add a gift to the catalog. -/
def addGift (gs : GiftSuggester) (name category priceRange : String)
    (suitableFor : List String := []) : GiftSuggester :=
  { gs with gifts := gs.gifts.push { name, category, priceRange, suitableFor } }

/-- Count the number of gifts. -/
def count (gs : GiftSuggester) : Nat := gs.gifts.size

/-- Compute relevance score: number of matching traits. -/
private def relevanceScore (traits : List String) (gift : Gift) : Nat :=
  let traitSet := traits.map String.toLower
  gift.suitableFor.foldl (fun acc s =>
    if traitSet.elem s.toLower then acc + 1 else acc) 0

/-- Suggest gifts matching traits, sorted by relevance (descending).
    If budget is some, only gifts in that price range are included. -/
def suggest (gs : GiftSuggester) (traits : List String)
    (budget : Option String := none) : Array Gift :=
  let filtered := match budget with
    | some b => gs.gifts.filter fun g => g.priceRange == b
    | none   => gs.gifts
  let scored := filtered.filterMap fun g =>
    let score := relevanceScore traits g
    if score > 0 then some (score, g) else none
  let sorted := scored.qsort fun a b => a.1 > b.1
  sorted.map Prod.snd

/-- Return all gifts in a given category. -/
def suggestByCategory (gs : GiftSuggester) (category : String) : Array Gift :=
  let cat := category.toLower
  gs.gifts.filter fun g => g.category.toLower == cat

end GiftSuggester
end Text
