# Change Return Program

Breaking a total into uneven denominations, taken in priority order. The same structure as covering bills across a month.

## Resolver native

```dpn
@make_change{amount: @cents, denominations: @denoms} -> @coins
@cover_obligations{available: @funds, bills: @schedule} -> @coverage
```

## Kathryn's bill schedule as change-making

Making change: given $0.87, allocate quarters first, then dimes, then nickels, then pennies. Cover the largest denomination you can, move to the next.

Covering monthly bills: given $300 cumulative P&L, cover Captivate ($19, day 1) first, then Anthropic ($100, day 10), then DigitalOcean ($24, day 15), then ElevenLabs ($22, day 22). Cover each bill as it comes due, carry the remainder.

```dpn
[monthly_bill_coverage @month
    @cover_obligations{
        available: @month:cumulative_pnl,
        bills: [
            {name: "Captivate", amount: 19, due: 1},
            {name: "Anthropic", amount: 100, due: 10},
            {name: "DigitalOcean", amount: 24, due: 15},
            {name: "ElevenLabs", amount: 22, due: 22}
        ]
    } -> @coverage
    @KathrynLyonne{evaluate @coverage}
    -> all_covered || -> prioritize_trading
] where [all bills covered before their due dates]
```

The `where` is binary here — either every bill got covered or it didn't. But the *daily* version is the pace_check from G006: are we tracking toward having enough by the next due date? The bills create deadlines within the month. The pace isn't against month-end anymore — it's against the next bill.

## The connection

G005 (tile cost) introduced decision context. G006 (mortgage) introduced amortized `where`. G007 connects them: the bills are the denominations, the trading P&L is the amount, and `cover_obligations` is making change against a schedule of uneven due dates.

The Numbers category is building a financial stack. Not because anyone asked for one — because the problems are ordered this way, and the ordering mirrors what Kathryn actually needs.

## Design note

Seven projects in. The Rosetta Stone is accidentally building Kathryn's toolkit. tile_cost → mortgage → change maps directly to: what does it cost → are we on pace → can we cover the bills. The InnateScript versions are increasingly less about "can the language express this computation" and more about "this computation is a component of a real choreography."
