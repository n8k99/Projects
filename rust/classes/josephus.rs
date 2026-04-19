//! Josephus Problem — circular elimination and the survivor.
//!
//! N people stand in a circle (0..n). Starting from position 0, count `k`
//! and remove that person; continue from the next remaining position.
//! The last one left is the survivor.
//!
//! Two algorithms, two different queries on the same process:
//! - `josephus` — O(n) recurrence `J(n,k) = (J(n-1,k) + k) % n`, survivor only.
//! - `josephus_simulate` — O(n²) array simulation, returns survivor + full
//!   elimination order.
//!
//! The recurrence is the first closed-form shortcut in Rosetta Stone Classes:
//! a problem whose answer can be computed without materializing the process.

pub fn josephus(n: usize, k: usize) -> usize {
    assert!(n >= 1, "n must be >= 1");
    assert!(k >= 1, "k must be >= 1");
    let mut survivor: usize = 0;
    for m in 2..=n {
        survivor = (survivor + k) % m;
    }
    survivor
}

pub fn josephus_simulate(n: usize, k: usize) -> (usize, Vec<usize>) {
    assert!(n >= 1, "n must be >= 1");
    assert!(k >= 1, "k must be >= 1");
    let mut ring: Vec<usize> = (0..n).collect();
    let mut order: Vec<usize> = Vec::with_capacity(n.saturating_sub(1));
    let mut pos: usize = 0;
    while ring.len() > 1 {
        pos = (pos + k - 1) % ring.len();
        order.push(ring.remove(pos));
    }
    (ring[0], order)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton_survives() {
        assert_eq!(josephus(1, 3), 0);
        let (s, order) = josephus_simulate(1, 3);
        assert_eq!(s, 0);
        assert!(order.is_empty());
    }

    #[test]
    fn k_of_one_eliminates_in_order() {
        // With k=1 every step removes the current position; last person is n-1.
        let (s, order) = josephus_simulate(5, 1);
        assert_eq!(s, 4);
        assert_eq!(order, vec![0, 1, 2, 3]);
        assert_eq!(josephus(5, 1), 4);
    }

    #[test]
    fn classical_josephus_41_3() {
        // Flavius Josephus legend: 41 men, every 3rd killed. Survivor is 30 (0-indexed).
        assert_eq!(josephus(41, 3), 30);
        let (s, order) = josephus_simulate(41, 3);
        assert_eq!(s, 30);
        assert_eq!(order.len(), 40);
        assert!(!order.contains(&30));
    }

    #[test]
    fn small_case_7_3() {
        // Hand-traceable: n=7, k=3. Elimination order from 0: 2,5,1,6,4,0; survivor 3.
        let (s, order) = josephus_simulate(7, 3);
        assert_eq!(s, 3);
        assert_eq!(order, vec![2, 5, 1, 6, 4, 0]);
    }

    #[test]
    fn recurrence_matches_simulation() {
        for n in 1..=40 {
            for k in 1..=6 {
                let (sim, _) = josephus_simulate(n, k);
                assert_eq!(josephus(n, k), sim, "mismatch at n={n}, k={k}");
            }
        }
    }

    #[test]
    fn every_position_eliminated_exactly_once() {
        let (survivor, order) = josephus_simulate(20, 4);
        let mut seen = vec![false; 20];
        for &i in &order {
            assert!(!seen[i], "position {i} eliminated twice");
            seen[i] = true;
        }
        assert!(!seen[survivor], "survivor also appears in elimination order");
        assert_eq!(order.len(), 19);
    }
}
