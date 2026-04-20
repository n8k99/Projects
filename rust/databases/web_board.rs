//! Web Board / Forum — threaded posts with vote tallies and hot-score ranking.
//!
//! Thirteenth Databases project. **Closes the Databases category.**
//! Introduces the **parent-pointer tree** — posts store `parent_id`
//! (Option<u64>), and the tree of replies is derived on read via DFS.
//! Every forum, Reddit-style threaded comment system, and email-like
//! reply tree uses this storage layout. Ranking adds a **hot-score**
//! formula (Reddit-style: log10(score) + age_penalty) so that new
//! content surfaces against old content.
//!
//! Distinctive moves:
//!   * **Parent-pointer tree** stored flat, rebuilt on query.
//!   * **Thread tree traversal** — DFS yielding `(post, depth)` pairs.
//!   * **Vote tally** — `upvotes − downvotes`; can go negative.
//!   * **Hot score** — `sign(s) * log10(max(1, |s|)) − age_days * weight`
//!      (a simplified Reddit hot function).
//!   * **Multi-order sort** — threads sortable by New / Top / Hot.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadSort { New, Top, Hot }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thread {
    pub id: u64,
    pub title: String,
    pub author: String,
    pub created_ms: i64,
    pub pinned: bool,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub id: u64,
    pub thread_id: u64,
    pub parent_id: Option<u64>,  // None = top-level post in thread
    pub author: String,
    pub body: String,
    pub created_ms: i64,
    pub upvotes: u32,
    pub downvotes: u32,
}

impl Post {
    pub fn score(&self) -> i64 { self.upvotes as i64 - self.downvotes as i64 }
}

/// Hot score: score's magnitude on a log scale, minus an age penalty.
pub fn hot_score(post: &Post, now_ms: i64) -> f64 {
    let s = post.score();
    let sign = if s > 0 { 1.0 } else if s < 0 { -1.0 } else { 0.0 };
    let magnitude = (s.unsigned_abs() as f64).max(1.0).log10();
    let age_days = (now_ms - post.created_ms).max(0) as f64 / 86_400_000.0;
    sign * magnitude - age_days * 0.2
}

/// Thread-level score for ranking: sum of vote scores across all posts in the thread.
pub fn thread_score(board: &Board, thread_id: u64) -> i64 {
    board.posts.values()
        .filter(|p| p.thread_id == thread_id)
        .map(|p| p.score())
        .sum()
}

/// Thread-level hot score: sum of per-post hot scores.
pub fn thread_hot_score(board: &Board, thread_id: u64, now_ms: i64) -> f64 {
    board.posts.values()
        .filter(|p| p.thread_id == thread_id)
        .map(|p| hot_score(p, now_ms))
        .sum()
}

#[derive(Debug, Clone)]
pub struct Board {
    pub threads: BTreeMap<u64, Thread>,
    pub posts: BTreeMap<u64, Post>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostWithDepth<'a> {
    pub post: &'a Post,
    pub depth: u32,
}

impl Board {
    pub fn new() -> Self {
        Self { threads: BTreeMap::new(), posts: BTreeMap::new() }
    }

    pub fn add_thread(&mut self, t: Thread) { self.threads.insert(t.id, t); }

    pub fn add_post(&mut self, p: Post) -> Result<(), String> {
        if !self.threads.contains_key(&p.thread_id) {
            return Err(format!("unknown thread {}", p.thread_id));
        }
        if let Some(pid) = p.parent_id {
            match self.posts.get(&pid) {
                None => return Err(format!("unknown parent {pid}")),
                Some(parent) if parent.thread_id != p.thread_id =>
                    return Err(format!("parent {pid} is in a different thread")),
                _ => {}
            }
        }
        self.posts.insert(p.id, p);
        Ok(())
    }

    /// Top-level posts in a thread (parent_id is None).
    pub fn top_level_posts(&self, thread_id: u64) -> Vec<&Post> {
        let mut out: Vec<&Post> = self.posts.values()
            .filter(|p| p.thread_id == thread_id && p.parent_id.is_none())
            .collect();
        out.sort_by_key(|p| p.created_ms);
        out
    }

    /// Children of a post in creation order.
    pub fn children_of(&self, post_id: u64) -> Vec<&Post> {
        let mut out: Vec<&Post> = self.posts.values()
            .filter(|p| p.parent_id == Some(post_id))
            .collect();
        out.sort_by_key(|p| p.created_ms);
        out
    }

