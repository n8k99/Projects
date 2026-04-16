"""CD Key Generator -- generate and validate software license keys."""

import random
import string

CHARSET = string.ascii_uppercase + string.digits  # 36 chars
GROUP_LEN = 5
DATA_GROUPS = 4
PRIME = 65521  # largest prime < 2^16


def _compute_checksum(groups: list[str]) -> str:
    """Compute a 5-char checksum from the first 4 groups."""
    raw = "".join(groups).encode("ascii")
    h = sum(b * (i + 1) for i, b in enumerate(raw)) % PRIME
    chars = []
    for _ in range(GROUP_LEN):
        chars.append(CHARSET[h % len(CHARSET)])
        h //= len(CHARSET)
    return "".join(chars)


def generate_key() -> str:
    """Generate a single CD key in XXXXX-XXXXX-XXXXX-XXXXX-XXXXX format."""
    groups = [
        "".join(random.choices(CHARSET, k=GROUP_LEN))
        for _ in range(DATA_GROUPS)
    ]
    checksum = _compute_checksum(groups)
    groups.append(checksum)
    return "-".join(groups)


def validate_key(key: str) -> bool:
    """Validate a CD key by recomputing its checksum."""
    parts = key.strip().upper().split("-")
    if len(parts) != DATA_GROUPS + 1:
        return False
    if any(len(p) != GROUP_LEN for p in parts):
        return False
    if any(c not in CHARSET for p in parts for c in p):
        return False
    expected = _compute_checksum(parts[:DATA_GROUPS])
    return parts[DATA_GROUPS] == expected


def generate_batch(n: int) -> list[str]:
    """Generate a batch of n CD keys."""
    return [generate_key() for _ in range(n)]


if __name__ == "__main__":
    print("=== CD Key Generator ===\n")

    keys = generate_batch(5)
    for key in keys:
        valid = validate_key(key)
        print(f"  {key}  valid={valid}")

    print()
    tampered = keys[0][:4] + "Z" + keys[0][5:]
    print(f"  Tampered: {tampered}  valid={validate_key(tampered)}")
