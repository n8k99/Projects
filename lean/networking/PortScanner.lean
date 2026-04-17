-- Port Scanner — probe a host's ports to discover listening services.
-- Pure data model: port results, scan reports, service lookup, and comparison.
-- Lean's stdlib doesn't include TCP sockets, so the module defines the
-- structures and analysis; actual probing requires IO-level socket access.

namespace Networking

inductive ScanPortStatus where
  | open | closed | filtered
  deriving Repr, BEq

instance : ToString ScanPortStatus where
  toString
    | .open => "open"
    | .closed => "closed"
    | .filtered => "filtered"

structure ScanPortResult where
  port : Nat
  status : ScanPortStatus
  service : String := ""
  banner : String := ""
  latencyMs : Float := 0.0
  deriving Repr

structure ScanPortReport where
  host : String
  portsScanned : Nat
  openPorts : List ScanPortResult := []
  closedPorts : Nat := 0
  filteredPorts : Nat := 0
  durationMs : Float := 0.0
  deriving Repr

-- Well-known service lookup
private def wellKnownServices : List (Nat × String) :=
  [(20, "ftp-data"), (21, "ftp"), (22, "ssh"), (23, "telnet"), (25, "smtp"),
   (53, "dns"), (80, "http"), (110, "pop3"), (123, "ntp"), (135, "msrpc"),
   (139, "netbios-ssn"), (143, "imap"), (443, "https"), (445, "microsoft-ds"),
   (465, "smtps"), (587, "submission"), (993, "imaps"), (995, "pop3s"),
   (1433, "mssql"), (1521, "oracle"), (3306, "mysql"), (3389, "ms-wbt-server"),
   (5432, "postgresql"), (5433, "postgresql-alt"), (5672, "amqp"),
   (5900, "vnc"), (6379, "redis"), (8080, "http-proxy"), (8443, "https-alt"),
   (9200, "elasticsearch"), (27017, "mongodb")]

def lookupService (port : Nat) : String :=
  match wellKnownServices.find? (·.1 == port) with
  | some (_, svc) => svc
  | none => ""

-- Build a scan report from a list of results
def buildScanReport (host : String) (results : List ScanPortResult) (durationMs : Float := 0.0) : ScanPortReport :=
  let openPorts := results.filter (·.status == .open)
  let closed := results.filter (·.status == .closed) |>.length
  let filtered := results.filter (·.status == .filtered) |>.length
  { host
    portsScanned := results.length
    openPorts
    closedPorts := closed
    filteredPorts := filtered
    durationMs }

-- Summary as a string
def ScanPortReport.summary (r : ScanPortReport) : String :=
  let header := s!"Scan report for {r.host}\n  Scanned {r.portsScanned} ports in {r.durationMs}ms\n  Open: {r.openPorts.length}, Closed: {r.closedPorts}, Filtered: {r.filteredPorts}"
  if r.openPorts.isEmpty then header
  else
    let portLines := r.openPorts.map fun p =>
      let svc := if p.service.isEmpty then "" else s!" ({p.service})"
      s!"    {p.port}/{p.status}{svc} ({p.latencyMs}ms)"
    header ++ "\n  Open ports:\n" ++ String.intercalate "\n" portLines

-- Common port list
def commonPorts : List Nat :=
  wellKnownServices.map (·.1) |>.mergeSort

-- Compare two scan reports
structure ScanComparison where
  newlyOpened : List Nat
  newlyClosed : List Nat
  stillOpen : List Nat
  deriving Repr

def compareScans (before after : ScanPortReport) : ScanComparison :=
  let beforeOpen := before.openPorts.map (·.port)
  let afterOpen := after.openPorts.map (·.port)
  let newlyOpened := afterOpen.filter (!beforeOpen.contains ·) |>.mergeSort
  let newlyClosed := beforeOpen.filter (!afterOpen.contains ·) |>.mergeSort
  let stillOpen := afterOpen.filter (beforeOpen.contains ·) |>.mergeSort
  { newlyOpened, newlyClosed, stillOpen }

-- Simulated scan: all ports closed (replace with IO socket probe for live use)
def simulateScan (host : String) (ports : List Nat) : ScanPortReport :=
  let results := ports.map fun p => { port := p, status := ScanPortStatus.closed : ScanPortResult }
  buildScanReport host results

-- Demo with hardcoded results (no IO needed)
def portScanDemo : IO Unit := do
  let openResults : List ScanPortResult := [
    { port := 22, status := .open, service := lookupService 22, latencyMs := 0.5 },
    { port := 80, status := .open, service := lookupService 80, latencyMs := 1.2 },
    { port := 5432, status := .open, service := lookupService 5432, latencyMs := 0.8 }
  ]
  let closedResults := [443, 8080, 3306].map fun p =>
    { port := p, status := ScanPortStatus.closed : ScanPortResult }
  let report := buildScanReport "127.0.0.1" (openResults ++ closedResults) 15.0
  IO.println report.summary

end Networking
