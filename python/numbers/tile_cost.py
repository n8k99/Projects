"""Tile cost calculator — total cost to cover a W x H floor."""

import math


def tile_cost(floor_width: float, floor_height: float,
              tile_width: float, tile_height: float,
              cost_per_tile: float) -> dict:
    """Calculate tiles needed and total cost to cover a floor.

    Returns dict with tiles_wide, tiles_high, total_tiles, total_cost.
    """
    if any(v <= 0 for v in (floor_width, floor_height, tile_width, tile_height)):
        raise ValueError("dimensions must be positive")
    if cost_per_tile < 0:
        raise ValueError("cost must be non-negative")

    tiles_wide = math.ceil(floor_width / tile_width)
    tiles_high = math.ceil(floor_height / tile_height)
    total_tiles = tiles_wide * tiles_high
    total_cost = total_tiles * cost_per_tile

    return {
        "tiles_wide": tiles_wide,
        "tiles_high": tiles_high,
        "total_tiles": total_tiles,
        "total_cost": round(total_cost, 2),
    }
