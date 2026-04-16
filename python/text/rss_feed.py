"""RSS Feed Creator — generate and parse RSS 2.0 XML feeds."""

from dataclasses import dataclass, field
from datetime import datetime
import xml.etree.ElementTree as ET


@dataclass
class RssItem:
    """A single item in an RSS feed."""
    title: str
    link: str
    description: str
    pub_date: str = ""
    guid: str = ""


@dataclass
class RssFeed:
    """An RSS 2.0 feed with items."""
    title: str
    link: str
    description: str
    language: str = "en-us"
    items: list[RssItem] = field(default_factory=list)

    def add_item(self, title: str, link: str, description: str,
                 pub_date: str = "", guid: str = "") -> None:
        """Add an item to the feed."""
        self.items.append(RssItem(
            title=title, link=link, description=description,
            pub_date=pub_date, guid=guid if guid else link,
        ))

    def to_xml(self) -> str:
        """Render the feed as a valid RSS 2.0 XML string."""
        rss = ET.Element("rss", version="2.0")
        channel = ET.SubElement(rss, "channel")

        ET.SubElement(channel, "title").text = self.title
        ET.SubElement(channel, "link").text = self.link
        ET.SubElement(channel, "description").text = self.description
        ET.SubElement(channel, "language").text = self.language

        for item in self.items:
            item_el = ET.SubElement(channel, "item")
            ET.SubElement(item_el, "title").text = item.title
            ET.SubElement(item_el, "link").text = item.link
            ET.SubElement(item_el, "description").text = item.description
            if item.pub_date:
                ET.SubElement(item_el, "pubDate").text = item.pub_date
            if item.guid:
                ET.SubElement(item_el, "guid").text = item.guid

        ET.indent(rss, space="  ")
        return '<?xml version="1.0" encoding="UTF-8"?>\n' + ET.tostring(
            rss, encoding="unicode"
        )

    @classmethod
    def from_xml(cls, xml_str: str) -> "RssFeed":
        """Parse an RSS 2.0 XML string back into an RssFeed."""
        root = ET.fromstring(xml_str)
        channel = root.find("channel")
        if channel is None:
            raise ValueError("No <channel> element found in RSS XML")

        feed = cls(
            title=channel.findtext("title", ""),
            link=channel.findtext("link", ""),
            description=channel.findtext("description", ""),
            language=channel.findtext("language", "en-us"),
        )

        for item_el in channel.findall("item"):
            feed.items.append(RssItem(
                title=item_el.findtext("title", ""),
                link=item_el.findtext("link", ""),
                description=item_el.findtext("description", ""),
                pub_date=item_el.findtext("pubDate", ""),
                guid=item_el.findtext("guid", ""),
            ))

        return feed


if __name__ == "__main__":
    # --- Demo: create, render, parse, verify roundtrip ---
    feed = RssFeed(
        title="DragonPunk Noir Dispatch",
        link="https://dragonpunk.noir/feed",
        description="Transmissions from the noosphere.",
        language="en-us",
    )
    feed.add_item(
        title="The Forge Ignites",
        link="https://dragonpunk.noir/posts/forge-ignites",
        description="The first spark in the temporal chain.",
        pub_date="Wed, 16 Apr 2026 08:00:00 GMT",
    )
    feed.add_item(
        title="Nine Domains Mapped",
        link="https://dragonpunk.noir/posts/nine-domains",
        description="Schema of the noosphere revealed.",
        pub_date="Thu, 17 Apr 2026 10:00:00 GMT",
    )
    feed.add_item(
        title="Rosetta Stone: RSS",
        link="https://dragonpunk.noir/posts/rosetta-rss",
        description="Structured content meets syndication.",
        pub_date="Fri, 18 Apr 2026 12:00:00 GMT",
    )

    xml_out = feed.to_xml()
    print("=== Generated RSS 2.0 XML ===")
    print(xml_out)
    print()

    # --- Roundtrip ---
    parsed = RssFeed.from_xml(xml_out)
    assert parsed.title == feed.title
    assert parsed.link == feed.link
    assert parsed.description == feed.description
    assert parsed.language == feed.language
    assert len(parsed.items) == len(feed.items)
    for orig, rt in zip(feed.items, parsed.items):
        assert orig.title == rt.title
        assert orig.link == rt.link
        assert orig.description == rt.description
        assert orig.pub_date == rt.pub_date
        assert orig.guid == rt.guid

    print("=== Roundtrip verified: all fields match ===")
