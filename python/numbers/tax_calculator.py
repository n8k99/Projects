"""Tax Calculator — progressive income tax computation."""


# 2024 US federal single-filer brackets: (upper_limit, rate)
# Last entry uses float('inf') for the unlimited top bracket.
BRACKETS = [
    (11_600, 0.10),
    (47_150, 0.12),
    (100_525, 0.22),
    (191_950, 0.24),
    (243_725, 0.32),
    (609_350, 0.35),
    (float("inf"), 0.37),
]


def calculate_tax(income: float) -> dict:
    """Calculate progressive income tax for a given income.

    Returns a dict with:
        total_tax: total tax owed
        effective_rate: total_tax / income (0.0 if income <= 0)
        marginal_rate: rate of the bracket the top dollar falls into
        breakdown: list of dicts with bracket, rate, taxable, tax
    """
    if income <= 0:
        return {
            "total_tax": 0.0,
            "effective_rate": 0.0,
            "marginal_rate": 0.0,
            "breakdown": [],
        }

    breakdown = []
    total_tax = 0.0
    prev_limit = 0.0
    marginal_rate = 0.0

    for limit, rate in BRACKETS:
        if income <= prev_limit:
            break
        taxable = min(income, limit) - prev_limit
        tax = round(taxable * rate, 2)
        total_tax += tax
        marginal_rate = rate
        breakdown.append({
            "bracket": f"${prev_limit:,.0f} - ${limit:,.0f}" if limit != float("inf") else f"${prev_limit:,.0f}+",
            "rate": rate,
            "taxable": taxable,
            "tax": tax,
        })
        prev_limit = limit

    total_tax = round(total_tax, 2)
    effective_rate = round(total_tax / income, 6)

    return {
        "total_tax": total_tax,
        "effective_rate": effective_rate,
        "marginal_rate": marginal_rate,
        "breakdown": breakdown,
    }


if __name__ == "__main__":
    test_incomes = [50_000, 100_000, 250_000, 750_000]

    for income in test_incomes:
        result = calculate_tax(income)
        print(f"\nIncome: ${income:>10,.0f}")
        print(f"  Total tax:      ${result['total_tax']:>10,.2f}")
        print(f"  Effective rate:  {result['effective_rate'] * 100:>6.2f}%")
        print(f"  Marginal rate:   {result['marginal_rate'] * 100:>6.1f}%")
        print("  Breakdown:")
        for b in result["breakdown"]:
            print(f"    {b['bracket']:>25s}  @{b['rate']*100:5.1f}%  "
                  f"${b['taxable']:>10,.0f}  = ${b['tax']:>10,.2f}")
