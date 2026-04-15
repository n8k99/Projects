"""Change return program — break an amount into denominations."""


def make_change(amount: float, denominations: list[float] = None) -> dict:
    """Break amount into fewest coins/bills possible.

    denominations: descending list of values. Defaults to US coins.
    Returns dict of {denomination: count}.
    """
    if denominations is None:
        denominations = [0.25, 0.10, 0.05, 0.01]

    remaining = round(amount * 100) if all(d < 1 for d in denominations) else amount
    scale = 100 if all(d < 1 for d in denominations) else 1

    result = {}
    for denom in sorted(denominations, reverse=True):
        d_scaled = round(denom * scale)
        count = int(remaining // d_scaled)
        if count > 0:
            result[denom] = count
            remaining -= count * d_scaled

    return result


def cover_obligations(available: float, obligations: list[dict]) -> dict:
    """Given available funds, cover obligations in chronological order.

    obligations: list of {name, amount, due_day} sorted by due_day.
    Returns {covered: [...], uncovered: [...], remaining: float}.

    Same structure as making change — break a total into uneven
    denominations taken in priority order. Bills across a month
    are coins of different sizes.
    """
    obligations = sorted(obligations, key=lambda o: o["due_day"])
    covered = []
    uncovered = []
    remaining = available

    for ob in obligations:
        if remaining >= ob["amount"]:
            remaining = round(remaining - ob["amount"], 2)
            covered.append({**ob, "status": "covered"})
        else:
            uncovered.append({**ob, "shortfall": round(ob["amount"] - remaining, 2)})

    return {
        "covered": covered,
        "uncovered": uncovered,
        "remaining": round(remaining, 2),
        "all_covered": len(uncovered) == 0,
    }
