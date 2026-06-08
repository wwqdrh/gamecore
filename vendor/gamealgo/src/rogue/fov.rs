// fov: 视野/战争迷雾计算
//
// 提供 Raycasting 和 Shadowcasting 两种视野算法
// Shadowcasting 为推荐默认算法，高效精确

use super::dungeon::DungeonMap;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct FovResult {
    pub visible: HashSet<(usize, usize)>,
    pub newly_explored: HashSet<(usize, usize)>,
}

impl FovResult {
    pub fn new() -> Self {
        FovResult {
            visible: HashSet::new(),
            newly_explored: HashSet::new(),
        }
    }

    pub fn with_explored(mut self, previously_explored: &HashSet<(usize, usize)>) -> Self {
        self.newly_explored = self
            .visible
            .difference(previously_explored)
            .copied()
            .collect();
        self
    }
}

pub trait FovAlgorithm {
    fn compute(
        &self,
        map: &DungeonMap,
        origin: (usize, usize),
        radius: usize,
    ) -> FovResult;

    fn compute_with_explored(
        &self,
        map: &DungeonMap,
        origin: (usize, usize),
        radius: usize,
        previously_explored: &HashSet<(usize, usize)>,
    ) -> FovResult {
        let result = self.compute(map, origin, radius);
        result.with_explored(previously_explored)
    }
}

pub struct RaycastingFov {
    pub ray_count: usize,
}

impl Default for RaycastingFov {
    fn default() -> Self {
        RaycastingFov { ray_count: 360 }
    }
}

impl RaycastingFov {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ray_count(ray_count: usize) -> Self {
        RaycastingFov { ray_count }
    }
}

impl FovAlgorithm for RaycastingFov {
    fn compute(
        &self,
        map: &DungeonMap,
        origin: (usize, usize),
        radius: usize,
    ) -> FovResult {
        let mut visible = HashSet::new();
        visible.insert(origin);

        let ox = origin.0 as f64 + 0.5;
        let oy = origin.1 as f64 + 0.5;
        let radius_sq = (radius * radius) as f64;

        for i in 0..self.ray_count {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / self.ray_count as f64;
            let dx = angle.cos();
            let dy = angle.sin();

            let mut x = ox;
            let mut y = oy;

            for _ in 0..radius {
                x += dx;
                y += dy;

                let ix = x as usize;
                let iy = y as usize;

                if !map.in_bounds(ix, iy) {
                    break;
                }

                let dist_sq = (x - ox) * (x - ox) + (y - oy) * (y - oy);
                if dist_sq > radius_sq {
                    break;
                }

                visible.insert((ix, iy));

                if !map.tile(ix, iy).is_transparent() {
                    break;
                }
            }
        }

        FovResult {
            visible,
            newly_explored: HashSet::new(),
        }
    }
}

pub struct ShadowcastingFov;

impl ShadowcastingFov {
    pub fn new() -> Self {
        ShadowcastingFov
    }

    fn cast_light(
        map: &DungeonMap,
        ox: usize,
        oy: usize,
        row: i32,
        mut start_slope: f64,
        end_slope: f64,
        radius: i32,
        xx: i32,
        xy: i32,
        yx: i32,
        yy: i32,
        visible: &mut HashSet<(usize, usize)>,
    ) {
        if start_slope < end_slope {
            return;
        }

        let mut next_start_slope = start_slope;

        for i in row..=radius {
            let mut blocked = false;

            for dx in (-i)..=0 {
                let dy = -i;
                let map_x = ox as i32 + dx * xx + dy * yx;
                let map_y = oy as i32 + dx * xy + dy * yy;

                if map_x < 0 || map_y < 0 || map_x >= map.width as i32 || map_y >= map.height as i32 {
                    continue;
                }

                let l_slope = (dx as f64 - 0.5) / (dy as f64 - 0.5);
                let r_slope = (dx as f64 + 0.5) / (dy as f64 + 0.5);

                if start_slope < r_slope {
                    continue;
                }
                if end_slope > l_slope {
                    break;
                }

                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= radius * radius {
                    visible.insert((map_x as usize, map_y as usize));
                }

                let is_blocked = !map.tile(map_x as usize, map_y as usize).is_transparent();

                if blocked {
                    if is_blocked {
                        next_start_slope = r_slope;
                        continue;
                    } else {
                        blocked = false;
                        start_slope = next_start_slope;
                    }
                } else if is_blocked && i < radius {
                    blocked = true;
                    Self::cast_light(
                        map, ox, oy, i + 1, start_slope, l_slope, radius,
                        xx, xy, yx, yy, visible,
                    );
                    next_start_slope = r_slope;
                }
            }

            if blocked {
                break;
            }
        }
    }
}

