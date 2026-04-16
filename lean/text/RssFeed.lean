/- RSS Feed Creator — generate and parse RSS 2.0 XML feeds. -/

namespace Text

/-- A single item in an RSS feed. -/
structure RssItem where
  title       : String := ""
  link        : String := ""
  description : String := ""
  pubDate     : String := ""
  guid        : String := ""
  deriving Repr, BEq

/-- An RSS 2.0 feed with items. -/
structure RssFeed where
  title       : String := ""
  link        : String := ""
  description : String := ""
  language    : String := "en-us"
  items       : Array RssItem := #[]
  deriving Repr, BEq

/-- Escape special XML characters. -/
def xmlEscape (s : String) : String :=
  s.replace "&" "&amp;"
  |>.replace "<" "&lt;"
  |>.replace ">" "&gt;"
  |>.replace "\"" "&quot;"
  |>.replace "'" "&apos;"

/-- Unescape XML entities. -/
def xmlUnescape (s : String) : String :=
  s.replace "&lt;" "<"
  |>.replace "&gt;" ">"
  |>.replace "&quot;" "\""
  |>.replace "&apos;" "'"
  |>.replace "&amp;" "&"

/-- Wrap text content in an XML tag. -/
def wrapTag (tag : String) (content : String) : String :=
  s!"<{tag}>{xmlEscape content}</{tag}>"

/-- Extract text between <tag> and </tag>. Returns "" if not found. -/
def extractTag (xml : String) (tag : String) : String :=
  let openTag := s!"<{tag}>"
  let closeTag := s!"</{tag}>"
  match xml.findSubstr? openTag with
  | none => ""
  | some startPos =>
    let contentStart := startPos + openTag.length
    let rest := xml.drop contentStart
    match rest.findSubstr? closeTag with
    | none => ""
    | some endPos => xmlUnescape (rest.take endPos)

/-- Find the position of a substring, returning none if not found. -/
def String.findSubstr? (s : String) (sub : String) : Option Nat := do
  let subLen := sub.length
  if subLen > s.length then none
  else
    let limit := s.length - subLen
    for i in [:limit + 1] do
      if (s.drop i |>.take subLen) == sub then
        return i
    none

/-- Create a new RSS feed. -/
def RssFeed.new (title link description : String)
    (language : String := "en-us") : RssFeed :=
  { title, link, description, language }

/-- Add an item to the feed. -/
def RssFeed.addItem (feed : RssFeed) (title link description : String)
    (pubDate : String := "") (guid : String := "") : RssFeed :=
  let actualGuid := if guid.isEmpty then link else guid
  { feed with items := feed.items.push {
    title, link, description, pubDate, guid := actualGuid
  }}

/-- Render the feed as RSS 2.0 XML. -/
def RssFeed.toXml (feed : RssFeed) : String :=
  let header := "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"
  let channelOpen := "<rss version=\"2.0\">\n  <channel>\n"
  let meta := s!"    {wrapTag "title" feed.title}\n" ++
              s!"    {wrapTag "link" feed.link}\n" ++
              s!"    {wrapTag "description" feed.description}\n" ++
              s!"    {wrapTag "language" feed.language}\n"
  let itemsXml := feed.items.foldl (fun acc item =>
    let pubDateLine := if item.pubDate.isEmpty then ""
                       else s!"      {wrapTag "pubDate" item.pubDate}\n"
    let guidLine := if item.guid.isEmpty then ""
                    else s!"      {wrapTag "guid" item.guid}\n"
    acc ++ "    <item>\n" ++
      s!"      {wrapTag "title" item.title}\n" ++
      s!"      {wrapTag "link" item.link}\n" ++
      s!"      {wrapTag "description" item.description}\n" ++
      pubDateLine ++ guidLine ++
      "    </item>\n"
  ) ""
  let channelClose := "  </channel>\n</rss>"
  header ++ channelOpen ++ meta ++ itemsXml ++ channelClose

/-- Collect all substrings between openTag and closeTag. -/
partial def extractAllBetween (xml : String) (openTag closeTag : String)
    (acc : Array String := #[]) : Array String :=
  match xml.findSubstr? openTag with
  | none => acc
  | some startPos =>
    let contentStart := startPos + openTag.length
    let rest := xml.drop contentStart
    match rest.findSubstr? closeTag with
    | none => acc
    | some endPos =>
      let content := rest.take endPos
      let remaining := rest.drop (endPos + closeTag.length)
      extractAllBetween remaining openTag closeTag (acc.push content)

/-- Parse an RSS 2.0 XML string back into an RssFeed. -/
def RssFeed.fromXml (xmlStr : String) : RssFeed :=
  let channelContent := extractTag xmlStr "channel"
  -- For channel metadata, use content before first <item>
  let meta := match channelContent.findSubstr? "<item>" with
    | none => channelContent
    | some pos => channelContent.take pos
  let feed : RssFeed := {
    title := extractTag meta "title"
    link := extractTag meta "link"
    description := extractTag meta "description"
    language := let lang := extractTag meta "language"
                if lang.isEmpty then "en-us" else lang
  }
  let itemBodies := extractAllBetween channelContent "<item>" "</item>"
  let items := itemBodies.map fun body => {
    title := extractTag body "title"
    link := extractTag body "link"
    description := extractTag body "description"
    pubDate := extractTag body "pubDate"
    guid := extractTag body "guid"
    : RssItem
  }
  { feed with items }

end Text
