"""Credit Card Validator — Luhn algorithm and network identification."""


def luhn_check(number: str) -> bool:
    """Validate a credit card number using the Luhn algorithm."""
    digits = [int(d) for d in number if d.isdigit()]
    if len(digits) < 2:
        return False

    total = 0
    for i, d in enumerate(reversed(digits)):
        if i % 2 == 1:
            d *= 2
            if d > 9:
                d -= 9
        total += d
    return total % 10 == 0


def identify_network(number: str) -> str | None:
    """Identify the card network by prefix and length.

    Returns 'Visa', 'Mastercard', 'Amex', 'Discover', or None.
    """
    digits = "".join(c for c in number if c.isdigit())
    n = len(digits)

    # Amex: 34xx or 37xx, length 15
    if n == 15 and digits[:2] in ("34", "37"):
        return "Amex"

    # Visa: starts with 4, length 13 or 16
    if digits[0] == "4" and n in (13, 16):
        return "Visa"

    # Mastercard: 51-55 or 2221-2720, length 16
    if n == 16:
        prefix2 = int(digits[:2])
        prefix4 = int(digits[:4])
        if 51 <= prefix2 <= 55:
            return "Mastercard"
        if 2221 <= prefix4 <= 2720:
            return "Mastercard"

    # Discover: 6011, 644-649, 65xx, length 16
    if n == 16:
        if digits[:4] == "6011":
            return "Discover"
        prefix3 = int(digits[:3])
        if 644 <= prefix3 <= 649:
            return "Discover"
        if digits[:2] == "65":
            return "Discover"

    return None


def validate(number: str) -> dict:
    """Validate a credit card number.

    Returns a dict with:
        valid: bool — passes Luhn check, correct length, known network
        network: str or None — card network name
        display: str — masked number showing last 4 digits
    """
    digits = "".join(c for c in number if c.isdigit())
    last4 = digits[-4:] if len(digits) >= 4 else digits
    display = "****" + last4

    network = identify_network(digits)
    valid = luhn_check(digits) and network is not None

    return {"valid": valid, "network": network, "display": display}


if __name__ == "__main__":
    test_numbers = [
        ("4532015112830366", "Visa"),
        ("5425233430109903", "Mastercard"),
        ("378282246310005", "Amex"),
        ("6011111111111117", "Discover"),
        ("1234567890123456", "invalid"),
    ]
    for number, label in test_numbers:
        result = validate(number)
        status = "VALID" if result["valid"] else "INVALID"
        net = result["network"] or "unknown"
        print(f"{result['display']}  {status:7s}  {net:12s}  (expected {label})")
