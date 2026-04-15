//! Tile cost calculator — total cost to cover a W x H floor.

/// Result of a tile cost calculation.
pub struct TileCostResult {
    pub tiles_wide: u32,
    pub tiles_high: u32,
    pub total_tiles: u32,
    pub total_cost: f64,
}

/// Calculate tiles needed and total cost to cover a floor.
pub fn tile_cost(
    floor_width: f64, floor_height: f64,
    tile_width: f64, tile_height: f64,
    cost_per_tile: f64,
) -> TileCostResult {
    assert!(floor_width > 0.0 && floor_height > 0.0, "floor dimensions must be positive");
    assert!(tile_width > 0.0 && tile_height > 0.0, "tile dimensions must be positive");

    let tiles_wide = (floor_width / tile_width).ceil() as u32;
    let tiles_high = (floor_height / tile_height).ceil() as u32;
    let total_tiles = tiles_wide * tiles_high;
    let total_cost = (total_tiles as f64) * cost_per_tile;

    TileCostResult { tiles_wide, tiles_high, total_tiles, total_cost }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_cost() {
        let r = tile_cost(10.0, 12.0, 2.0, 2.0, 5.50);
        assert_eq!(r.tiles_wide, 5);
        assert_eq!(r.tiles_high, 6);
        assert_eq!(r.total_tiles, 30);
        assert!((r.total_cost - 165.0).abs() < 0.01);
    }
}
