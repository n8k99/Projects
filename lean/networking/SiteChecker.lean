-- Site Checker with Time Scheduling — periodic health monitoring of web endpoints.

namespace Networking

inductive SiteHealth where
  | up | down | degraded | unknown
  deriving Repr, BEq

instance : ToString SiteHealth where
  toString | .up => "up" | .down => "down" | .degraded => "degraded" | .unknown => "unknown"

structure MonitorConfig where
  url : String
  name : String := ""
  intervalSecs : Nat := 60
  expectedStatus : Nat := 200
  maxResponseMs : Float := 5000.0
  deriving Repr

structure MonitorCheckResult where
  url : String
  status : SiteHealth
  httpStatus : Nat := 0
  responseTimeMs : Float := 0.0
  error : String := ""
  checkedAt : Nat := 0
  deriving Repr

structure MonitorHistory where
  config : MonitorConfig
  checks : List MonitorCheckResult := []
  consecutiveFailures : Nat := 0
  deriving Repr

namespace MonitorHistory

def currentStatus (h : MonitorHistory) : SiteHealth :=
  match h.checks.getLast? with
  | some c => c.status
  | none => .unknown

def uptimePct (h : MonitorHistory) : Float :=
  if h.checks.isEmpty then 0.0
  else
    let up := h.checks.filter (·.status == .up) |>.length
    up.toFloat / h.checks.length.toFloat * 100.0

def avgResponseMs (h : MonitorHistory) : Float :=
  let times := h.checks.filter (·.status == .up) |>.map (·.responseTimeMs)
  if times.isEmpty then 0.0
  else times.foldl (· + ·) 0.0 / times.length.toFloat

def addCheck (h : MonitorHistory) (r : MonitorCheckResult) : MonitorHistory × Option String :=
  let prev := h.currentStatus
  let failures := if r.status == .down || r.status == .degraded
    then h.consecutiveFailures + 1 else 0
  let h' := { h with checks := h.checks ++ [r], consecutiveFailures := failures }
  let alert := if prev != r.status && prev != .unknown
    then some s!"{h.config.name}: {prev} -> {r.status}"
    else none
  (h', alert)

def summary (h : MonitorHistory) : String :=
  s!"{h.config.name}: {h.currentStatus} | uptime {h.uptimePct}% | avg {h.avgResponseMs}ms | {h.checks.length} checks | {h.consecutiveFailures} failures"

end MonitorHistory

-- Monitor manages multiple sites
structure SiteMonitorState where
  histories : List (String × MonitorHistory) := []
  deriving Repr

namespace SiteMonitorState

def addSite (m : SiteMonitorState) (config : MonitorConfig) : SiteMonitorState :=
  let name := if config.name.isEmpty then config.url else config.name
  let config := { config with name }
  { m with histories := m.histories ++ [(config.url, { config })] }

def recordCheck (m : SiteMonitorState) (r : MonitorCheckResult) : SiteMonitorState × Option String :=
  let mut alert := none
  let histories := m.histories.map fun (url, h) =>
    if url == r.url then
      let (h', a) := h.addCheck r
      alert := a
      (url, h')
    else (url, h)
  ({ m with histories }, alert)

def sitesByStatus (m : SiteMonitorState) (status : SiteHealth) : List String :=
  m.histories.filter (·.2.currentStatus == status) |>.map (·.1)

def dashboard (m : SiteMonitorState) : String :=
  let header := "Site Monitor Dashboard\n" ++ String.mk (List.replicate 60 '=')
  let lines := m.histories.map (·.2.summary)
  header ++ "\n" ++ String.intercalate "\n" lines

end SiteMonitorState

def siteCheckerDemo : IO Unit := do
  let m := SiteMonitorState.addSite {} { url := "https://example.com", name := "Example" }
  let (m, _) := m.recordCheck { url := "https://example.com", status := .up, httpStatus := 200, responseTimeMs := 150.0 }
  let (m, _) := m.recordCheck { url := "https://example.com", status := .up, httpStatus := 200, responseTimeMs := 120.0 }
  let (m, alert) := m.recordCheck { url := "https://example.com", status := .down, error := "timeout" }
  IO.println m.dashboard
  match alert with
  | some a => IO.println s!"ALERT: {a}"
  | none => pure ()

end Networking
