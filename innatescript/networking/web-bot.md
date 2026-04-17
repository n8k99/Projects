# G047 — Web Bot

> Automated web interaction: crawling, scraping, form submission, and task automation.

```yaml
id: G047
title: Web Bot
category: networking
requires: [G030-text-to-html, G032-regex-query, G038-port-scanner, G046-web-server]
provides: [web-crawling, content-extraction, link-graph, scripted-automation]
```

## Insight: The Agent That Navigates

The web server (G046) sat still and waited for requests. The web bot moves — it navigates, follows links, reads pages, extracts content, fills forms, and makes assertions. It's the first **autonomous actor** in the Networking category: an entity that decides where to go next based on what it finds.

This is the ghost. Every executive ghost in the noosphere is a web bot: it navigates the vault, reads documents, extracts relevant data, fills in daily note sections, and asserts conditions via `where` clauses. Lena's `{nightly_summary}` is a crawl: start at today's daily note, follow links to project pages, extract accomplishments, compose a summary. The bot is the abstract shape of agent behavior in the noosphere.

## Insight: Crawling Is BFS Over a Link Graph

The crawler starts at a URL, extracts links, adds them to a queue, and visits each one. This is breadth-first search over the web's link graph — G015's Dijkstra without weights, G038's port scan applied to URLs. The link graph IS a graph in the formal sense: pages are nodes, links are directed edges. The crawler explores it systematically.

The vault's wiki-link structure is the same graph. `[[Rosetta Stone]]` links to `[[InnateScript]]`, which links to `[[Innate Spec]]`. A vault crawler would follow `[[wiki-links]]` the way the web bot follows `href` attributes. Same algorithm, different link syntax.

## Insight: Scripted Automation Is a Choreography

The `BotScript` is a sequence of actions: navigate, assert, extract, fill, click. Each action depends on the previous — you can't assert the title until you've navigated. The script IS a choreography: ordered steps with preconditions, where failure at any step halts the dance.

The `assert` action is the `<-` gate. The `extract` action stores variables for later steps. The `fill` action prepares state for a form submission. The `navigate` action is `@reference` resolution. A bot script expressed in InnateScript would be indistinguishable from a choreography.

## Insight: Same-Domain Filtering Is Scope Restriction

`same_domain_only: true` keeps the crawler within one domain — don't follow links to external sites. This is namespace scoping: the crawler stays within its authorized scope. In the noosphere, an agent assigned to The Forge shouldn't crawl into The Markets unless explicitly authorized. The domain filter is the agent's role boundary.

## Insight: The Link Graph Is the Noosphere's Structure

`CrawlResult.link_graph` maps each page to its outgoing links. This is the vault's structure made explicit: which notes reference which other notes. Running the crawler over the vault would produce the same graph that the `em-org-wallpaper` renders on Nathan's desktop. The web bot builds the map that the graph visualizer displays.

## Choreographic Case: Automated Content Verification

```innate
(@verify-publications){
  @bot <- @web-bot{}
  @script <- @bot-script{name: "verify-em-site"}
    .navigate{"https://eckenrodemuziekopname.com"}
    .assert{"status:200"}
    .assert{"contains:Eckenrode Muziekopname"}
    .extract{"<title>(.*?)</title>", var: "title"}
    .navigate{"https://wiki.eckenrodemuziekopname.com"}
    .assert{"status:200"}
  @results <- @bot/execute{script: @script}
  where {
    all_passed: @results.all{|a| a.success}
    title_correct: @variables.title == "Eckenrode Muziekopname"
  }
}
```

The bot script verifies that the live site matches expectations. The `where` scores the verification. If any assertion fails, the choreography reports the failure. This is automated acceptance testing expressed as a choreography.

## Structures

```innate
(defstruct bot-page
  url         : String
  status-code : Nat
  body        : String)

(defstruct crawl-result
  pages      : {String -> BotPage}
  errors     : {String -> String}
  link-graph : {String -> [String]})

(defstruct bot-action
  action-type : "navigate" | "extract" | "assert" | "fill" | "click"
  target      : String
  value       : String
  result      : String?
  success     : Bool
  error       : String)

(defstruct bot-script
  name    : String
  actions : [BotAction])
```

## Resolver Natives

```innate
@bot{}                                                -> WebBot
@bot/navigate{url: String}                            -> BotPage
@bot/crawl{start: String, max-pages: Nat}             -> CrawlResult
@bot/execute{script: BotScript}                        -> [BotAction]
@page/title                                            -> String
@page/links                                            -> [String]
@page/text                                             -> String
@page/extract{pattern: String}                         -> [String]
```

## Demo

```innate
(@demo){
  @bot <- @web-bot{}
  @result <- @bot/crawl{start: "https://em.com/", max-pages: 50}
  @print{@result.summary}
  ;; => Crawl: 12 pages, 2 errors, 34 unique links
  
  @script <- @bot-script{name: "check-em"}
    .navigate{"https://em.com/"}
    .assert{"title:EM"}
    .extract{"href=\"([^\"]+)\"", var: "links"}
  @actions <- @bot/execute{script: @script}
}
```
