# Tile Cost Calculator

Pure computation — but this is the first project with *domain context*. A floor. Tiles. Cost. Someone is making a decision based on this number.

## Resolver native

```dpn
@tile_cost{floor: [@w, @h], tile: [@tw, @th], cost: @price} -> @result
```

## Where domain context appears

This is arithmetic anyone can do. The interesting thing isn't the calculation — it's that someone *needs* it. A tile cost calculator exists because a person is standing in a hardware store making a purchasing decision. The computation serves a context.

In InnateScript, that context is the choreography:

```dpn
[renovation_budget @room
    @tile_cost{floor: @room:dimensions, tile: @selected_tile:size, cost: @selected_tile:price} -> @flooring
    @tile_cost{floor: @room:dimensions, tile: @backsplash:size, cost: @backsplash:price} -> @walls
    @calculator{@flooring:total_cost + @walls:total_cost + @labor} -> @total
    <- @KathrynLyonne{verify @total against @room:budget}
] where [total is within budget or the scope is reduced to fit]
```

The `where` isn't "did the math work." The math always works. The `where` is "can we afford this." That's judgment. That's Kathryn. The calculator doesn't know about budgets. The choreography does.

## Design note

Five projects in. First encounter with a computation that exists because of a *human decision context*. The calculation is trivial. The coordination around it — budget verification, scope reduction, purchasing decisions — is where agents earn their keep. InnateScript doesn't make math easier. It makes the conversation around math structured.
