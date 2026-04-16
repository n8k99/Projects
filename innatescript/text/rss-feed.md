---
id: G022
title: RSS Feed Creator
domain: text
type: rosetta-stone
status: active
depends_on: [G021]
concepts:
  - structured content
  - serialization
  - syndication
  - publication channel
  - document schema
  - roundtrip encoding
  - broadcast pattern
---

# G022 — RSS Feed Creator

Generate and parse RSS 2.0 XML feeds programmatically.

## Insight: From String to Document

RSS is **structured content** — the first time the Rosetta Stone moves from raw strings to a defined format with schema, nesting, and semantics. An RSS feed isn't text you manipulate character by character — it's a document with structure. This is the jump from "string" to "document."

The text editor (G021) treats content as lines of characters. The RSS feed treats content as **typed fields in a hierarchy**: feed > channel > items, each with title, link, description, pubDate. Structure isn't just formatting — it's meaning.

## Insight: A Publication Channel

An RSS feed is a **broadcast mechanism**. One producer, many consumers. This maps directly to InnateScript's agent model: an agent publishes to a channel, subscribers consume. The dpn-reader on Nathan's desktop IS an RSS consumer. The Rosetta Stone is building the infrastructure that the vault already uses.

The feed doesn't know its readers. The readers don't coordinate with each other. The feed is a **one-to-many contract**: "here is structured content in a format we all agree on." This is the simplest distributed protocol in the Rosetta Stone.

## Insight: Serialization is Communication

`to_xml` / `from_xml` is **serialization** — the first roundtrip format in the Rosetta Stone. Content goes to wire format and back. Serialization is how agents communicate across boundaries: they serialize state into a transport format, send it, and the receiver deserializes.

InnateScript's resolver needs serialization primitives because agents may run in different processes, on different machines. The RSS feed is the training ground: can you take a structured object, flatten it to a string, transmit it, and reconstruct the original? If yes, agents can talk to each other.

## Insight: RSS Items are Vault Notes

The RSS item structure (title, link, description, pubDate) maps to vault notes. A daily note IS an RSS item: it has a title (the date), a link (the file path), content (the day's work), and a publication date. The vault's temporal chain could be exposed as an RSS feed. The Rosetta Stone is accidentally building the vault's syndication layer.

## The Shape

```innate
(define-shape rss-feed
  "An RSS 2.0 feed: structured content as a publication channel."

  ;; --- Data ---
  (state
    (title       : string)
    (link        : string)
    (description : string)
    (language    : string "en-us")
    (items       : (vector-of rss-item)))

  (define-record rss-item
    (title       : string)
    (link        : string)
    (description : string)
    (pub-date    : string "")
    (guid        : string ""))

  ;; --- Construction ---
  (define (add-item feed title link description &key pub-date guid)
    "Append an item to the feed's channel."
    (let ((actual-guid (if (empty? guid) link guid)))
      (-> feed
          (push-item (make-rss-item
                       :title title
                       :link link
                       :description description
                       :pub-date (or pub-date "")
                       :guid actual-guid)))))

  ;; --- Serialization ---
  (define (to-xml feed)
    "Serialize the feed to an RSS 2.0 XML string."
    (xml-document "1.0" "UTF-8"
      (xml-element "rss" '(:version "2.0")
        (xml-element "channel"
          (xml-element "title" (xml-escape (title feed)))
          (xml-element "link" (xml-escape (link feed)))
          (xml-element "description" (xml-escape (description feed)))
          (xml-element "language" (xml-escape (language feed)))
          (map (items feed)
               (lambda (item)
                 (xml-element "item"
                   (xml-element "title" (xml-escape (rss-item-title item)))
                   (xml-element "link" (xml-escape (rss-item-link item)))
                   (xml-element "description" (xml-escape (rss-item-description item)))
                   (when (not (empty? (rss-item-pub-date item)))
                     (xml-element "pubDate" (xml-escape (rss-item-pub-date item))))
                   (when (not (empty? (rss-item-guid item)))
                     (xml-element "guid" (xml-escape (rss-item-guid item)))))))))))

  (define (from-xml xml-str)
    "Deserialize an RSS 2.0 XML string back into an rss-feed."
    (let* ((doc     (parse-xml xml-str))
           (channel (xml-child doc "rss" "channel"))
           (feed    (make-rss-feed
                      :title       (xml-text channel "title")
                      :link        (xml-text channel "link")
                      :description (xml-text channel "description")
                      :language    (or (xml-text channel "language") "en-us"))))
      (dolist (item-node (xml-children channel "item"))
        (add-item feed
                  (xml-text item-node "title")
                  (xml-text item-node "link")
                  (xml-text item-node "description")
                  :pub-date (xml-text item-node "pubDate")
                  :guid     (xml-text item-node "guid")))
      feed)))
```

## Choreographic Extension: Syndication Network

The RSS feed becomes a coordination primitive when agents publish and subscribe:

```innate
(define-choreography syndication-network
  "Agents publishing and consuming structured feeds."

  (roles publisher subscriber-a subscriber-b aggregator)

  ;; The publisher broadcasts — it doesn't know who listens
  (phase :publish
    (publisher -> feed (add-item "New dispatch" url content))
    (publisher -> channel (broadcast (to-xml feed))))

  ;; Subscribers independently consume
  (phase :consume
    (parallel
      (subscriber-a -> local-feed (from-xml (receive channel)))
      (subscriber-b -> local-feed (from-xml (receive channel)))))

  ;; The aggregator merges multiple feeds
  (phase :aggregate
    (aggregator -> merged-feed
      (merge-feeds
        (from-xml (fetch publisher-a-url))
        (from-xml (fetch publisher-b-url))))
    ;; The aggregator is ALSO a publisher
    (aggregator -> meta-channel (broadcast (to-xml merged-feed)))))
```

## What This Means for InnateScript

The RSS feed reveals that InnateScript needs:

1. **Structured data as a first-class concept** — not just strings and numbers, but records with typed fields, nesting, and schemas. The `define-record` form. Content has structure, and the language must express it.

2. **Serialization primitives** — `to-xml` and `from-xml` are the prototype for a general serialization facility. Agents need to serialize any shape to any wire format (XML, JSON, S-expressions) and reconstruct it on the other side. The resolver protocol depends on this.

3. **The broadcast pattern** — one producer, many consumers, no coordination. This is simpler than choreography (which requires role coordination) but essential for the noosphere. Feeds, logs, event streams — all broadcasts. InnateScript needs a `channel` primitive alongside choreography.

4. **Schema as contract** — the RSS 2.0 spec is a contract between producer and consumer. They never meet; they agree on structure. InnateScript shapes ARE contracts. The shape definition IS the schema. `define-shape` is `define-protocol`.

5. **Roundtrip integrity** — if you serialize and deserialize, you must get back what you started with. This is a fundamental property for agent communication: messages must survive the wire. The Rosetta Stone's roundtrip test is a **correctness proof for communication**.

The RSS feed is where the Rosetta Stone shifts from computation to communication. Previous entries computed results. This one publishes them. The feed is the simplest model of an agent addressing the world: "Here is what I have. Take it or leave it."
