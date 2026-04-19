// Josephus Problem — circular elimination and the survivor.
//
// N people stand in a circle (0..n). Starting at position 0, count k and
// remove that person; continue from the next remaining position. The last
// one standing is the survivor.
//
// Two algorithms, two different queries on the same process:
//
//   Josephus         O(n) recurrence J(n,k) = (J(n-1,k) + k) % n, survivor only.
//   JosephusSimulate O(n²) slice simulation, returns survivor + elimination order.
//
// The recurrence is the first closed-form shortcut in Rosetta Stone Classes:
// a problem whose answer can be computed without materialising the process.
package classes

import "fmt"

// Josephus returns the 0-indexed survivor of the problem using the
// O(n) recurrence. Panics on n < 1 or k < 1.
func Josephus(n, k int) int {
	if n < 1 {
		panic(fmt.Sprintf("josephus: n must be >= 1, got %d", n))
	}
	if k < 1 {
		panic(fmt.Sprintf("josephus: k must be >= 1, got %d", k))
	}
	survivor := 0
	for m := 2; m <= n; m++ {
		survivor = (survivor + k) % m
	}
	return survivor
}

// JosephusSimulate runs the full elimination and returns both the
// survivor's 0-indexed position and the order in which positions were
// eliminated.
func JosephusSimulate(n, k int) (int, []int) {
	if n < 1 {
		panic(fmt.Sprintf("josephus: n must be >= 1, got %d", n))
	}
	if k < 1 {
		panic(fmt.Sprintf("josephus: k must be >= 1, got %d", k))
	}
	ring := make([]int, n)
	for i := range ring {
		ring[i] = i
	}
	order := make([]int, 0, n-1)
	pos := 0
	for len(ring) > 1 {
		pos = (pos + k - 1) % len(ring)
		order = append(order, ring[pos])
		ring = append(ring[:pos], ring[pos+1:]...)
	}
	return ring[0], order
}
