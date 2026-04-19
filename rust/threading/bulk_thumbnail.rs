//! Bulk Thumbnail Creator — CPU-bound fan-out with content-addressed dedup.
//!
//! G066's pattern was fan-out with isolated per-job results appended to a
//! list. G068 is fan-out with a **shared index** as the output: the result
//! IS the aggregation, not a side-effect of it. Workers compute content
//! hashes and check the index before generating a thumbnail — so the same
//! content processed twice (by two different source files, or in two
//! requests) produces one thumbnail, and the second worker's entry simply
//! records the already-computed result.
//!
//! Two sources of concurrency cost:
//!   1. Queue pop (inherited from G066).
//!   2. Index check-and-insert (new to G068). The critical section is larger
//!      than G066's one-line pop because it must atomically check "do we
//!      already have this hash?" and, if not, insert the thumbnail — a
//!      read-modify-write spanning two statements.
//!
//! The hasher and thumbnailer are parameters so the pattern works for any
//! content-addressed derivation: image → thumbnail, source code → AST,
//! document → summary, ledger entry → balance.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct ThumbnailRequest {
    pub source_id: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ThumbnailIndexSnapshot {
    /// content_hash -> thumbnail_token. One entry per unique content.
    pub thumbnails: HashMap<u64, String>,
    /// source_id -> content_hash. Multiple sources may map to the same hash.
    pub mappings: HashMap<String, u64>,
    pub generated: u64,     // thumbnails actually computed (non-dedup'd)
    pub deduplicated: u64,  // sources that hit an existing thumbnail
}

pub struct BulkThumbnailCreator {
    queue: Arc<Mutex<VecDeque<ThumbnailRequest>>>,
    thumbnails: Arc<Mutex<HashMap<u64, String>>>,
    mappings: Arc<Mutex<HashMap<String, u64>>>,
    generated: Arc<Mutex<u64>>,
    deduplicated: Arc<Mutex<u64>>,
    worker_count: usize,
}

