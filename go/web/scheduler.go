// Scheduled Auto Login and Action — cron-like task scheduler.
package web

import "math"

const schedDayMs = 86_400_000

// SchedKind classifies a schedule.
type SchedKind int

const (
	SchedEveryMs SchedKind = iota
	SchedAtMs
	SchedDaily
)

// SchedSchedule describes when a task fires.
type SchedSchedule struct {
	Kind       SchedKind
	IntervalMs int64
	AtMs       int64
	Hour       int
	Minute     int
}

// SchedEveryMs builds an every-N-ms schedule.
func SchedEveryMsNew(ms int64) SchedSchedule {
	return SchedSchedule{Kind: SchedEveryMs, IntervalMs: ms}
}

// SchedAtMsNew builds a one-shot schedule firing at absolute time t.
func SchedAtMsNew(t int64) SchedSchedule {
	return SchedSchedule{Kind: SchedAtMs, AtMs: t}
}

// SchedDailyNew builds a daily schedule at the given wall-clock time.
func SchedDailyNew(hour, minute int) SchedSchedule {
	return SchedSchedule{Kind: SchedDaily, Hour: hour, Minute: minute}
}

// SchedTask is a scheduled task.
type SchedTask struct {
	ID             int64
	Name           string
	Schedule       SchedSchedule
	Action         string
	CredentialRef  string
	NextFireMs     int64
	Enabled        bool
}

// SchedRunStatus is the outcome of a dispatched task.
type SchedRunStatus int

const (
	SchedSucceeded SchedRunStatus = iota
	SchedFailed
)

// SchedRun is one recorded execution.
type SchedRun struct {
	TaskID     int64
	StartedMs  int64
	FinishedMs int64
	Status     SchedRunStatus
	Message    string
}

// SchedDispatchContext carries task + clock into the dispatch function.
type SchedDispatchContext struct {
	Task  *SchedTask
	NowMs int64
}

// SchedDispatch is the pluggable task executor. Returns (succeeded, message).
type SchedDispatch func(SchedDispatchContext) (bool, string)

// SchedScheduler holds tasks and run history.
type SchedScheduler struct {
	tasks   []*SchedTask
	history []SchedRun
	nextID  int64
}

// NewSchedScheduler returns an empty scheduler.
func NewSchedScheduler() *SchedScheduler { return &SchedScheduler{nextID: 1} }

// AddTask schedules a new task; returns its id.
func (s *SchedScheduler) AddTask(name string, sched SchedSchedule,
	action, credentialRef string, firstFireBaseMs int64) int64 {
	id := s.nextID
	s.nextID++
	s.tasks = append(s.tasks, &SchedTask{
		ID: id, Name: name, Schedule: sched, Action: action,
		CredentialRef: credentialRef,
		NextFireMs:    schedComputeInitial(sched, firstFireBaseMs),
		Enabled:       true,
	})
	return id
}

// RemoveTask deletes a task; returns whether it was present.
func (s *SchedScheduler) RemoveTask(id int64) bool {
	for i, t := range s.tasks {
		if t.ID == id {
			s.tasks = append(s.tasks[:i], s.tasks[i+1:]...)
			return true
		}
	}
	return false
}

// SetEnabled toggles a task's enabled flag.
func (s *SchedScheduler) SetEnabled(id int64, enabled bool) bool {
	for _, t := range s.tasks {
		if t.ID == id {
			t.Enabled = enabled
			return true
		}
	}
	return false
}

// Tasks returns a view of all tasks.
func (s *SchedScheduler) Tasks() []*SchedTask { return s.tasks }

// GetTask returns the task with the given id, or nil.
func (s *SchedScheduler) GetTask(id int64) *SchedTask {
	for _, t := range s.tasks {
		if t.ID == id {
			return t
		}
	}
	return nil
}

// History returns all recorded runs.
func (s *SchedScheduler) History() []SchedRun { return s.history }

// HistoryFor returns runs filtered by task id.
func (s *SchedScheduler) HistoryFor(id int64) []SchedRun {
	var out []SchedRun
	for _, r := range s.history {
		if r.TaskID == id {
			out = append(out, r)
		}
	}
	return out
}

// NextFires returns task_id → next_fire_ms.
func (s *SchedScheduler) NextFires() map[int64]int64 {
	out := map[int64]int64{}
	for _, t := range s.tasks {
		out[t.ID] = t.NextFireMs
	}
	return out
}

// Tick fires every task whose next_fire_ms <= now_ms.
func (s *SchedScheduler) Tick(nowMs int64, dispatch SchedDispatch) []SchedRun {
	var newRuns []SchedRun
	for _, task := range s.tasks {
		if !task.Enabled || task.NextFireMs > nowMs {
			continue
		}
		ok, msg := dispatch(SchedDispatchContext{Task: task, NowMs: nowMs})
		status := SchedSucceeded
		if !ok {
			status = SchedFailed
		}
		run := SchedRun{
			TaskID: task.ID, StartedMs: nowMs, FinishedMs: nowMs,
			Status: status, Message: msg,
		}
		s.history = append(s.history, run)
		newRuns = append(newRuns, run)
		next, disable := schedScheduleNext(task.Schedule, task.NextFireMs, nowMs)
		task.NextFireMs = next
		if disable {
			task.Enabled = false
		}
	}
	return newRuns
}

func schedNextDaily(baseMs int64, hour, minute int) int64 {
	day := baseMs / schedDayMs
	dayStart := day * schedDayMs
	target := int64(hour)*3_600_000 + int64(minute)*60_000
	candidate := dayStart + target
	if candidate > baseMs {
		return candidate
	}
	return candidate + schedDayMs
}

func schedComputeInitial(sched SchedSchedule, baseMs int64) int64 {
	switch sched.Kind {
	case SchedEveryMs:
		return baseMs + sched.IntervalMs
	case SchedAtMs:
		return sched.AtMs
	case SchedDaily:
		return schedNextDaily(baseMs, sched.Hour, sched.Minute)
	}
	return 0
}

func schedScheduleNext(sched SchedSchedule, prev, now int64) (int64, bool) {
	switch sched.Kind {
	case SchedEveryMs:
		nxt := prev + sched.IntervalMs
		for nxt <= now {
			nxt += sched.IntervalMs
		}
		return nxt, false
	case SchedAtMs:
		return math.MaxInt64, true
	case SchedDaily:
		return schedNextDaily(now, sched.Hour, sched.Minute), false
	}
	return math.MaxInt64, true
}