    /// DFS the thread, yielding (post, depth). Deep replies indented per depth.
    pub fn thread_tree(&self, thread_id: u64) -> Vec<PostWithDepth> {
        let mut out = Vec::new();
        for top in self.top_level_posts(thread_id) {
            self.walk_subtree(top, 0, &mut out);
        }
        out
    }

    fn walk_subtree<'a>(&'a self, post: &'a Post, depth: u32, out: &mut Vec<PostWithDepth<'a>>) {
        out.push(PostWithDepth { post, depth });
        for child in self.children_of(post.id) {
            self.walk_subtree(child, depth + 1, out);
        }
    }

    /// List threads sorted by the given order. Pinned threads always come first.
    pub fn list_threads(&self, sort: ThreadSort, now_ms: i64) -> Vec<&Thread> {
        let mut out: Vec<&Thread> = self.threads.values().collect();
        out.sort_by(|a, b| {
            if a.pinned && !b.pinned { return std::cmp::Ordering::Less; }
            if !a.pinned && b.pinned { return std::cmp::Ordering::Greater; }
            match sort {
                ThreadSort::New => b.created_ms.cmp(&a.created_ms),
                ThreadSort::Top => thread_score(self, b.id).cmp(&thread_score(self, a.id)),
                ThreadSort::Hot => {
                    let ha = thread_hot_score(self, a.id, now_ms);
                    let hb = thread_hot_score(self, b.id, now_ms);
                    hb.partial_cmp(&ha).unwrap_or(std::cmp::Ordering::Equal)
                }
            }
        });
        out
    }

    pub fn post_count(&self, thread_id: u64) -> usize {
        self.posts.values().filter(|p| p.thread_id == thread_id).count()
    }
}

impl Default for Board { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_board() -> Board {
        let mut board = Board::new();
        board.add_thread(Thread {
            id: 1, title: "Welcome".into(), author: "alice".into(),
            created_ms: 1_000_000, pinned: true, locked: false,
        });
        board.add_thread(Thread {
            id: 2, title: "General discussion".into(), author: "bob".into(),
            created_ms: 2_000_000, pinned: false, locked: false,
        });
        // Thread 1 posts: root + two replies + nested reply
        board.add_post(Post {
            id: 10, thread_id: 1, parent_id: None,
            author: "alice".into(), body: "hi everyone".into(),
            created_ms: 1_000_000, upvotes: 10, downvotes: 1,
        }).unwrap();
        board.add_post(Post {
            id: 11, thread_id: 1, parent_id: Some(10),
            author: "bob".into(), body: "hi alice".into(),
            created_ms: 1_100_000, upvotes: 5, downvotes: 0,
        }).unwrap();
        board.add_post(Post {
            id: 12, thread_id: 1, parent_id: Some(10),
            author: "carol".into(), body: "hello".into(),
            created_ms: 1_200_000, upvotes: 3, downvotes: 0,
        }).unwrap();
        board.add_post(Post {
            id: 13, thread_id: 1, parent_id: Some(11),
            author: "alice".into(), body: "hey bob".into(),
            created_ms: 1_300_000, upvotes: 2, downvotes: 0,
        }).unwrap();
        // Thread 2 post
        board.add_post(Post {
            id: 20, thread_id: 2, parent_id: None,
            author: "bob".into(), body: "first!".into(),
            created_ms: 2_000_000, upvotes: 100, downvotes: 5,
        }).unwrap();
        board
    }

    #[test]
    fn post_score_can_go_negative() {
        let p = Post {
            id: 0, thread_id: 0, parent_id: None,
            author: "x".into(), body: "".into(),
            created_ms: 0, upvotes: 3, downvotes: 7,
        };
        assert_eq!(p.score(), -4);
    }

    #[test]
    fn hot_score_decays_with_age() {
        let old = Post {
            id: 0, thread_id: 0, parent_id: None,
            author: "x".into(), body: "".into(),
            created_ms: 0, upvotes: 10, downvotes: 0,
        };
        let now_0 = hot_score(&old, 0);
        let now_week = hot_score(&old, 7 * 86_400_000);
        assert!(now_0 > now_week);
    }

    #[test]
    fn thread_score_sums_all_posts() {
        let board = sample_board();
        // Thread 1: 9 + 5 + 3 + 2 = 19
        assert_eq!(thread_score(&board, 1), 19);
        // Thread 2: 95
        assert_eq!(thread_score(&board, 2), 95);
    }

