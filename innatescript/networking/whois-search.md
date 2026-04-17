# G042 — Whois Search Tool

> Query and parse domain registration data.

```yaml
id: G042
title: Whois Search Tool
category: networking
requires: [G032-regex-query, G038-port-scanner, G041-ip-lookup]
provides: [ownership-resolution, domain-provenance, registration-lifecycle, change-monitoring]
```

## Insight: Ownership — Who Controls This Name?

G041 (IP Lookup) answered "where is this address?" Whois answers "who owns this name?" The domain name is the identity. The whois record is the provenance: who registered it, when, through whom, and when it expires. This is the first time the Rosetta Stone queries for **ownership** of a network identity.

In the noosphere, every `@reference` has an owner. `@kathryn` is owned by the executive team. `@finance_positions` is owned by Kathryn. The Ghost Registry is the whois database of the noosphere — it maps names to owners, roles, and capabilities. A whois query on `@finance_positions` returns: registrant is Kathryn Lyonne, registered 2023-01-01, status active, capabilities include `@cover_obligations` and `@pace_check`.

## Insight: Registration Is Temporal — Names Expire

Domains have creation dates, update dates, and expiry dates. A domain that exists today might expire tomorrow. The name is not permanent — it's a renewable lease. This connects to G037's peer staleness and G026's TTL: identities in the network are time-bounded.

In InnateScript, agent assignments could have temporal scope. Kathryn's role as CFO isn't indefinite — it was assigned at a point in time and could be reassigned. The `check_expiry` function — which warns about domains expiring within N days — maps to proactive monitoring of agent assignments: whose role expires soon? Which capability delegation needs renewal?

## Insight: Text Parsing Is Domain-Specific Regex (G032 Returns)

Whois responses are semi-structured text. Each registrar formats slightly differently. The parser uses field-matching patterns: `Registrar: (.+)`, `Name Server: (.+)`, `Creation Date: (.+)`. This is G032's regex toolkit applied to a real-world protocol. The whois parser IS a regex query tool specialized for domain registration data.

The parsing challenge: the format is not standardized. Different TLDs, different registrars, different field names for the same data. The parser must be flexible — multiple patterns for the same field (`Creation Date` vs `Created Date` vs `Creat Date`). This is the resolver's challenge with natural language: the same concept expressed in multiple surface forms. Normalization before extraction.

## Insight: Record Comparison Is Domain-Level Change Detection

`compare_records(before, after)` is G038's `compare_scans` applied to registration data. Did the registrar change? (Possible domain transfer.) Did the name servers change? (Possible DNS hijack.) Did the expiry date change? (Renewal or lapse.) Each change type has different security implications.

The temporal diff pattern is now fully general: scan comparison (ports), inbox delta (mail), and now registration diff (whois). The shape is always the same: snapshot A, snapshot B, report the delta. The domain determines what the delta *means*.

## Choreographic Case: Domain Health Monitoring

```innate
(@domain-health){
  @domains <- ["eckenrodemuziekopname.com", "dragonpunk.wiki"]
  concurrent {
    @records <- @map{over: @domains, apply: @whois/query}
  }
  join {
    @expiring <- @filter{@records, where: days_until_expiry < 30}
    @changed <- @map{over: @domains, apply: @whois/compare{with: @last_records}}
  }
  where {
    no_unexpected_changes: @changed.all{|c| c.changes.empty?}
    no_imminent_expiry: @expiring.empty?
    all_nameservers_digitalocean: @records.all{|r| r.name-servers.all{contains: "digitalocean"}}
  }
}
```

## Structures

```innate
(defstruct whois-contact
  name         : String
  organization : String
  email        : String
  city         : String
  country      : String)

(defstruct whois-record
  domain       : String
  registrar    : String
  registrant   : WhoisContact?
  name-servers : [String]
  status       : [String]
  created-date : String
  updated-date : String
  expiry-date  : String
  dnssec       : String)
```

## Resolver Natives

```innate
@whois{domain: String}                                   -> WhoisRecord
@whois/bulk{domains: [String]}                           -> [WhoisRecord]
@whois/compare{before: WhoisRecord, after: WhoisRecord}  -> WhoisDiff
@whois/expiry-check{domains: [String], warn-days: Nat}   -> [WhoisRecord]
@whois/registrar-breakdown{domains: [String]}            -> {String -> Nat}
```

## Demo

```innate
(@demo){
  @record <- @whois{domain: "eckenrodemuziekopname.com"}
  @print{@record.summary}
  ;; => Domain: eckenrodemuziekopname.com
  ;;      Registrar: Namecheap, Inc.
  ;;      Registrant: Nathan Eckenrode, Eckenrode Muziekopname, Jacksonville, US
  ;;      Name Servers: ns1.digitalocean.com, ns2.digitalocean.com, ns3.digitalocean.com
  ;;      Created: 2023-06-15
  ;;      Expires: 2027-06-15
}
```
