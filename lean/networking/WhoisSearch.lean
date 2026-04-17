-- Whois Search Tool — query and parse domain registration data.
-- Pure data model: records, parsing, comparison, and client with cache.

namespace Networking

structure WhoisContact' where
  name : String := ""
  organization : String := ""
  email : String := ""
  city : String := ""
  country : String := ""
  deriving Repr

def WhoisContact'.summary (c : WhoisContact') : String :=
  let parts := [c.name, c.organization, c.city, c.country].filter (!·.isEmpty)
  if parts.isEmpty then "(redacted)" else String.intercalate ", " parts

structure WhoisRecord' where
  domain : String
  registrar : String := ""
  registrant : Option WhoisContact' := none
  nameServers : List String := []
  status : List String := []
  createdDate : String := ""
  updatedDate : String := ""
  expiryDate : String := ""
  dnssec : String := ""
  rawText : String := ""
  deriving Repr

def WhoisRecord'.tld (r : WhoisRecord') : String :=
  match r.domain.splitOn "." |>.getLast? with
  | some t => t
  | none => ""

def WhoisRecord'.summary (r : WhoisRecord') : String :=
  let lines := [s!"Domain: {r.domain}"]
  let lines := if r.registrar.isEmpty then lines else lines ++ [s!"  Registrar: {r.registrar}"]
  let lines := match r.registrant with
    | some c => lines ++ [s!"  Registrant: {c.summary}"]
    | none => lines
  let lines := if r.nameServers.isEmpty then lines
    else lines ++ [s!"  Name Servers: {String.intercalate \", \" r.nameServers}"]
  let lines := if r.createdDate.isEmpty then lines else lines ++ [s!"  Created: {r.createdDate}"]
  let lines := if r.expiryDate.isEmpty then lines else lines ++ [s!"  Expires: {r.expiryDate}"]
  String.intercalate "\n" lines

-- Simple field extraction from raw whois text
private def findField (raw key : String) : String :=
  let keyLower := key.toLower
  let lines := raw.splitOn "\n"
  match lines.find? fun line =>
    let trimmed := line.trim.toLower
    trimmed.startsWith keyLower
  with
  | some line =>
    match line.findIdx (· == ':') with
    | pos => if pos < line.length then (line.drop (pos + 1)).trim else ""
  | none => ""

private def findAllFields (raw key : String) : List String :=
  let keyLower := key.toLower
  raw.splitOn "\n" |>.filterMap fun line =>
    let trimmed := line.trim
    if trimmed.toLower.startsWith keyLower then
      match trimmed.findIdx (· == ':') with
      | pos => if pos < trimmed.length then
          let v := (trimmed.drop (pos + 1)).trim
          if v.isEmpty then none else some v
        else none
    else none

def parseWhoisText' (domain raw : String) : WhoisRecord' :=
  let created := findField raw "Creation Date"
  let created := if created.isEmpty then findField raw "Created Date" else created
  let expiry := findField raw "Registry Expiry Date"
  let expiry := if expiry.isEmpty then findField raw "Expiration Date" else expiry
  let status := findAllFields raw "Domain Status"
  let status := if status.isEmpty then findAllFields raw "Status" else status
  let regName := findField raw "Registrant Name"
  let regOrg := findField raw "Registrant Organization"
  let registrant := if regName.isEmpty && regOrg.isEmpty then none
    else some {
      name := regName
      organization := regOrg
      email := findField raw "Registrant Email"
      country := findField raw "Registrant Country"
      city := findField raw "Registrant City" : WhoisContact' }
  { domain := domain.toLower
    registrar := findField raw "Registrar"
    registrant
    nameServers := (findAllFields raw "Name Server").map String.toLower
    status
    createdDate := created
    updatedDate := findField raw "Updated Date"
    expiryDate := expiry
    dnssec := findField raw "DNSSEC"
    rawText := raw }

-- Client with simulated data and cache
structure WhoisClientState where
  cache : List (String × WhoisRecord') := []
  simulated : List (String × String) := []
  deriving Repr

namespace WhoisClientState

def addSimulated (c : WhoisClientState) (domain raw : String) : WhoisClientState :=
  { c with simulated := c.simulated ++ [(domain.toLower, raw)] }

def query (c : WhoisClientState) (domain : String) : WhoisClientState × WhoisRecord' :=
  let d := domain.toLower
  match c.cache.find? (·.1 == d) with
  | some (_, r) => (c, r)
  | none =>
    let raw := match c.simulated.find? (·.1 == d) with
      | some (_, r) => r | none => ""
    let record := parseWhoisText' d raw
    ({ c with cache := c.cache ++ [(d, record)] }, record)

end WhoisClientState

-- Demo
def whoisDemo : IO Unit := do
  let raw := "Domain Name: example.com\nRegistrar: Test Registrar\nCreation Date: 2020-01-15\nName Server: ns1.example.com\nName Server: ns2.example.com\nRegistrant Name: John Doe\nRegistrant Country: US\n"
  let c := WhoisClientState.addSimulated {} "example.com" raw
  let (_, record) := c.query "example.com"
  IO.println record.summary

end Networking
