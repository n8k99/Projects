package numbers

import "math"

// TileCostResult holds the result of a tile cost calculation.
type TileCostResult struct {
	TilesWide  int
	TilesHigh  int
	TotalTiles int
	TotalCost  float64
}

// TileCost calculates tiles needed and total cost to cover a floor.
func TileCost(floorWidth, floorHeight, tileWidth, tileHeight, costPerTile float64) TileCostResult {
	tilesWide := int(math.Ceil(floorWidth / tileWidth))
	tilesHigh := int(math.Ceil(floorHeight / tileHeight))
	totalTiles := tilesWide * tilesHigh
	totalCost := float64(totalTiles) * costPerTile

	return TileCostResult{
		TilesWide:  tilesWide,
		TilesHigh:  tilesHigh,
		TotalTiles: totalTiles,
		TotalCost:  totalCost,
	}
}