    #[test]
    fn top_level_posts_filter_replies() {
        let board = sample_board();
        let top = board.top_level_posts(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].id, 10);
    }

    #[test]
    fn children_of_root_post() {
        let board = sample_board();
        let children = board.children_of(10);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].id, 11);  // ordered by created_ms
        assert_eq!(children[1].id, 12);
    }

    #[test]
    fn thread_tree_dfs_order() {
        let board = sample_board();
        let tree = board.thread_tree(1);
        let ids: Vec<u64> = tree.iter().map(|p| p.post.id).collect();
        // DFS: 10, 11, 13, 12  (13 is nested under 11)
        assert_eq!(ids, vec![10, 11, 13, 12]);
    }

    #[test]
    fn thread_tree_depth_tracking() {
        let board = sample_board();
        let tree = board.thread_tree(1);
        let depths: Vec<u32> = tree.iter().map(|p| p.depth).collect();
        assert_eq!(depths, vec![0, 1, 2, 1]);
    }

    #[test]
    fn pinned_threads_come_first_regardless_of_sort() {
        let board = sample_board();
        let by_new = board.list_threads(ThreadSort::New, 3_000_000);
        // Thread 1 is pinned, even though thread 2 is newer.
        assert_eq!(by_new[0].id, 1);
        let by_top = board.list_threads(ThreadSort::Top, 3_000_000);
        // Thread 2 has higher score (95 vs 19), but thread 1 is pinned.
        assert_eq!(by_top[0].id, 1);
    }

    #[test]
    fn new_sort_orders_by_created_desc() {
        let mut board = Board::new();
        board.add_thread(Thread { id: 1, title: "old".into(), author: "x".into(),
                                    created_ms: 100, pinned: false, locked: false });
        board.add_thread(Thread { id: 2, title: "new".into(), author: "y".into(),
                                    created_ms: 200, pinned: false, locked: false });
        let sorted = board.list_threads(ThreadSort::New, 300);
        assert_eq!(sorted[0].id, 2);
    }

    #[test]
    fn add_post_errors_on_unknown_thread() {
        let mut board = sample_board();
        let err = board.add_post(Post {
            id: 99, thread_id: 999, parent_id: None,
            author: "x".into(), body: "".into(),
            created_ms: 0, upvotes: 0, downvotes: 0,
        });
        assert!(err.is_err());
    }

    #[test]
    fn add_post_errors_on_unknown_parent() {
        let mut board = sample_board();
        let err = board.add_post(Post {
            id: 99, thread_id: 1, parent_id: Some(9999),
            author: "x".into(), body: "".into(),
            created_ms: 0, upvotes: 0, downvotes: 0,
        });
        assert!(err.is_err());
    }

    #[test]
    fn add_post_errors_if_parent_in_different_thread() {
        let mut board = sample_board();
        // parent 20 is in thread 2, but we're posting to thread 1
        let err = board.add_post(Post {
            id: 99, thread_id: 1, parent_id: Some(20),
            author: "x".into(), body: "".into(),
            created_ms: 0, upvotes: 0, downvotes: 0,
        });
        assert!(err.is_err());
    }

    #[test]
    fn post_count_per_thread() {
        let board = sample_board();
        assert_eq!(board.post_count(1), 4);
        assert_eq!(board.post_count(2), 1);
    }

    #[test]
    fn empty_thread_has_empty_tree() {
        let mut board = Board::new();
        board.add_thread(Thread { id: 1, title: "empty".into(), author: "x".into(),
                                    created_ms: 0, pinned: false, locked: false });
        assert!(board.thread_tree(1).is_empty());
    }

    #[test]
    fn hot_sort_prefers_recent_high_score() {
        let mut board = Board::new();
        board.add_thread(Thread { id: 1, title: "old-popular".into(), author: "x".into(),
                                    created_ms: 0, pinned: false, locked: false });
        board.add_thread(Thread { id: 2, title: "new-okay".into(), author: "y".into(),
                                    created_ms: 7 * 86_400_000, pinned: false, locked: false });
        board.add_post(Post { id: 1, thread_id: 1, parent_id: None, author: "x".into(), body: "".into(),
                                created_ms: 0, upvotes: 100, downvotes: 0 }).unwrap();
        board.add_post(Post { id: 2, thread_id: 2, parent_id: None, author: "y".into(), body: "".into(),
                                created_ms: 7 * 86_400_000, upvotes: 10, downvotes: 0 }).unwrap();
        let now = 7 * 86_400_000;
        let sorted = board.list_threads(ThreadSort::Hot, now);
        // Newer thread with score 10 beats week-old thread with score 100
        // under hot-score decay.
        assert_eq!(sorted[0].id, 2);
    }
}
