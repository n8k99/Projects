# Mortgage Calculator

This is the first project that maps directly onto a real choreographic need: Kathryn's monthly infrastructure `where`.

## Resolver native

```dpn
@monthly_payment{principal: @p, rate: @r, years: @y} -> @payment
@amortization{principal: @p, rate: @r, years: @y} -> @schedule
@pace_check{target: @t, periods: @n, current: @i, cumulative: @c} -> @pace
```

## Kathryn's `where` — amortized

The monthly aspiration: forex trading covers EM's infrastructure costs. But the `where` isn't evaluated once at month end. It's evaluated every day as a pace check:

```dpn
[daily_finance_where @today
    @pace_check{
        target: @em:monthly_costs,
        periods: @month:trading_days,
        current: @today:day_number,
        cumulative: @month:cumulative_pnl
    } -> @pace
    @KathrynLyonne{evaluate @pace against acceptable variance}
    -> on_track || -> adjust_strategy
] where [trading is on pace to cover infrastructure by month end]
```

The `where` score isn't binary. Day 3 with 5% of the month's target isn't failure — it's information. Day 20 with 40% is a signal. The pace check turns a monthly obligation into a daily gradient.

And this is what `where` looks like when it's amortized across time. Not "did we hit the target" but "are we tracking toward it." The same math that tells you whether your mortgage payments are on schedule tells Kathryn whether the noosphere is funding itself.

## The pattern

Obligation + time + periodic measurement = amortized `where`.

Every `where` in the temporal chain works this way. The daily `where` is a pace check against the weekly aspiration. The weekly against the monthly. The monthly against the quarterly. The quarterly against the yearly. The yearly against the vault's top-level aspiration. Each level amortizes the one above it into periodic measurements.

The mortgage calculator isn't about houses. It's the math underneath every temporal `where` in the noosphere.

## Design note

Six projects in. First direct mapping between a Rosetta Stone project and InnateScript's core design. The `pace_check` function is the implementation of amortized `where` — turning a distant target into a daily score. This is the function Kathryn would call every night, and it's the same function that the temporal compression chain uses to evaluate whether any given period is on track toward its parent period's aspiration.
