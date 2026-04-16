// G034 — Get Atomic Time from Internet Clock
// Time synchronization protocol: in-process NTP-style client/server.
package networking

import "time"

// TimeServer provides authoritative time. Wraps system clock with optional offset.
type TimeServer struct {
	offsetSeconds float64
}

// NewTimeServer creates a server using the system clock.
func NewTimeServer() *TimeServer {
	return &TimeServer{}
}

// NewTimeServerWithOffset creates a server shifted by offset seconds.
func NewTimeServerWithOffset(offset float64) *TimeServer {
	return &TimeServer{offsetSeconds: offset}
}

// GetTime returns the authoritative timestamp as epoch seconds.
func (s *TimeServer) GetTime() float64 {
	return float64(time.Now().UnixNano())/1e9 + s.offsetSeconds
}

// TimeClient queries a TimeServer and tracks synchronization state.
type TimeClient struct {
	server      *TimeServer
	localOffset float64
	syncOffset  float64
	synced      bool
	samples     []float64
}

// NewTimeClient creates a client with no drift.
func NewTimeClient(server *TimeServer) *TimeClient {
	return &TimeClient{server: server}
}

// NewTimeClientWithDrift creates a client whose local clock is off by drift seconds.
func NewTimeClientWithDrift(server *TimeServer, drift float64) *TimeClient {
	return &TimeClient{server: server, localOffset: drift}
}

func systemTime() float64 {
	return float64(time.Now().UnixNano()) / 1e9
}

// LocalTime returns the client's uncorrected local time.
func (c *TimeClient) LocalTime() float64 {
	return systemTime() + c.localOffset
}

// QueryTime queries the server and returns its authoritative timestamp.
func (c *TimeClient) QueryTime() float64 {
	tSend := c.LocalTime()
	serverTime := c.server.GetTime()
	tRecv := c.LocalTime()
	roundTrip := tRecv - tSend
	estimatedServerNow := serverTime + roundTrip/2.0
	c.samples = append(c.samples, estimatedServerNow-tRecv)
	return serverTime
}

// GetOffset returns measured offset between local and server clock.
// Positive means local is behind server.
func (c *TimeClient) GetOffset() float64 {
	if len(c.samples) == 0 {
		c.QueryTime()
	}
	sum := 0.0
	for _, s := range c.samples {
		sum += s
	}
	return sum / float64(len(c.samples))
}

// Sync adjusts local time estimate by applying the measured offset.
func (c *TimeClient) Sync() {
	c.syncOffset = c.GetOffset()
	c.synced = true
}

// GetSyncedTime returns local time corrected by the synchronization offset.
func (c *TimeClient) GetSyncedTime() float64 {
	if !c.synced {
		return c.LocalTime()
	}
	return c.LocalTime() + c.syncOffset
}

// IsSynced returns whether the client has been synchronized.
func (c *TimeClient) IsSynced() bool {
	return c.synced
}
