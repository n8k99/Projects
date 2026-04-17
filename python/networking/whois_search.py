"""Whois Search Tool — query and parse domain registration data."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional
import re


@dataclass
class WhoisContact:
    """Contact information from a whois record."""
    name: str = ""
    organization: str = ""
    email: str = ""
    phone: str = ""
    street: str = ""
    city: str = ""
    state: str = ""
    postal_code: str = ""
    country: str = ""

    def summary(self) -> str:
        parts = [p for p in [self.name, self.organization, self.city, self.country] if p]
        return ", ".join(parts) if parts else "(redacted)"


@dataclass
class WhoisRecord:
    """Parsed whois record for a domain."""
    domain: str
    registrar: str = ""
    registrant: Optional[WhoisContact] = None
    admin_contact: Optional[WhoisContact] = None
    tech_contact: Optional[WhoisContact] = None
    name_servers: list[str] = field(default_factory=list)
    status: list[str] = field(default_factory=list)
    created_date: Optional[datetime] = None
    updated_date: Optional[datetime] = None
    expiry_date: Optional[datetime] = None
    dnssec: str = ""
    raw_text: str = ""

    @property
    def tld(self) -> str:
        parts = self.domain.rsplit(".", 1)
        return parts[-1] if len(parts) > 1 else ""

    @property
    def age_days(self) -> Optional[int]:
        if self.created_date:
            return (datetime.now(timezone.utc) - self.created_date).days
        return None

    @property
    def days_until_expiry(self) -> Optional[int]:
        if self.expiry_date:
            return (self.expiry_date - datetime.now(timezone.utc)).days
        return None

    @property
    def is_expired(self) -> bool:
        if self.expiry_date:
            return self.expiry_date < datetime.now(timezone.utc)
        return False

    def summary(self) -> str:
        lines = [f"Domain: {self.domain}"]
        if self.registrar:
            lines.append(f"  Registrar: {self.registrar}")
        if self.registrant:
            lines.append(f"  Registrant: {self.registrant.summary()}")
        if self.name_servers:
            lines.append(f"  Name Servers: {', '.join(self.name_servers)}")
        if self.created_date:
            lines.append(f"  Created: {self.created_date.strftime('%Y-%m-%d')}")
        if self.expiry_date:
            days = self.days_until_expiry
            label = f"({days} days)" if days and days > 0 else "(EXPIRED)"
            lines.append(f"  Expires: {self.expiry_date.strftime('%Y-%m-%d')} {label}")
        if self.status:
            lines.append(f"  Status: {', '.join(self.status)}")
        return "\n".join(lines)


def parse_whois_text(domain: str, raw: str) -> WhoisRecord:
    """Parse raw whois text into a structured WhoisRecord."""
    record = WhoisRecord(domain=domain, raw_text=raw)

    def find_field(pattern: str) -> str:
        m = re.search(pattern, raw, re.IGNORECASE | re.MULTILINE)
        return m.group(1).strip() if m else ""

    def find_date(pattern: str) -> Optional[datetime]:
        val = find_field(pattern)
        if not val:
            return None
        for fmt in ("%Y-%m-%dT%H:%M:%SZ", "%Y-%m-%d", "%d-%b-%Y", "%Y-%m-%dT%H:%M:%S%z"):
            try:
                dt = datetime.strptime(val, fmt)
                if dt.tzinfo is None:
                    dt = dt.replace(tzinfo=timezone.utc)
                return dt
            except ValueError:
                continue
        return None

    record.registrar = find_field(r"Registrar:\s*(.+)")
    record.dnssec = find_field(r"DNSSEC:\s*(.+)")

    # Name servers
    ns_matches = re.findall(r"Name Server:\s*(.+)", raw, re.IGNORECASE)
    record.name_servers = [ns.strip().lower() for ns in ns_matches if ns.strip()]

    # Status
    status_matches = re.findall(r"(?:Domain )?Status:\s*(.+)", raw, re.IGNORECASE)
    record.status = [s.strip() for s in status_matches if s.strip()]

    # Dates
    record.created_date = find_date(r"Creat(?:ion|ed) Date:\s*(.+)")
    record.updated_date = find_date(r"Updated Date:\s*(.+)")
    record.expiry_date = find_date(r"(?:Registry Expiry|Expir(?:ation|y)) Date:\s*(.+)")

    # Registrant contact
    registrant = WhoisContact()
    registrant.name = find_field(r"Registrant Name:\s*(.+)")
    registrant.organization = find_field(r"Registrant Organization:\s*(.+)")
    registrant.email = find_field(r"Registrant Email:\s*(.+)")
    registrant.country = find_field(r"Registrant Country:\s*(.+)")
    registrant.city = find_field(r"Registrant City:\s*(.+)")
    registrant.state = find_field(r"Registrant State/Province:\s*(.+)")
    if any([registrant.name, registrant.organization, registrant.email]):
        record.registrant = registrant

    return record


class WhoisClient:
    """Whois client that queries and parses domain registration data.

    The actual whois query requires a TCP connection to port 43 of the
    appropriate whois server. This module provides the parsing layer and
    a simulated database for in-process use.
    """

    def __init__(self) -> None:
        self._cache: dict[str, WhoisRecord] = {}
        self._simulated_db: dict[str, str] = {}

    def add_simulated(self, domain: str, raw_text: str) -> None:
        """Add a simulated whois response for testing."""
        self._simulated_db[domain.lower()] = raw_text

    def query(self, domain: str) -> WhoisRecord:
        """Query whois data for a domain."""
        domain = domain.lower().strip()
        if domain in self._cache:
            return self._cache[domain]
        raw = self._simulated_db.get(domain, "")
        record = parse_whois_text(domain, raw)
        self._cache[domain] = record
        return record

    def bulk_query(self, domains: list[str]) -> list[WhoisRecord]:
        """Query multiple domains."""
        return [self.query(d) for d in domains]

    def check_expiry(self, domains: list[str], warn_days: int = 30) -> list[WhoisRecord]:
        """Return domains expiring within warn_days."""
        results = []
        for d in domains:
            r = self.query(d)
            if r.days_until_expiry is not None and 0 < r.days_until_expiry <= warn_days:
                results.append(r)
        return results

    def registrar_breakdown(self, domains: list[str]) -> dict[str, int]:
        """Count domains per registrar."""
        counts: dict[str, int] = {}
        for d in domains:
            r = self.query(d)
            key = r.registrar or "unknown"
            counts[key] = counts.get(key, 0) + 1
        return counts


def compare_records(before: WhoisRecord, after: WhoisRecord) -> dict[str, tuple]:
    """Compare two whois records for the same domain, returning changed fields."""
    changes: dict[str, tuple] = {}
    if before.registrar != after.registrar:
        changes["registrar"] = (before.registrar, after.registrar)
    if before.name_servers != after.name_servers:
        changes["name_servers"] = (before.name_servers, after.name_servers)
    if before.expiry_date != after.expiry_date:
        changes["expiry_date"] = (before.expiry_date, after.expiry_date)
    if before.status != after.status:
        changes["status"] = (before.status, after.status)
    return changes


# --- demo ---
if __name__ == "__main__":
    client = WhoisClient()

    # Simulate a whois response
    client.add_simulated("eckenrodemuziekopname.com", """
Domain Name: eckenrodemuziekopname.com
Registrar: Namecheap, Inc.
Creation Date: 2023-06-15T00:00:00Z
Updated Date: 2025-06-15T00:00:00Z
Registry Expiry Date: 2027-06-15T00:00:00Z
Registrant Name: Nathan Eckenrode
Registrant Organization: Eckenrode Muziekopname
Registrant Country: US
Registrant City: Jacksonville
Registrant State/Province: Florida
Name Server: ns1.digitalocean.com
Name Server: ns2.digitalocean.com
Name Server: ns3.digitalocean.com
Domain Status: clientTransferProhibited
DNSSEC: unsigned
""")

    record = client.query("eckenrodemuziekopname.com")
    print(record.summary())