impl FovAlgorithm for ShadowcastingFov {
    fn compute(
        &self,
        map: &DungeonMap,
        origin: (usize, usize),
        radius: usize,
    ) -> FovResult {
        let mut visible = HashSet::new();
        visible.insert(origin);

        let radius_i32 = radius as i32;

        let octant_transforms: [(i32, i32, i32, i32); 8] = [
            (1, 0, 0, 1),
            (0, 1, 1, 0),
            (-1, 0, 0, 1),
            (0, -1, 1, 0),
            (1, 0, 0, -1),
            (0, 1, -1, 0),
            (-1, 0, 0, -1),
            (0, -1, -1, 0),
        ];

        for (xx, xy, yx, yy) in &octant_transforms {
            Self::cast_light(
                map,
                origin.0,
                origin.1,
                1,
                1.0,
                0.0,
                radius_i32,
                *xx,
                *xy,
                *yx,
                *yy,
                &mut visible,
            );
        }

        FovResult {
            visible,
            newly_explored: HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rogue::dungeon::{DungeonConfig, DungeonGenerator, RoomCorridorGenerator};
    use crate::rogue::rng::GameRng;

    fn make_test_map() -> DungeonMap {
        let mut rng = GameRng::from_seed(42);
        let config = DungeonConfig {
            width: 40,
            height: 30,
            min_rooms: 4,
            max_rooms: 6,
            ..Default::default()
        };
        RoomCorridorGenerator::new().generate(&mut rng, &config)
    }

    #[test]
    fn test_raycasting_basic() {
        let map = make_test_map();
        let origin = map.rooms[0].rect.center();
        let result = RaycastingFov::new().compute(&map, origin, 8);
        assert!(result.visible.contains(&origin));
        assert!(result.visible.len() > 1);
    }

    #[test]
    fn test_shadowcasting_basic() {
        let map = make_test_map();
        let origin = map.rooms[0].rect.center();
        let result = ShadowcastingFov::new().compute(&map, origin, 8);
        assert!(result.visible.contains(&origin));
        assert!(result.visible.len() > 1);
    }

    #[test]
    fn test_shadowcasting_sees_room() {
        let map = make_test_map();
        let origin = map.rooms[0].rect.center();
        let result = ShadowcastingFov::new().compute(&map, origin, 20);

        let room = &map.rooms[0];
        for y in room.rect.y..room.rect.y + room.rect.h {
            for x in room.rect.x..room.rect.x + room.rect.w {
                if map.tile(x, y).is_walkable() {
                    assert!(
                        result.visible.contains(&(x, y)),
                        "Room tile ({}, {}) should be visible from {:?}",
                        x, y, origin
                    );
                }
            }
        }
    }

    #[test]
    fn test_shadowcasting_radius_limit() {
        let map = make_test_map();
        let origin = map.rooms[0].rect.center();
        let radius = 5;
        let result = ShadowcastingFov::new().compute(&map, origin, radius);

        for &(x, y) in &result.visible {
            let dx = (x as i32 - origin.0 as i32).abs();
            let dy = (y as i32 - origin.1 as i32).abs();
            assert!(
                dx * dx + dy * dy <= (radius * radius) as i32 + 2,
                "Visible tile ({}, {}) is beyond radius {}",
                x, y, radius
            );
        }
    }

    #[test]
    fn test_fov_with_explored() {
        let map = make_test_map();
        let origin = map.rooms[0].rect.center();
        let mut explored = HashSet::new();
        explored.insert(origin);

        let result = ShadowcastingFov::new().compute_with_explored(&map, origin, 8, &explored);
        assert!(!result.newly_explored.contains(&origin));
        assert!(result.newly_explored.len() > 0 || result.visible.len() == 1);
    }

    #[test]
    fn test_walls_block_vision() {
        let mut map = crate::rogue::dungeon::DungeonMap::new(20, 20);
        for x in 0..20 {
            map.set_tile(x, 10, crate::rogue::dungeon::Tile::Floor);
        }
        for y in 0..20 {
            map.set_tile(10, y, crate::rogue::dungeon::Tile::Floor);
        }
        map.set_tile(10, 10, crate::rogue::dungeon::Tile::Wall);

        let result = ShadowcastingFov::new().compute(&map, (10, 9), 10);
        assert!(result.visible.contains(&(10, 9)));
        assert!(!result.visible.contains(&(10, 11)));
    }
}
