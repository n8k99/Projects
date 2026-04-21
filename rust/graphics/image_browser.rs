//! Image Browser — 2D grid navigation + LRU cache + filter/sort query.
//!
//! Eleventh Graphics project. Distinctive moves:
//!
//!   * **2D grid cursor with wrap-around** — `move(direction)` maps a
//!     1D selected_idx through a (cols × rows) grid. Right at row end
//!     wraps to next row start; Down past last row clamps.
//!   * **LRU thumbnail cache** — capacity-bounded with recency-based
//!     eviction. Distinct from G123's FIFO ring: a cache hit *refreshes*
//!     an entry's position; least-recently-used is evicted on insertion.
//!   * **Filter+sort query as a data value** — list of `FilterPredicate`
//!     composed AND-wise, one `SortBy` + `Order`. Pure function over
//!     entries; same query, same result.
//!   * **Breadcrumb path stack** — navigation is push/pop; current path
//!     is `breadcrumbs.join('/')`. `navigate_up` pops; `navigate_into`
//!     pushes; path rendering is a fold.

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageEntry {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub size_bytes: u64,
    pub tags: Vec<String>,
    pub created_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterPredicate {
    NameContains(String),
    TagHas(String),
    SizeUnder(u64),
    SizeOver(u64),
    WidthAtLeast(u32),
    CreatedAfter(u64),
}

impl FilterPredicate {
    pub fn matches(&self, e: &ImageEntry) -> bool {
        match self {
            FilterPredicate::NameContains(s) => e.name.contains(s),
            FilterPredicate::TagHas(s) => e.tags.iter().any(|t| t == s),
            FilterPredicate::SizeUnder(n) => e.size_bytes < *n,
            FilterPredicate::SizeOver(n) => e.size_bytes > *n,
            FilterPredicate::WidthAtLeast(n) => e.width >= *n,
            FilterPredicate::CreatedAfter(n) => e.created_ms > *n,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy { Name, Size, Created, Width }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order { Asc, Desc }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub filters: Vec<FilterPredicate>,
    pub sort_by: SortBy,
    pub order: Order,
}

impl Default for Query {
    fn default() -> Self {
        Self { filters: vec![], sort_by: SortBy::Name, order: Order::Asc }
    }
}

pub fn apply_query<'a>(entries: &'a [ImageEntry], query: &Query) -> Vec<&'a ImageEntry> {
    let mut filtered: Vec<&ImageEntry> = entries.iter()
        .filter(|e| query.filters.iter().all(|p| p.matches(e)))
        .collect();
    filtered.sort_by(|a, b| {
        let ord = match query.sort_by {
            SortBy::Name => a.name.cmp(&b.name),
            SortBy::Size => a.size_bytes.cmp(&b.size_bytes),
            SortBy::Created => a.created_ms.cmp(&b.created_ms),
            SortBy::Width => a.width.cmp(&b.width),
        };
        if query.order == Order::Desc { ord.reverse() } else { ord }
    });
    filtered
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridMove { Left, Right, Up, Down, Home, End }

#[derive(Debug, Clone)]
pub struct Grid {
    pub cols: u32,
    pub total: u32,
    pub selected: u32,
}

impl Grid {
    pub fn new(cols: u32, total: u32) -> Self {
        Self { cols, total, selected: 0 }
    }

    pub fn rows(&self) -> u32 {
        if self.cols == 0 || self.total == 0 { return 0; }
        (self.total + self.cols - 1) / self.cols
    }

    pub fn selected_row_col(&self) -> (u32, u32) {
        (self.selected / self.cols, self.selected % self.cols)
    }

    pub fn move_cursor(&mut self, mv: GridMove) {
        if self.total == 0 { return; }
        let (row, col) = self.selected_row_col();
        match mv {
            GridMove::Left => {
                if self.selected > 0 { self.selected -= 1; }
            }
            GridMove::Right => {
                if self.selected + 1 < self.total { self.selected += 1; }
            }
            GridMove::Up => {
                if row > 0 {
                    self.selected = (row - 1) * self.cols + col;
                }
            }
            GridMove::Down => {
                let next = (row + 1) * self.cols + col;
                if next < self.total { self.selected = next; }
                else if row + 1 < self.rows() {
                    // last row is partial: clamp to last entry
                    self.selected = self.total - 1;
                }
            }
            GridMove::Home => { self.selected = 0; }
            GridMove::End => {
                if self.total > 0 { self.selected = self.total - 1; }
            }
        }
    }
}

/// LRU cache: capacity-bounded map where access refreshes recency.
#[derive(Debug, Clone)]
pub struct LruCache<V: Clone> {
    pub capacity: usize,
    order: VecDeque<u64>, // most recently used at back
    entries: HashMap<u64, V>,
}

impl<V: Clone> LruCache<V> {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, order: VecDeque::new(), entries: HashMap::new() }
    }

    pub fn get(&mut self, k: u64) -> Option<V> {
        if !self.entries.contains_key(&k) { return None; }
        // refresh recency
        self.order.retain(|x| *x != k);
        self.order.push_back(k);
        self.entries.get(&k).cloned()
    }

    pub fn put(&mut self, k: u64, v: V) -> Option<u64> {
        if self.entries.contains_key(&k) {
            self.order.retain(|x| *x != k);
            self.order.push_back(k);
            self.entries.insert(k, v);
            return None;
        }
        if self.entries.len() >= self.capacity {
            if let Some(evicted) = self.order.pop_front() {
                self.entries.remove(&evicted);
                self.entries.insert(k, v);
                self.order.push_back(k);
                return Some(evicted);
            }
        }
        self.entries.insert(k, v);
        self.order.push_back(k);
        None
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn contains(&self, k: u64) -> bool { self.entries.contains_key(&k) }
}

#[derive(Debug, Clone)]
pub struct Browser {
    pub breadcrumbs: Vec<String>,
    pub all_entries: Vec<ImageEntry>,
    pub query: Query,
    pub grid_cols: u32,
    pub grid: Grid,
    pub thumbnail_cache: LruCache<Vec<u8>>, // thumbnail pixel blob keyed by entry id
}

impl Browser {
    pub fn new(grid_cols: u32, thumbnail_cache_capacity: usize) -> Self {
        Self {
            breadcrumbs: vec![],
            all_entries: vec![],
            query: Query::default(),
            grid_cols,
            grid: Grid::new(grid_cols, 0),
            thumbnail_cache: LruCache::new(thumbnail_cache_capacity),
        }
    }

    pub fn set_entries(&mut self, entries: Vec<ImageEntry>) {
        self.all_entries = entries;
        let filtered = self.filtered_ids();
        self.grid = Grid::new(self.grid_cols, filtered.len() as u32);
    }

    pub fn set_query(&mut self, query: Query) {
        self.query = query;
        let filtered = self.filtered_ids();
        self.grid = Grid::new(self.grid_cols, filtered.len() as u32);
    }

    fn filtered_ids(&self) -> Vec<u64> {
        apply_query(&self.all_entries, &self.query)
            .iter().map(|e| e.id).collect()
    }

    pub fn filtered_entries(&self) -> Vec<ImageEntry> {
        apply_query(&self.all_entries, &self.query)
            .iter().map(|e| (*e).clone()).collect()
    }

    pub fn selected_entry(&self) -> Option<ImageEntry> {
        self.filtered_entries().into_iter().nth(self.grid.selected as usize)
    }

    pub fn navigate_into(&mut self, subdir: impl Into<String>) {
        self.breadcrumbs.push(subdir.into());
    }

    pub fn navigate_up(&mut self) -> bool {
        self.breadcrumbs.pop().is_some()
    }

    pub fn current_path(&self) -> String {
        "/".to_string() + &self.breadcrumbs.join("/")
    }

    pub fn move_cursor(&mut self, mv: GridMove) {
        self.grid.move_cursor(mv);
    }

    /// Returns the thumbnail if cached, else computes via the producer
    /// and caches it. Returns whether a cache miss occurred.
    pub fn get_or_load_thumbnail(&mut self, id: u64,
                                  producer: impl FnOnce() -> Vec<u8>)
        -> (Vec<u8>, bool)
    {
        if let Some(v) = self.thumbnail_cache.get(id) {
            return (v, false);
        }
        let v = producer();
        self.thumbnail_cache.put(id, v.clone());
        (v, true)
    }
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: u64, name: &str, size: u64, created: u64, tags: &[&str]) -> ImageEntry {
        ImageEntry {
            id, name: name.to_string(), path: format!("/imgs/{}", name),
            width: 800, height: 600, size_bytes: size, created_ms: created,
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn filter_name_contains() {
        let entries = vec![
            entry(1, "photo-1.jpg", 100, 0, &[]),
            entry(2, "doc.png", 200, 0, &[]),
            entry(3, "photo-2.jpg", 300, 0, &[]),
        ];
        let q = Query {
            filters: vec![FilterPredicate::NameContains("photo".to_string())],
            ..Query::default()
        };
        let r = apply_query(&entries, &q);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].id, 1);
        assert_eq!(r[1].id, 3);
    }

    #[test]
    fn filter_combined_and() {
        let entries = vec![
            entry(1, "photo-1.jpg", 100, 0, &["nature"]),
            entry(2, "photo-2.jpg", 1000, 0, &["nature"]),
            entry(3, "photo-3.jpg", 500, 0, &["abstract"]),
        ];
        let q = Query {
            filters: vec![
                FilterPredicate::NameContains("photo".to_string()),
                FilterPredicate::TagHas("nature".to_string()),
                FilterPredicate::SizeOver(500),
            ],
            ..Query::default()
        };
        let r = apply_query(&entries, &q);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, 2);
    }

    #[test]
    fn sort_by_size_desc() {
        let entries = vec![
            entry(1, "a", 100, 0, &[]),
            entry(2, "b", 300, 0, &[]),
            entry(3, "c", 200, 0, &[]),
        ];
        let q = Query {
            filters: vec![], sort_by: SortBy::Size, order: Order::Desc,
        };
        let r = apply_query(&entries, &q);
        assert_eq!(r.iter().map(|e| e.id).collect::<Vec<_>>(), vec![2, 3, 1]);
    }

    #[test]
    fn grid_move_right_across_rows() {
        let mut g = Grid::new(3, 9);
        g.move_cursor(GridMove::Right);
        g.move_cursor(GridMove::Right);
        assert_eq!(g.selected, 2);
        g.move_cursor(GridMove::Right);
        assert_eq!(g.selected, 3); // wrapped to next row
    }

    #[test]
    fn grid_move_down_same_column() {
        let mut g = Grid::new(3, 9);
        g.move_cursor(GridMove::Right); // col 1
        g.move_cursor(GridMove::Down);
        assert_eq!(g.selected, 4); // col 1, row 1
    }

    #[test]
    fn grid_move_up_from_top_noop() {
        let mut g = Grid::new(3, 9);
        g.move_cursor(GridMove::Right);
        g.move_cursor(GridMove::Up);
        assert_eq!(g.selected, 1); // already top
    }

    #[test]
    fn grid_move_down_partial_last_row() {
        let mut g = Grid::new(3, 7); // 3 cols, 7 items = 3 rows (last row has 1)
        g.move_cursor(GridMove::Right);
        g.move_cursor(GridMove::Right); // at col 2, row 0
        g.move_cursor(GridMove::Down); // col 2, row 1 → idx 5
        assert_eq!(g.selected, 5);
        g.move_cursor(GridMove::Down);
        // row 2 col 2 = idx 8 which is >= total 7, so clamp to 6
        assert_eq!(g.selected, 6);
    }

    #[test]
    fn grid_home_end() {
        let mut g = Grid::new(3, 7);
        g.selected = 3;
        g.move_cursor(GridMove::End);
        assert_eq!(g.selected, 6);
        g.move_cursor(GridMove::Home);
        assert_eq!(g.selected, 0);
    }

    #[test]
    fn lru_cache_basic() {
        let mut c: LruCache<i32> = LruCache::new(3);
        assert!(c.put(1, 10).is_none());
        assert!(c.put(2, 20).is_none());
        assert!(c.put(3, 30).is_none());
        assert_eq!(c.get(1), Some(10));
        // insert 4, evicts 2 (LRU after 1 was touched)
        assert_eq!(c.put(4, 40), Some(2));
        assert!(c.contains(1));
        assert!(!c.contains(2));
        assert!(c.contains(3));
        assert!(c.contains(4));
    }

    #[test]
    fn lru_get_refreshes_recency() {
        let mut c: LruCache<i32> = LruCache::new(2);
        c.put(1, 10);
        c.put(2, 20);
        c.get(1); // 1 is now MRU
        let evicted = c.put(3, 30);
        assert_eq!(evicted, Some(2)); // 2 is LRU
    }

    #[test]
    fn lru_put_same_key_updates_no_evict() {
        let mut c: LruCache<i32> = LruCache::new(2);
        c.put(1, 10);
        c.put(2, 20);
        assert!(c.put(1, 100).is_none());
        assert_eq!(c.get(1), Some(100));
    }

    #[test]
    fn browser_filter_updates_grid_size() {
        let mut br = Browser::new(3, 10);
        br.set_entries(vec![
            entry(1, "a.jpg", 100, 0, &["a"]),
            entry(2, "b.png", 200, 0, &["b"]),
            entry(3, "c.jpg", 300, 0, &["a"]),
        ]);
        assert_eq!(br.grid.total, 3);
        br.set_query(Query {
            filters: vec![FilterPredicate::NameContains(".jpg".to_string())],
            ..Query::default()
        });
        assert_eq!(br.grid.total, 2);
    }

    #[test]
    fn browser_selected_entry_reflects_grid_cursor() {
        let mut br = Browser::new(3, 10);
        br.set_entries(vec![
            entry(1, "a", 100, 0, &[]),
            entry(2, "b", 200, 0, &[]),
            entry(3, "c", 300, 0, &[]),
        ]);
        br.move_cursor(GridMove::Right);
        assert_eq!(br.selected_entry().unwrap().id, 2);
    }

    #[test]
    fn breadcrumb_stack_navigation() {
        let mut br = Browser::new(3, 10);
        br.navigate_into("photos");
        br.navigate_into("2026");
        assert_eq!(br.current_path(), "/photos/2026");
        br.navigate_up();
        assert_eq!(br.current_path(), "/photos");
        br.navigate_up();
        assert_eq!(br.current_path(), "/");
        assert!(!br.navigate_up()); // pop on empty returns false
    }

    #[test]
    fn thumbnail_cache_miss_then_hit() {
        let mut br = Browser::new(3, 2);
        let mut invocations = 0;
        {
            let (_, miss) = br.get_or_load_thumbnail(1, || { invocations += 1; vec![1, 2, 3] });
            assert!(miss);
        }
        {
            let (_, miss) = br.get_or_load_thumbnail(1, || { invocations += 1; vec![9, 9, 9] });
            assert!(!miss); // cache hit; producer not called
        }
        assert_eq!(invocations, 1);
    }

    #[test]
    fn filter_then_sort_composition() {
        let entries = vec![
            entry(1, "beta.jpg", 200, 1000, &["keep"]),
            entry(2, "alpha.jpg", 100, 2000, &["keep"]),
            entry(3, "gamma.jpg", 300, 3000, &["drop"]),
            entry(4, "delta.jpg", 400, 4000, &["keep"]),
        ];
        let q = Query {
            filters: vec![FilterPredicate::TagHas("keep".to_string())],
            sort_by: SortBy::Size, order: Order::Asc,
        };
        let r = apply_query(&entries, &q);
        // tag="keep": {1, 2, 4}; by size asc: 2 (100), 1 (200), 4 (400)
        assert_eq!(r.iter().map(|e| e.id).collect::<Vec<_>>(), vec![2, 1, 4]);
    }
}
