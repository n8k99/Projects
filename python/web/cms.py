"""Content Management System — lifecycle FSM + revisions + scheduled publish.

Articles progress through a state machine: Draft → InReview → Published →
Archived. Scheduled publish puts an article in Scheduled until the time
arrives, then tick(now) transitions it to Published. Every save creates a
new revision; revert_to_revision adds a NEW revision with old content
(history is append-only).
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum


class ContentState(Enum):
    DRAFT = "draft"
    IN_REVIEW = "in_review"
    SCHEDULED = "scheduled"
    PUBLISHED = "published"
    ARCHIVED = "archived"


@dataclass
class Article:
    id: int
    slug: str
    title: str
    body: str
    author: str
    state: ContentState = ContentState.DRAFT
    tags: set[str] = field(default_factory=set)
    created_at_ms: int = 0
    modified_at_ms: int = 0
    scheduled_publish_at_ms: int | None = None
    published_at_ms: int | None = None


@dataclass(frozen=True)
class Revision:
    article_id: int
    version: int
    title: str
    body: str
    saved_at_ms: int
    saved_by: str


class CmsError(Exception):
    pass


class UnknownArticle(CmsError):
    def __init__(self, aid: int) -> None:
        super().__init__(f"unknown article: {aid}")
        self.article_id = aid


class UnknownRevision(CmsError):
    def __init__(self, aid: int, version: int) -> None:
        super().__init__(f"unknown revision: {aid} v{version}")
        self.article_id = aid
        self.version = version


class IllegalTransition(CmsError):
    def __init__(self, from_state: ContentState, action: str) -> None:
        super().__init__(f"illegal {action} from {from_state.value}")
        self.from_state = from_state
        self.action = action


class ArchivedIsFrozen(CmsError):
    def __init__(self) -> None:
        super().__init__("archived articles are frozen")


class SlugTaken(CmsError):
    def __init__(self, slug: str) -> None:
        super().__init__(f"slug taken: {slug}")
        self.slug = slug


class Cms:
    def __init__(self) -> None:
        self._articles: list[Article] = []
        self._revisions: dict[int, list[Revision]] = {}
        self._next_id = 1

    def create_article(self, author: str, slug: str, title: str, body: str,
                       now_ms: int) -> int:
        if any(a.slug == slug for a in self._articles):
            raise SlugTaken(slug)
        aid = self._next_id
        self._next_id += 1
        self._articles.append(Article(
            id=aid, slug=slug, title=title, body=body, author=author,
            state=ContentState.DRAFT,
            created_at_ms=now_ms, modified_at_ms=now_ms,
        ))
        self._revisions[aid] = [Revision(
            article_id=aid, version=1,
            title=title, body=body,
            saved_at_ms=now_ms, saved_by=author,
        )]
        return aid

    def _require(self, aid: int) -> Article:
        a = next((a for a in self._articles if a.id == aid), None)
        if a is None:
            raise UnknownArticle(aid)
        return a

    def save_revision(self, aid: int, user: str, title: str, body: str,
                      now_ms: int) -> int:
        a = self._require(aid)
        if a.state is ContentState.ARCHIVED:
            raise ArchivedIsFrozen()
        a.title = title
        a.body = body
        a.modified_at_ms = now_ms
        revs = self._revisions[aid]
        version = (revs[-1].version if revs else 0) + 1
        revs.append(Revision(
            article_id=aid, version=version,
            title=title, body=body,
            saved_at_ms=now_ms, saved_by=user,
        ))
        return version

    def submit_for_review(self, aid: int, now_ms: int) -> None:
        a = self._require(aid)
        if a.state is not ContentState.DRAFT:
            raise IllegalTransition(a.state, "submit_for_review")
        a.state = ContentState.IN_REVIEW
        a.modified_at_ms = now_ms

    def approve_and_publish(self, aid: int, now_ms: int) -> None:
        a = self._require(aid)
        if a.state not in (ContentState.DRAFT, ContentState.IN_REVIEW,
                           ContentState.SCHEDULED):
            raise IllegalTransition(a.state, "approve_and_publish")
        a.state = ContentState.PUBLISHED
        a.published_at_ms = now_ms
        a.modified_at_ms = now_ms
        a.scheduled_publish_at_ms = None

    def schedule_publish(self, aid: int, at_ms: int, now_ms: int) -> None:
        a = self._require(aid)
        if a.state not in (ContentState.DRAFT, ContentState.IN_REVIEW,
                           ContentState.SCHEDULED):
            raise IllegalTransition(a.state, "schedule_publish")
        a.state = ContentState.SCHEDULED
        a.scheduled_publish_at_ms = at_ms
        a.modified_at_ms = now_ms

    def archive(self, aid: int, now_ms: int) -> None:
        a = self._require(aid)
        if a.state is ContentState.ARCHIVED:
            raise IllegalTransition(a.state, "archive")
        a.state = ContentState.ARCHIVED
        a.modified_at_ms = now_ms

    def unarchive(self, aid: int, now_ms: int) -> None:
        a = self._require(aid)
        if a.state is not ContentState.ARCHIVED:
            raise IllegalTransition(a.state, "unarchive")
        a.state = ContentState.DRAFT
        a.modified_at_ms = now_ms

    def tick(self, now_ms: int) -> list[int]:
        published: list[int] = []
        for a in self._articles:
            if (a.state is ContentState.SCHEDULED
                    and a.scheduled_publish_at_ms is not None
                    and a.scheduled_publish_at_ms <= now_ms):
                a.state = ContentState.PUBLISHED
                a.published_at_ms = now_ms
                a.modified_at_ms = now_ms
                a.scheduled_publish_at_ms = None
                published.append(a.id)
        return published

    def revert_to_revision(self, aid: int, version: int, user: str,
                           now_ms: int) -> int:
        a = self._require(aid)
        if a.state is ContentState.ARCHIVED:
            raise ArchivedIsFrozen()
        revs = self._revisions[aid]
        target = next((r for r in revs if r.version == version), None)
        if target is None:
            raise UnknownRevision(aid, version)
        a.title = target.title
        a.body = target.body
        a.modified_at_ms = now_ms
        new_version = (revs[-1].version if revs else 0) + 1
        revs.append(Revision(
            article_id=aid, version=new_version,
            title=target.title, body=target.body,
            saved_at_ms=now_ms, saved_by=user,
        ))
        return new_version

    def set_tags(self, aid: int, tags: set[str], now_ms: int) -> None:
        a = self._require(aid)
        a.tags = set(tags)
        a.modified_at_ms = now_ms

    def article(self, aid: int) -> Article | None:
        return next((a for a in self._articles if a.id == aid), None)

    def article_by_slug(self, slug: str) -> Article | None:
        return next((a for a in self._articles if a.slug == slug), None)

    def articles(self) -> list[Article]:
        return list(self._articles)

    def by_state(self, state: ContentState) -> list[Article]:
        return [a for a in self._articles if a.state is state]

    def by_tag(self, tag: str) -> list[Article]:
        return [a for a in self._articles if tag in a.tags]

    def by_author(self, author: str) -> list[Article]:
        return [a for a in self._articles if a.author == author]

    def revisions_of(self, aid: int) -> list[Revision]:
        return list(self._revisions.get(aid, []))


if __name__ == "__main__":
    cms = Cms()
    a = cms.create_article("alice", "hello-world",
                           "Hello, World", "First post body.",
                           now_ms=1000)
    cms.save_revision(a, "alice", "Hello, World", "Edited body.", now_ms=2000)
    cms.set_tags(a, {"intro", "tutorial"}, now_ms=2000)
    cms.submit_for_review(a, now_ms=3000)
    cms.approve_and_publish(a, now_ms=4000)

    b = cms.create_article("alice", "announcement",
                           "Big Announcement", "Coming soon!", now_ms=5000)
    cms.schedule_publish(b, at_ms=100_000, now_ms=5000)

    print(f"articles: {len(cms.articles())}")
    print(f"published: {len(cms.by_state(ContentState.PUBLISHED))}")
    print(f"scheduled: {len(cms.by_state(ContentState.SCHEDULED))}")
    print(f"tagged 'tutorial': {len(cms.by_tag('tutorial'))}")
    print(f"revisions of '{cms.article(a).slug}': "
          f"{[r.version for r in cms.revisions_of(a)]}")

    # Tick past scheduled time.
    print()
    print(f"ticking to t=200_000...")
    published = cms.tick(200_000)
    print(f"newly published: {published}")
    print(f"  '{cms.article(b).slug}' state: {cms.article(b).state.value}")

    # Revert demo.
    print()
    v = cms.revert_to_revision(a, 1, "alice", now_ms=500_000)
    print(f"reverted to v1 → appended as v{v}. "
          f"Article body: {cms.article(a).body!r}")
