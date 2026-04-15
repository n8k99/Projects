/-
  Tile cost calculator — total cost to cover a W x H floor.
  Uses Nat for tile counts, Float for cost.
-/

namespace Numbers

structure TileCostResult where
  tilesWide : Nat
  tilesHigh : Nat
  totalTiles : Nat
  totalCost : Float

/-- Calculate tiles needed and total cost to cover a floor. -/
def tileCost (floorWidth floorHeight tileWidth tileHeight costPerTile : Float) : TileCostResult :=
  let tilesWide := (floorWidth / tileWidth).ceil.toUInt64.toNat
  let tilesHigh := (floorHeight / tileHeight).ceil.toUInt64.toNat
  let totalTiles := tilesWide * tilesHigh
  let totalCost := totalTiles.toFloat * costPerTile
  { tilesWide, tilesHigh, totalTiles, totalCost }

end Numbers
