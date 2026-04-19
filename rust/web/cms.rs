//! Content Management System — lifecycle FSM + revisions + scheduled publish.
//!
//! Articles progress through a state machine: Draft → InReview → Published →
//! Archived. A scheduled publish puts an article in `Scheduled` state until
//! the scheduled time arrives, at which point `tick(now)` transitions it to
//! Published. Every save creates a new revision; `revert_to_revision` restores
//! an old body by creating a *new* revision with that content (history is
//! append-only — reverts don't erase, they add).
//!
//! This composes G080's scheduler pattern (tick + scheduled firing),
//! G077's state-gated operations (actions allowed only in certain states),
//! and G081's content/presentation separation (articles are structured,
//! rendering is someone else's problem).

use std::collections::{BTreeSet, HashMap};

pub type ArticleId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentState {
    Draft,
    InReview,
    Scheduled,
    Published,
    Archived,
}

#[derive(Debug, Clone)]
pub struct Article {
    pub id: ArticleId,
    pub slug: String,
    pub title: String,
    pub body: String,
    pub author: String,
    pub state: ContentState,
    pub tags: BTreeSet<String>,
    pub created_at_ms: u64,
    pub modified_at_ms: u64,
    pub scheduled_publish_at_ms: Option<u64>,
    pub published_at_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Revision {
    pub article_id: ArticleId,
    pub version: u32,
    pub title: String,
    pub body: String,
    pub saved_at_ms: u64,
    pub saved_by: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmsError {
    UnknownArticle(ArticleId),
    UnknownRevision(ArticleId, u32),
    IllegalTransition { from: ContentState, action: String },
    ArchivedIsFrozen,
    SlugTaken(String),
}

pub struct Cms {
    articles: Vec<Article>,
    /// article_id → list of revisions (version ascending).
    revisions: HashMap<ArticleId, Vec<Revision>>,
    next_id: ArticleId,
}

impl Cms {
    pub fn new() -> Self {
        Self { articles: Vec::new(), revisions: HashMap::new(), next_id: 1 }
    }

    pub fn create_article(
        &mut self, author: &str, slug: &str, title: &str, body: &str, now_ms: u64,
    ) -> Result<ArticleId, CmsError> {
        if self.articles.iter().any(|a| a.slug == slug) {
            return Err(CmsError::SlugTaken(slug.into()));
        }
        let id = self.next_id;
        self.next_id += 1;
        self.articles.push(Article {
            id, slug: slug.into(), title: title.into(), body: body.into(),
            author: author.into(),
            state: ContentState::Draft,
            tags: BTreeSet::new(),
            created_at_ms: now_ms, modified_at_ms: now_ms,
            scheduled_publish_at_ms: None, published_at_ms: None,
        });
        self.revisions.insert(id, vec![Revision {
            article_id: id, version: 1,
            title: title.into(), body: body.into(),
            saved_at_ms: now_ms, saved_by: author.into(),
        }]);
        Ok(id)
    }

    pub fn save_revision(
        &mut self, id: ArticleId, user: &str, title: &str, body: &str, now_ms: u64,
    ) -> Result<u32, CmsError> {
        let article = self.articles.iter_mut().find(|a| a.id == id)
            .ok_or(CmsError::UnknownArticle(id))?;
        if article.state == ContentState::Archived {
            return Err(CmsError::ArchivedIsFrozen);
        }
        article.title = title.into();
        article.body = body.into();
        article.modified_at_ms = now_ms;
        let revs = self.revisions.get_mut(&id).unwrap();
        let version = revs.last().map(|r| r.version).unwrap_or(0) + 1;
        revs.push(Revision {
            article_id: id, version,
            title: title.into(), body: body.into(),
            saved_at_ms: now_ms, saved_by: user.into(),
        });
        Ok(version)
    }

    pub fn submit_for_review(&mut self, id: ArticleId, now_ms: u64)
        -> Result<(), CmsError> {
        let article = self.articles.iter_mut().find(|a| a.id == id)
            .ok_or(CmsError::UnknownArticle(id))?;
        match article.state {
            ContentState::Draft => {
                article.state = ContentState::InReview;
                article.modified_at_ms = now_ms;
                Ok(())
            }
            s => Err(CmsError::IllegalTransition { from: s, action: "submit_for_review".into() }),
        }
    }

    pub fn approve_and_publish(&mut self, id: ArticleId, now_ms: u64)
        -> Result<(), CmsError> {
        let article = self.articles.iter_mut().find(|a| a.id == id)
            .ok_or(CmsError::UnknownArticle(id))?;
        match article.state {
            ContentState::Draft | ContentState::InReview | ContentState::Scheduled => {
                article.state = ContentState::Published;
                article.modified_at_ms = now_ms;
                article.published_at_ms = Some(now_ms);
                article.scheduled_publish_at_ms = None;
                Ok(())
            }
            s => Err(CmsError::IllegalTransition { from: s, action: "approve_and_publish".into() }),
        }
    }

    pub fn schedule_publish(&mut self, id: ArticleId, at_ms: u64, now_ms: u64)
        -> Result<(), CmsError> {
        let article = self.articles.iter_mut().find(|a| a.id == id)
            .ok_or(CmsError::UnknownArticle(id))?;
        match article.state {
            ContentState::Draft | ContentState::InReview | ContentState::Scheduled => {
                article.state = ContentState::Scheduled;
                article.scheduled_publish_at_ms = Some(at_ms);
                article.modified_at_ms = now_ms;
                Ok(())
            }
            s => Err(CmsError::IllegalTransition { from: s, action: "schedule_publish".into() }),
        }
    }

    pub fn archive(&mut self, id: ArticleId, now_ms: u64) -> Result<(), CmsError> {
        let article = self.articles.iter_mut().find(|a| a.id == id)
            .ok_or(CmsError::UnknownArticle(id))?;
        if article.state == ContentState::Archived {
            return Err(CmsError::IllegalTransition {
                from: article.state, action: "archive".into() });
        }
        article.state = ContentState::Archived;
        article.modified_at_ms = now_ms;
        Ok(())
    }

    pub fn unarchive(&mut self, id: ArticleId, now_ms: u64) -> Result<(), CmsError> {
        let article = self.articles.iter_mut().find(|a| a.id == id)
            .ok_or(CmsError::UnknownArticle(id))?;
        match article.state {
            ContentState::Archived => {
                article.state = ContentState::Draft;
                article.modified_at_ms = now_ms;
                Ok(())
            }
            s => Err(CmsError::IllegalTransition { from: s, action: "unarchive".into() }),
        }
    }

    /// Fire every Scheduled article whose scheduled_publish_at_ms ≤ now_ms.
    /// Returns the ids that transitioned to Published on this tick.
    pub fn tick(&mut self, now_ms: u64) -> Vec<ArticleId> {
        let mut published = Vec::new();
        for article in self.articles.iter_mut() {
            if article.state == ContentState::Scheduled {
                if let Some(at) = article.scheduled_publish_at_ms {
                    if at <= now_ms {
                        article.state = ContentState::Published;
                        article.published_at_ms = Some(now_ms);
                        article.modified_at_ms = now_ms;
                        article.scheduled_publish_at_ms = None;
                        published.push(article.id);
                    }
                }
            }
        }
        published
    }

    /// Revert to a specific version: copies that revision's title/body into
    /// the article and APPENDS a new revision (history is append-only).
    pub fn revert_to_revision(&mut self, id: ArticleId, version: u32, user: &str, now_ms: u64)
        -> Result<u32, CmsError> {
        let article = self.articles.iter_mut().find(|a| a.id == id)
            .ok_or(CmsError::UnknownArticle(id))?;
        if article.state == ContentState::Archived {
            return Err(CmsError::ArchivedIsFrozen);
        }
        let revs = self.revisions.get_mut(&id).unwrap();
        let target = revs.iter().find(|r| r.version == version)
            .ok_or(CmsError::UnknownRevision(id, version))?;
        let old_title = target.title.clone();
        let old_body = target.body.clone();
        article.title = old_title.clone();
        article.body = old_body.clone();
        article.modified_at_ms = now_ms;
        let new_version = revs.last().map(|r| r.version).unwrap_or(0) + 1;
        revs.push(Revision {
            article_id: id, version: new_version,
            title: old_title, body: old_body,
            saved_at_ms: now_ms, saved_by: user.into(),
        });
        Ok(new_version)
    }

    pub fn set_tags(&mut self, id: ArticleId, tags: BTreeSet<String>, now_ms: u64)
        -> Result<(), CmsError> {
        let article = self.articles.iter_mut().find(|a| a.id == id)
            .ok_or(CmsError::UnknownArticle(id))?;
        article.tags = tags;
        article.modified_at_ms = now_ms;
        Ok(())
    }

    pub fn article(&self, id: ArticleId) -> Option<&Article> {
        self.articles.iter().find(|a| a.id == id)
    }
    pub fn article_by_slug(&self, slug: &str) -> Option<&Article> {
        self.articles.iter().find(|a| a.slug == slug)
    }
    pub fn articles(&self) -> &[Article] { &self.articles }

    pub fn by_state(&self, state: ContentState) -> Vec<&Article> {
        self.articles.iter().filter(|a| a.state == state).collect()
    }
    pub fn by_tag(&self, tag: &str) -> Vec<&Article> {
        self.articles.iter().filter(|a| a.tags.contains(tag)).collect()
    }
    pub fn by_author(&self, author: &str) -> Vec<&Article> {
        self.articles.iter().filter(|a| a.author == author).collect()
    }

    pub fn revisions_of(&self, id: ArticleId) -> &[Revision] {
        self.revisions.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }
}

impl Default for Cms {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_starts_in_draft() {
        let mut c = Cms::new();
        let id = c.create_article("alice", "hello", "Hello", "Body", 1000).unwrap();
        let a = c.article(id).unwrap();
        assert_eq!(a.state, ContentState::Draft);
        assert_eq!(c.revisions_of(id).len(), 1);
    }

    #[test]
    fn duplicate_slug_rejected() {
        let mut c = Cms::new();
        c.create_article("alice", "hello", "T1", "B1", 1000).unwrap();
        let err = c.create_article("bob", "hello", "T2", "B2", 1000);
        assert_eq!(err, Err(CmsError::SlugTaken("hello".into())));
    }

    #[test]
    fn save_revision_appends_version() {
        let mut c = Cms::new();
        let id = c.create_article("alice", "x", "T", "B1", 1000).unwrap();
        let v2 = c.save_revision(id, "alice", "T", "B2", 2000).unwrap();
        let v3 = c.save_revision(id, "alice", "T", "B3", 3000).unwrap();
        assert_eq!(v2, 2);
        assert_eq!(v3, 3);
        assert_eq!(c.revisions_of(id).len(), 3);
        assert_eq!(c.article(id).unwrap().body, "B3");
    }

    #[test]
    fn submit_for_review_from_draft_only() {
        let mut c = Cms::new();
        let id = c.create_article("alice", "x", "T", "B", 0).unwrap();
        c.submit_for_review(id, 100).unwrap();
        assert_eq!(c.article(id).unwrap().state, ContentState::InReview);
        // Submitting again is illegal.
        let err = c.submit_for_review(id, 200);
        assert!(matches!(err, Err(CmsError::IllegalTransition { .. })));
    }

    #[test]
    fn approve_moves_to_published_sets_timestamp() {
        let mut c = Cms::new();
        let id = c.create_article("alice", "x", "T", "B", 0).unwrap();
        c.submit_for_review(id, 100).unwrap();
        c.approve_and_publish(id, 200).unwrap();
        let a = c.article(id).unwrap();
        assert_eq!(a.state, ContentState::Published);
        assert_eq!(a.published_at_ms, Some(200));
    }

    #[test]
    fn scheduled_article_publishes_on_tick() {
        let mut c = Cms::new();
        let id = c.create_article("alice", "x", "T", "B", 0).unwrap();
        c.schedule_publish(id, 1000, 100).unwrap();
        assert_eq!(c.article(id).unwrap().state, ContentState::Scheduled);

        let published = c.tick(500);
        assert!(published.is_empty());
        let published = c.tick(1500);
        assert_eq!(published, vec![id]);
        let a = c.article(id).unwrap();
        assert_eq!(a.state, ContentState::Published);
        assert_eq!(a.scheduled_publish_at_ms, None);
    }

    #[test]
    fn archive_and_unarchive_round_trip() {
        let mut c = Cms::new();
        let id = c.create_article("alice", "x", "T", "B", 0).unwrap();
        c.archive(id, 1000).unwrap();
        assert_eq!(c.article(id).unwrap().state, ContentState::Archived);
        // Archived is frozen.
        let err = c.save_revision(id, "alice", "T", "B2", 2000);
        assert_eq!(err, Err(CmsError::ArchivedIsFrozen));
        c.unarchive(id, 3000).unwrap();
        assert_eq!(c.article(id).unwrap().state, ContentState::Draft);
    }

    #[test]
    fn revert_adds_new_revision_with_old_body() {
        let mut c = Cms::new();
        let id = c.create_article("alice", "x", "T", "B1", 0).unwrap();
        c.save_revision(id, "alice", "T", "B2", 100).unwrap();
        c.save_revision(id, "alice", "T", "B3", 200).unwrap();
        let v4 = c.revert_to_revision(id, 1, "alice", 300).unwrap();
        assert_eq!(v4, 4);
        let a = c.article(id).unwrap();
        assert_eq!(a.body, "B1");
        assert_eq!(c.revisions_of(id).len(), 4, "revert appends, doesn't erase");
    }

    #[test]
    fn by_state_filters() {
        let mut c = Cms::new();
        let a = c.create_article("alice", "a", "T", "B", 0).unwrap();
        let b = c.create_article("alice", "b", "T", "B", 0).unwrap();
        let _d = c.create_article("alice", "d", "T", "B", 0).unwrap();
        c.submit_for_review(a, 100).unwrap();
        c.approve_and_publish(b, 100).unwrap();
        assert_eq!(c.by_state(ContentState::Draft).len(), 1);
        assert_eq!(c.by_state(ContentState::InReview).len(), 1);
        assert_eq!(c.by_state(ContentState::Published).len(), 1);
    }

    #[test]
    fn by_tag_and_by_author_filter() {
        let mut c = Cms::new();
        let id1 = c.create_article("alice", "a", "T", "B", 0).unwrap();
        let id2 = c.create_article("bob", "b", "T", "B", 0).unwrap();
        let _ = c.create_article("alice", "c", "T", "B", 0).unwrap();
        c.set_tags(id1, ["tutorial".into()].into_iter().collect(), 100).unwrap();
        c.set_tags(id2, ["tutorial".into(), "intro".into()].into_iter().collect(), 100).unwrap();
        assert_eq!(c.by_tag("tutorial").len(), 2);
        assert_eq!(c.by_tag("intro").len(), 1);
        assert_eq!(c.by_author("alice").len(), 2);
    }

    #[test]
    fn scheduled_publish_over_unknown_state_refused() {
        let mut c = Cms::new();
        let id = c.create_article("alice", "x", "T", "B", 0).unwrap();
        c.approve_and_publish(id, 100).unwrap();
        let err = c.schedule_publish(id, 2000, 200);
        assert!(matches!(err, Err(CmsError::IllegalTransition { .. })));
    }
}
