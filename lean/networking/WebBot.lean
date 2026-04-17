-- Web Bot — automated web interaction: crawling, scraping, and task automation.

namespace Networking

structure WebBotPage where
  url : String
  statusCode : Nat := 200
  body : String := ""
  deriving Repr

-- Simple title extraction
def WebBotPage.title (p : WebBotPage) : String :=
  let body := p.body
  match body.findSubstr? "<title>", body.findSubstr? "</title>" with
  | some s, some e => if s + 7 < e then (body.extract ⟨s + 7⟩ ⟨e⟩).trim else ""
  | _, _ => ""
where
  String.findSubstr? (s needle : String) : Option Nat :=
    let sLen := s.length; let nLen := needle.length
    if nLen > sLen then none
    else
      let mut i := 0
      for _ in [:sLen - nLen + 1] do
        if s.extract ⟨i⟩ ⟨i + nLen⟩ == needle then return i
        i := i + 1
      none

-- Simple link extraction (finds href="..." patterns)
def WebBotPage.extractLinks (p : WebBotPage) : List String :=
  let body := p.body
  let mut links : List String := []
  let mut pos := 0
  -- Simple scan for href="
  for _ in [:body.length] do
    if pos >= body.length then break
    let rest := body.drop pos
    match rest.findSubstr? "href=\"" with
    | none => break
    | some idx =>
      let start := pos + idx + 6
      let afterStart := body.drop start
      match afterStart.findIdx (· == '"') with
      | endIdx =>
        if endIdx > 0 then
          links := links ++ [body.extract ⟨start⟩ ⟨start + endIdx⟩]
        pos := start + 1
  links
where
  String.findSubstr? (s needle : String) : Option Nat :=
    let sLen := s.length; let nLen := needle.length
    if nLen > sLen then none
    else
      let mut i := 0
      for _ in [:sLen - nLen + 1] do
        if s.extract ⟨i⟩ ⟨i + nLen⟩ == needle then return i
        i := i + 1
      none

structure WebBotCrawlResult where
  pages : List (String × WebBotPage) := []
  errors : List (String × String) := []
  linkGraph : List (String × List String) := []
  deriving Repr

def WebBotCrawlResult.pageCount (r : WebBotCrawlResult) : Nat := r.pages.length
def WebBotCrawlResult.summary (r : WebBotCrawlResult) : String :=
  let allLinks := r.linkGraph.foldl (init := []) fun acc (_, links) => acc ++ links |>.eraseDups
  s!"Crawl: {r.pages.length} pages, {r.errors.length} errors, {allLinks.length} unique links"

structure WebBotState where
  pageStore : List (String × WebBotPage) := []
  visited : List String := []
  history : List String := []
  currentPage : Option WebBotPage := none
  formData : List (String × String) := []
  deriving Repr

namespace WebBotState

def addPage (b : WebBotState) (url body : String) (status : Nat := 200) : WebBotState :=
  { b with pageStore := b.pageStore ++ [(url, { url, statusCode := status, body })] }

def fetch (b : WebBotState) (url : String) : WebBotState × WebBotPage :=
  let b := { b with visited := if b.visited.contains url then b.visited else b.visited ++ [url],
                     history := b.history ++ [url] }
  match b.pageStore.find? (·.1 == url) with
  | some (_, page) => (b, page)
  | none => (b, { url, statusCode := 404, body := "Not Found" })

def navigate (b : WebBotState) (url : String) : WebBotState × WebBotPage :=
  let (b, page) := b.fetch url
  ({ b with currentPage := some page }, page)

end WebBotState

def webBotDemo : IO Unit := do
  let b := WebBotState.addPage {} "https://em.com/" "<html><title>EM</title></html>"
  let (b, page) := b.navigate "https://em.com/"
  IO.println s!"Title: {page.title}"
  IO.println s!"Status: {page.statusCode}"
  IO.println s!"Visited: {b.visited.length}"

end Networking