impl BulkThumbnailCreator {
    pub fn new(worker_count: usize) -> Self {
        assert!(worker_count >= 1);
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            thumbnails: Arc::new(Mutex::new(HashMap::new())),
            mappings: Arc::new(Mutex::new(HashMap::new())),
            generated: Arc::new(Mutex::new(0)),
            deduplicated: Arc::new(Mutex::new(0)),
            worker_count,
        }
    }

    pub fn enqueue(&self, req: ThumbnailRequest) {
        self.queue.lock().unwrap().push_back(req);
    }

    pub fn run_with<H, T>(&self, hasher: H, thumbnailer: T) -> ThumbnailIndexSnapshot
    where
        H: Fn(&[u8]) -> u64 + Send + Sync + 'static,
        T: Fn(u64, &[u8]) -> String + Send + Sync + 'static,
    {
        let hasher = Arc::new(hasher);
        let thumbnailer = Arc::new(thumbnailer);
        let mut handles = Vec::with_capacity(self.worker_count);
        for _ in 0..self.worker_count {
            let queue = Arc::clone(&self.queue);
            let thumbnails = Arc::clone(&self.thumbnails);
            let mappings = Arc::clone(&self.mappings);
            let generated = Arc::clone(&self.generated);
            let deduplicated = Arc::clone(&self.deduplicated);
            let hasher = Arc::clone(&hasher);
            let thumbnailer = Arc::clone(&thumbnailer);
            handles.push(thread::spawn(move || {
                loop {
                    let job = queue.lock().unwrap().pop_front();
                    let Some(req) = job else { return; };
                    let hash = hasher(&req.content);

                    // Claim the slot under the lock using a placeholder, then
                    // run the expensive thumbnailer WITHOUT holding the lock.
                    // A concurrent worker with the same hash sees the placeholder
                    // and treats it as present — which is semantically correct:
                    // someone else IS generating this thumbnail.
                    let is_claimant = {
                        let mut idx = thumbnails.lock().unwrap();
                        if idx.contains_key(&hash) {
                            false
                        } else {
                            idx.insert(hash, String::new()); // reservation
                            true
                        }
                    };

                    if is_claimant {
                        let thumb = thumbnailer(hash, &req.content);
                        thumbnails.lock().unwrap().insert(hash, thumb);
                        *generated.lock().unwrap() += 1;
                    } else {
                        *deduplicated.lock().unwrap() += 1;
                    }
                    mappings.lock().unwrap().insert(req.source_id, hash);
                }
            }));
        }
        for h in handles { h.join().unwrap(); }

        ThumbnailIndexSnapshot {
            thumbnails: self.thumbnails.lock().unwrap().clone(),
            mappings: self.mappings.lock().unwrap().clone(),
            generated: *self.generated.lock().unwrap(),
            deduplicated: *self.deduplicated.lock().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn fake_hash(bytes: &[u8]) -> u64 {
        // Tiny deterministic hash: sum of bytes + length. Not cryptographic,
        // but enough to collide on identical content for the dedup tests.
        let mut h: u64 = bytes.len() as u64;
        for &b in bytes {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
        }
        h
    }

    fn fake_thumbnail(hash: u64, _content: &[u8]) -> String {
        format!("thumb_{hash:x}")
    }

    #[test]
    fn unique_sources_all_get_thumbnails() {
        let c = BulkThumbnailCreator::new(4);
        for i in 0..10 {
            c.enqueue(ThumbnailRequest {
                source_id: format!("img_{i}.jpg"),
                content: format!("unique content {i}").into_bytes(),
            });
        }
        let snap = c.run_with(fake_hash, fake_thumbnail);
        assert_eq!(snap.thumbnails.len(), 10);
        assert_eq!(snap.mappings.len(), 10);
        assert_eq!(snap.generated, 10);
        assert_eq!(snap.deduplicated, 0);
    }

    #[test]
    fn duplicate_content_produces_one_thumbnail() {
        let c = BulkThumbnailCreator::new(4);
        // Three sources with the same content.
        c.enqueue(ThumbnailRequest {
            source_id: "a.jpg".into(),
            content: b"same content".to_vec(),
        });
        c.enqueue(ThumbnailRequest {
            source_id: "b.jpg".into(),
            content: b"same content".to_vec(),
        });
        c.enqueue(ThumbnailRequest {
            source_id: "c.jpg".into(),
            content: b"same content".to_vec(),
        });
        let snap = c.run_with(fake_hash, fake_thumbnail);
        assert_eq!(snap.thumbnails.len(), 1, "only one unique content");
        assert_eq!(snap.mappings.len(), 3, "all three sources mapped");
        assert_eq!(snap.generated, 1);
        assert_eq!(snap.deduplicated, 2);
        // All three sources resolve to the same thumbnail.
        let hashes: Vec<_> = ["a.jpg", "b.jpg", "c.jpg"]
            .iter().map(|id| snap.mappings[*id]).collect();
        assert_eq!(hashes[0], hashes[1]);
        assert_eq!(hashes[1], hashes[2]);
    }

    #[test]
    fn mixed_unique_and_duplicate() {
        let c = BulkThumbnailCreator::new(3);
        // 5 unique contents, each used by 3 sources = 15 requests, 5 thumbnails.
        for kind in 0..5 {
            for copy in 0..3 {
                c.enqueue(ThumbnailRequest {
                    source_id: format!("src_{kind}_{copy}"),
                    content: format!("content kind {kind}").into_bytes(),
                });
            }
        }
        let snap = c.run_with(fake_hash, fake_thumbnail);
        assert_eq!(snap.thumbnails.len(), 5);
        assert_eq!(snap.mappings.len(), 15);
        assert_eq!(snap.generated, 5);
        assert_eq!(snap.deduplicated, 10);
    }

    #[test]
    fn parallel_faster_than_serial() {
        // Expensive thumbnailer: 30 ms per unique content. 8 unique contents,
        // 4 workers → ~60 ms (two rounds of 4). Serial would be ~240 ms.
        let c = BulkThumbnailCreator::new(4);
        for i in 0..8 {
            c.enqueue(ThumbnailRequest {
                source_id: format!("src_{i}"),
                content: format!("content {i}").into_bytes(),
            });
        }
        let slow_thumb = |h: u64, _: &[u8]| {
            thread::sleep(Duration::from_millis(30));
            format!("thumb_{h:x}")
        };
        let start = Instant::now();
        let snap = c.run_with(fake_hash, slow_thumb);
        let elapsed = start.elapsed();
        assert_eq!(snap.generated, 8);
        assert!(elapsed < Duration::from_millis(180),
                "expected parallel (~60-100ms), got {elapsed:?}");
    }

    #[test]
    fn concurrent_duplicate_insert_generates_exactly_once() {
        // 100 concurrent workers, 100 requests, all with identical content.
        // Only ONE thumbnail must be generated, regardless of how the
        // workers race on the check-and-insert.
        let c = BulkThumbnailCreator::new(8);
        for i in 0..100 {
            c.enqueue(ThumbnailRequest {
                source_id: format!("src_{i}"),
                content: b"identical content for all".to_vec(),
            });
        }
        let mut generation_counter = 0u64;
        // We can't capture a mut ref in a fn, so use a shared counter pattern.
        let gen_count = Arc::new(Mutex::new(0u64));
        let gen_clone = Arc::clone(&gen_count);
        let counting_thumb = move |h: u64, _: &[u8]| {
            *gen_clone.lock().unwrap() += 1;
            format!("thumb_{h:x}")
        };
        let snap = c.run_with(fake_hash, counting_thumb);
        generation_counter = *gen_count.lock().unwrap();
        assert_eq!(snap.thumbnails.len(), 1);
        assert_eq!(snap.generated, 1);
        assert_eq!(snap.deduplicated, 99);
        // And the thumbnailer itself ran exactly once — this is the real
        // correctness assertion, not just the counter in the snapshot.
        assert_eq!(generation_counter, 1,
                   "thumbnail function must have been invoked exactly once");
    }

    #[test]
    fn empty_queue_returns_empty_snapshot() {
        let c = BulkThumbnailCreator::new(2);
        let snap = c.run_with(fake_hash, fake_thumbnail);
        assert!(snap.thumbnails.is_empty());
        assert!(snap.mappings.is_empty());
        assert_eq!(snap.generated, 0);
        assert_eq!(snap.deduplicated, 0);
    }
}
