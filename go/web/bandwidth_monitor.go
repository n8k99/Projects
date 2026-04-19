// Bandwidth Monitor — time-series rate calculation over samples.
package web

// BWSample is one (timestamp, total bytes) observation.
type BWSample struct {
	TimestampMs int64
	BytesTotal  int64
}

// BWMonitor holds a ring buffer of samples and computes rates over windows.
type BWMonitor struct {
	samples  []BWSample
	capacity int
}

// NewBWMonitor returns a monitor with the given ring capacity. Panics if < 2.
func NewBWMonitor(capacity int) *BWMonitor {
	if capacity < 2 {
		panic("need at least 2 samples to compute a rate")
	}
	return &BWMonitor{capacity: capacity}
}

// Record appends a sample; evicts oldest if over capacity.
func (m *BWMonitor) Record(ts int64, bytesTotal int64) {
	if len(m.samples) == m.capacity {
		m.samples = m.samples[1:]
	}
	m.samples = append(m.samples, BWSample{TimestampMs: ts, BytesTotal: bytesTotal})
}

// SampleCount returns the number of stored samples.
func (m *BWMonitor) SampleCount() int { return len(m.samples) }

// Latest returns the most recent sample, or nil if none.
func (m *BWMonitor) Latest() *BWSample {
	if len(m.samples) == 0 {
		return nil
	}
	s := m.samples[len(m.samples)-1]
	return &s
}

// TotalBytes sums deltas across sample pairs, skipping wraparound intervals.
func (m *BWMonitor) TotalBytes() int64 {
	var total int64
	var prev *BWSample
	for i := range m.samples {
		s := m.samples[i]
		if prev != nil && s.BytesTotal >= prev.BytesTotal {
			total += s.BytesTotal - prev.BytesTotal
		}
		p := m.samples[i]
		prev = &p
	}
	return total
}

// InstantaneousRate returns bytes/sec between the last two samples.
func (m *BWMonitor) InstantaneousRate() float64 {
	n := len(m.samples)
	if n < 2 {
		return 0
	}
	a := m.samples[n-2]
	b := m.samples[n-1]
	if b.TimestampMs <= a.TimestampMs || b.BytesTotal < a.BytesTotal {
		return 0
	}
	dtS := float64(b.TimestampMs-a.TimestampMs) / 1000.0
	return float64(b.BytesTotal-a.BytesTotal) / dtS
}

// AverageRate returns bytes/sec over the last windowMs milliseconds.
func (m *BWMonitor) AverageRate(windowMs int64) float64 {
	if len(m.samples) == 0 {
		return 0
	}
	latest := m.samples[len(m.samples)-1]
	minTs := latest.TimestampMs - windowMs
	if minTs < 0 {
		minTs = 0
	}
	var earliest *BWSample
	for i := len(m.samples) - 1; i >= 0; i-- {
		if m.samples[i].TimestampMs >= minTs {
			e := m.samples[i]
			earliest = &e
		} else {
			break
		}
	}
	if earliest == nil || earliest.TimestampMs == latest.TimestampMs {
		return 0
	}
	if latest.BytesTotal < earliest.BytesTotal {
		return 0
	}
	dtS := float64(latest.TimestampMs-earliest.TimestampMs) / 1000.0
	return float64(latest.BytesTotal-earliest.BytesTotal) / dtS
}

// PeakRate returns the highest rate between any two consecutive samples
// starting within the window.
func (m *BWMonitor) PeakRate(windowMs int64) float64 {
	if len(m.samples) == 0 {
		return 0
	}
	latest := m.samples[len(m.samples)-1]
	minTs := latest.TimestampMs - windowMs
	if minTs < 0 {
		minTs = 0
	}
	var peak float64
	var prev *BWSample
	for i := range m.samples {
		s := m.samples[i]
		if prev != nil &&
			s.TimestampMs >= minTs &&
			s.TimestampMs > prev.TimestampMs &&
			s.BytesTotal >= prev.BytesTotal {
			dtS := float64(s.TimestampMs-prev.TimestampMs) / 1000.0
			rate := float64(s.BytesTotal-prev.BytesTotal) / dtS
			if rate > peak {
				peak = rate
			}
		}
		p := m.samples[i]
		prev = &p
	}
	return peak
}
