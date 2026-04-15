"""Mortgage calculator — monthly payments, amortization schedule, payoff tracking."""

from dataclasses import dataclass


@dataclass
class Payment:
    period: int
    payment: float
    principal: float
    interest: float
    balance: float


def monthly_payment(principal: float, annual_rate: float, years: int) -> float:
    """Calculate fixed monthly payment for a mortgage."""
    if principal <= 0:
        raise ValueError("principal must be positive")
    if years <= 0:
        raise ValueError("years must be positive")
    if annual_rate == 0:
        return round(principal / (years * 12), 2)

    r = annual_rate / 100 / 12
    n = years * 12
    payment = principal * (r * (1 + r)**n) / ((1 + r)**n - 1)
    return round(payment, 2)


def amortization_schedule(principal: float, annual_rate: float, years: int) -> list[Payment]:
    """Generate full amortization schedule."""
    pmt = monthly_payment(principal, annual_rate, years)
    r = annual_rate / 100 / 12
    balance = principal
    schedule = []

    for period in range(1, years * 12 + 1):
        interest = round(balance * r, 2)
        princ = round(pmt - interest, 2)
        balance = round(balance - princ, 2)
        if period == years * 12:
            princ = round(princ + balance, 2)
            balance = 0.0
        schedule.append(Payment(period, pmt, princ, interest, balance))

    return schedule


def total_cost(principal: float, annual_rate: float, years: int) -> dict:
    """Calculate total cost of the mortgage."""
    pmt = monthly_payment(principal, annual_rate, years)
    n = years * 12
    total = round(pmt * n, 2)
    total_interest = round(total - principal, 2)
    return {
        "monthly_payment": pmt,
        "total_payments": n,
        "total_paid": total,
        "total_interest": total_interest,
        "principal": principal,
    }


def pace_check(target: float, periods: int, current_period: int, cumulative: float) -> dict:
    """Check if cumulative payments are on pace to meet target.

    Generic amortization pace check — works for any periodic obligation.
    target: total amount due by end of all periods
    periods: total number of periods
    current_period: which period we're in (1-indexed)
    cumulative: total paid so far
    """
    expected = target * current_period / periods
    delta = round(cumulative - expected, 2)
    pace = round(cumulative / expected * 100, 1) if expected > 0 else 0.0

    return {
        "target": target,
        "expected_by_now": round(expected, 2),
        "actual": cumulative,
        "delta": delta,
        "pace_percent": pace,
        "on_track": delta >= 0,
    }
