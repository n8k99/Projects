"""Web Bot — automated web interaction: crawling, scraping, form submission, and task automation."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional, Callable
import re


@dataclass
class Page:
    """A fetched web page."""
    url: str
    status_code: int = 200
    content_type: str = "text/html"
    body: str = ""
    headers: dict[str, str] = field(default_factory=dict)
    fetched_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    @property
    def title(self) -> str:
        m = re.search(r"<title>(.*?)</title>", self.body, re.IGNORECASE | re.DOTALL)
        return m.group(1).strip() if m else ""

    def extract_links(self) -> list[str]:
        return re.findall(r'href=["\']([^"\']+)["\']', self.body, re.IGNORECASE)

    def extract_text(self) -> str:
        text = re.sub(r"<[^>]+>", " ", self.body)
        text = re.sub(r"\s+", " ", text).strip()
        return text

    def extract_by_pattern(self, pattern: str) -> list[str]:
        return re.findall(pattern, self.body)


@dataclass
class CrawlResult:
    """Result of crawling a set of pages."""
    pages: dict[str, Page] = field(default_factory=dict)
    errors: dict[str, str] = field(default_factory=dict)
    link_graph: dict[str, list[str]] = field(default_factory=dict)

    @property
    def page_count(self) -> int:
        return len(self.pages)

    @property
    def error_count(self) -> int:
        return len(self.errors)

    def all_links(self) -> set[str]:
        links: set[str] = set()
        for targets in self.link_graph.values():
            links.update(targets)
        return links

    def summary(self) -> str:
        return (f"Crawl: {self.page_count} pages, {self.error_count} errors, "
                f"{len(self.all_links())} unique links")


@dataclass
class BotAction:
    """A single action in a bot script."""
    action_type: str  # "navigate", "extract", "click", "fill", "wait", "assert"
    target: str = ""
    value: str = ""
    result: Optional[str] = None
    success: bool = True
    error: str = ""


@dataclass
class BotScript:
    """A sequence of automated actions."""
    name: str
    actions: list[BotAction] = field(default_factory=list)
    variables: dict[str, str] = field(default_factory=dict)

    def add_navigate(self, url: str) -> "BotScript":
        self.actions.append(BotAction("navigate", target=url))
        return self

    def add_extract(self, pattern: str, var_name: str = "") -> "BotScript":
        self.actions.append(BotAction("extract", target=pattern, value=var_name))
        return self

    def add_assert(self, condition: str) -> "BotScript":
        self.actions.append(BotAction("assert", target=condition))
        return self

    def add_fill(self, field_name: str, value: str) -> "BotScript":
        self.actions.append(BotAction("fill", target=field_name, value=value))
        return self

    def add_click(self, selector: str) -> "BotScript":
        self.actions.append(BotAction("click", target=selector))
        return self


class WebBot:
    """Automated web interaction engine.

    Operates on a simulated web (page_store) for in-process testing,
    or can be extended with real HTTP for live crawling.
    """

    def __init__(self) -> None:
        self.page_store: dict[str, Page] = {}
        self.visited: set[str] = set()
        self.history: list[str] = []
        self.current_page: Optional[Page] = None
        self.extraction_results: dict[str, list[str]] = {}
        self.form_data: dict[str, str] = {}

    def add_page(self, url: str, body: str, status: int = 200,
                 content_type: str = "text/html") -> None:
        """Add a simulated page to the bot's web."""
        self.page_store[url] = Page(url=url, body=body, status_code=status,
                                     content_type=content_type)

    def fetch(self, url: str) -> Page:
        """Fetch a page (from store or simulate 404)."""
        self.visited.add(url)
        self.history.append(url)
        page = self.page_store.get(url)
        if page is None:
            return Page(url=url, status_code=404, body="Not Found")
        return page

    def navigate(self, url: str) -> Page:
        """Navigate to a URL, updating current page."""
        self.current_page = self.fetch(url)
        return self.current_page

    def crawl(self, start_url: str, max_pages: int = 100,
              same_domain_only: bool = True) -> CrawlResult:
        """Crawl from a start URL, following links."""
        result = CrawlResult()
        queue = [start_url]
        domain = start_url.split("/")[2] if "/" in start_url else ""

        while queue and len(result.pages) < max_pages:
            url = queue.pop(0)
            if url in result.pages or url in result.errors:
                continue
            page = self.fetch(url)
            if page.status_code != 200:
                result.errors[url] = f"HTTP {page.status_code}"
                continue
            result.pages[url] = page
            links = page.extract_links()
            result.link_graph[url] = links
            for link in links:
                if link in result.pages or link in result.errors:
                    continue
                if same_domain_only and domain:
                    link_domain = link.split("/")[2] if link.count("/") >= 2 else ""
                    if link_domain != domain:
                        continue
                queue.append(link)

        return result

    def execute_script(self, script: BotScript) -> list[BotAction]:
        """Execute a bot script, returning the actions with results."""
        executed: list[BotAction] = []
        for action in script.actions:
            result = self._execute_action(action, script.variables)
            executed.append(result)
            if not result.success:
                break
        return executed

    def _execute_action(self, action: BotAction, variables: dict[str, str]) -> BotAction:
        result = BotAction(action.action_type, action.target, action.value)
        if action.action_type == "navigate":
            page = self.navigate(action.target)
            result.success = page.status_code == 200
            result.result = f"HTTP {page.status_code}"
            if not result.success:
                result.error = f"Navigation failed: {page.status_code}"

        elif action.action_type == "extract":
            if self.current_page is None:
                result.success = False
                result.error = "No page loaded"
            else:
                matches = self.current_page.extract_by_pattern(action.target)
                result.result = str(matches)
                if action.value:
                    variables[action.value] = ", ".join(matches)
                    self.extraction_results[action.value] = matches

        elif action.action_type == "assert":
            if self.current_page is None:
                result.success = False
                result.error = "No page loaded"
            else:
                if action.target.startswith("status:"):
                    expected = int(action.target.split(":")[1])
                    result.success = self.current_page.status_code == expected
                elif action.target.startswith("contains:"):
                    expected = action.target.split(":", 1)[1]
                    result.success = expected in self.current_page.body
                elif action.target.startswith("title:"):
                    expected = action.target.split(":", 1)[1]
                    result.success = self.current_page.title == expected
                if not result.success:
                    result.error = f"Assertion failed: {action.target}"

        elif action.action_type == "fill":
            self.form_data[action.target] = action.value
            result.result = f"Set {action.target}={action.value}"

        elif action.action_type == "click":
            result.result = f"Clicked {action.target}"

        return result

    def stats(self) -> dict:
        return {
            "pages_visited": len(self.visited),
            "history_length": len(self.history),
            "extractions": len(self.extraction_results),
            "form_fields": len(self.form_data),
        }


# --- demo ---
if __name__ == "__main__":
    bot = WebBot()

    # Simulate a small website
    bot.add_page("https://em.com/", '<html><title>EM</title><a href="https://em.com/about">About</a></html>')
    bot.add_page("https://em.com/about", '<html><title>About EM</title><p>Eckenrode Muziekopname</p></html>')
    bot.add_page("https://em.com/music", '<html><title>Music</title></html>')

    # Crawl
    result = bot.crawl("https://em.com/", max_pages=10)
    print(result.summary())

    # Script
    script = (BotScript("check-em")
              .add_navigate("https://em.com/")
              .add_assert("status:200")
              .add_assert("title:EM")
              .add_extract(r"<a href=\"([^\"]+)\">", "links"))

    actions = bot.execute_script(script)
    for a in actions:
        status = "OK" if a.success else f"FAIL: {a.error}"
        print(f"  {a.action_type} {a.target} -> {status}")
