//! RSS Feed Creator — generate and parse RSS 2.0 XML feeds.

/// A single item in an RSS feed.
#[derive(Debug, Clone, PartialEq)]
pub struct RssItem {
    pub title: String,
    pub link: String,
    pub description: String,
    pub pub_date: String,
    pub guid: String,
}

/// An RSS 2.0 feed with items.
#[derive(Debug, Clone, PartialEq)]
pub struct RssFeed {
    pub title: String,
    pub link: String,
    pub description: String,
    pub language: String,
    pub items: Vec<RssItem>,
}

impl RssFeed {
    /// Create a new feed with the given channel metadata.
    pub fn new(title: &str, link: &str, description: &str, language: &str) -> Self {
        Self {
            title: title.to_string(),
            link: link.to_string(),
            description: description.to_string(),
            language: language.to_string(),
            items: Vec::new(),
        }
    }

    /// Add an item to the feed.
    pub fn add_item(
        &mut self,
        title: &str,
        link: &str,
        description: &str,
        pub_date: &str,
        guid: &str,
    ) {
        let guid = if guid.is_empty() { link } else { guid };
        self.items.push(RssItem {
            title: title.to_string(),
            link: link.to_string(),
            description: description.to_string(),
            pub_date: pub_date.to_string(),
            guid: guid.to_string(),
        });
    }

    /// Render the feed as a valid RSS 2.0 XML string.
    pub fn to_xml(&self) -> String {
        let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str("<rss version=\"2.0\">\n");
        out.push_str("  <channel>\n");
        out.push_str(&format!("    <title>{}</title>\n", xml_escape(&self.title)));
        out.push_str(&format!("    <link>{}</link>\n", xml_escape(&self.link)));
        out.push_str(&format!(
            "    <description>{}</description>\n",
            xml_escape(&self.description)
        ));
        out.push_str(&format!(
            "    <language>{}</language>\n",
            xml_escape(&self.language)
        ));

        for item in &self.items {
            out.push_str("    <item>\n");
            out.push_str(&format!(
                "      <title>{}</title>\n",
                xml_escape(&item.title)
            ));
            out.push_str(&format!(
                "      <link>{}</link>\n",
                xml_escape(&item.link)
            ));
            out.push_str(&format!(
                "      <description>{}</description>\n",
                xml_escape(&item.description)
            ));
            if !item.pub_date.is_empty() {
                out.push_str(&format!(
                    "      <pubDate>{}</pubDate>\n",
                    xml_escape(&item.pub_date)
                ));
            }
            if !item.guid.is_empty() {
                out.push_str(&format!(
                    "      <guid>{}</guid>\n",
                    xml_escape(&item.guid)
                ));
            }
            out.push_str("    </item>\n");
        }

        out.push_str("  </channel>\n");
        out.push_str("</rss>");
        out
    }

    /// Parse an RSS 2.0 XML string back into an RssFeed.
    pub fn from_xml(xml: &str) -> Result<Self, String> {
        let channel_start = xml
            .find("<channel>")
            .ok_or("No <channel> found")?;
        let channel_end = xml
            .find("</channel>")
            .ok_or("No </channel> found")?;
        let channel = &xml[channel_start + 9..channel_end];

        // Extract channel-level fields (before any <item>)
        let items_start = channel.find("<item>");
        let channel_meta = if let Some(pos) = items_start {
            &channel[..pos]
        } else {
            channel
        };

        let title = extract_tag(channel_meta, "title").unwrap_or_default();
        let link = extract_tag(channel_meta, "link").unwrap_or_default();
        let description = extract_tag(channel_meta, "description").unwrap_or_default();
        let language = extract_tag(channel_meta, "language").unwrap_or_else(|| "en-us".to_string());

        let mut items = Vec::new();
        let mut search = channel;
        while let Some(start) = search.find("<item>") {
            let end = search[start..]
                .find("</item>")
                .ok_or("Unclosed <item>")?;
            let item_xml = &search[start + 6..start + end];
            items.push(RssItem {
                title: extract_tag(item_xml, "title").unwrap_or_default(),
                link: extract_tag(item_xml, "link").unwrap_or_default(),
                description: extract_tag(item_xml, "description").unwrap_or_default(),
                pub_date: extract_tag(item_xml, "pubDate").unwrap_or_default(),
                guid: extract_tag(item_xml, "guid").unwrap_or_default(),
            });
            search = &search[start + end + 7..];
        }

        Ok(Self {
            title,
            link,
            description,
            language,
            items,
        })
    }
}

/// Escape special XML characters.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Unescape XML entities back to characters.
fn xml_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Extract text content between <tag> and </tag>.
fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml_unescape(&xml[start..end]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_render() {
        let mut feed = RssFeed::new(
            "Test Feed",
            "https://example.com",
            "A test feed",
            "en-us",
        );
        feed.add_item(
            "First Post",
            "https://example.com/1",
            "Hello world",
            "Wed, 16 Apr 2026 08:00:00 GMT",
            "",
        );
        let xml = feed.to_xml();
        assert!(xml.contains("<title>Test Feed</title>"));
        assert!(xml.contains("<title>First Post</title>"));
        assert!(xml.contains("version=\"2.0\""));
    }

    #[test]
    fn test_roundtrip() {
        let mut feed = RssFeed::new(
            "DragonPunk Dispatch",
            "https://dragonpunk.noir/feed",
            "Transmissions from the noosphere.",
            "en-us",
        );
        feed.add_item(
            "The Forge Ignites",
            "https://dragonpunk.noir/posts/forge",
            "First spark.",
            "Wed, 16 Apr 2026 08:00:00 GMT",
            "forge-001",
        );
        feed.add_item(
            "Nine Domains",
            "https://dragonpunk.noir/posts/nine",
            "Schema revealed.",
            "Thu, 17 Apr 2026 10:00:00 GMT",
            "nine-001",
        );

        let xml = feed.to_xml();
        let parsed = RssFeed::from_xml(&xml).expect("parse failed");

        assert_eq!(feed.title, parsed.title);
        assert_eq!(feed.link, parsed.link);
        assert_eq!(feed.description, parsed.description);
        assert_eq!(feed.language, parsed.language);
        assert_eq!(feed.items.len(), parsed.items.len());
        for (orig, rt) in feed.items.iter().zip(parsed.items.iter()) {
            assert_eq!(orig.title, rt.title);
            assert_eq!(orig.link, rt.link);
            assert_eq!(orig.description, rt.description);
            assert_eq!(orig.pub_date, rt.pub_date);
            assert_eq!(orig.guid, rt.guid);
        }
    }

    #[test]
    fn test_xml_escape_roundtrip() {
        let mut feed = RssFeed::new("A & B", "https://x.com", "1 < 2 > 0", "en");
        feed.add_item("\"Quoted\"", "https://x.com/q", "It's here", "", "q1");

        let xml = feed.to_xml();
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&lt;"));

        let parsed = RssFeed::from_xml(&xml).unwrap();
        assert_eq!(parsed.title, "A & B");
        assert_eq!(parsed.items[0].title, "\"Quoted\"");
        assert_eq!(parsed.items[0].description, "It's here");
    }

    #[test]
    fn test_empty_feed() {
        let feed = RssFeed::new("Empty", "https://x.com", "Nothing here", "en");
        let xml = feed.to_xml();
        let parsed = RssFeed::from_xml(&xml).unwrap();
        assert_eq!(parsed.items.len(), 0);
        assert_eq!(parsed.title, "Empty");
    }
}
