// pathfind: 寻路算法
//
// 提供 A*、Dijkstra、JPS 三种寻路算法
// 支持四方向/八方向移动、自定义地形代价、多种启发函数

use super::dungeon::{DungeonMap, Tile};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heuristic {
    Manhattan,
    Euclidean,
    Chebyshev,
    Octile,
}

impl Heuristic {
    pub fn calculate(&self, dx: i32, dy: i32) -> f64 {
        match self {
            Heuristic::Manhattan => (dx.abs() + dy.abs()) as f64,
            Heuristic::Euclidean => ((dx * dx + dy * dy) as f64).sqrt(),
            Heuristic::Chebyshev => dx.abs().max(dy.abs()) as f64,
            Heuristic::Octile => {
                let dx = dx.abs() as f64;
                let dy = dy.abs() as f64;
                dx.max(dy) + (2.0_f64.sqrt() - 1.0) * dx.min(dy)
            }
        }
    }
}

pub struct PathConfig {
    pub allow_diagonal: bool,
    pub cut_corners: bool,
    pub heuristic: Heuristic,
    pub cost_fn: Option<std::rc::Rc<dyn Fn(Tile) -> f64>>,
}

impl Default for PathConfig {
    fn default() -> Self {
        PathConfig {
            allow_diagonal: false,
            cut_corners: false,
            heuristic: Heuristic::Manhattan,
            cost_fn: None,
        }
    }
}

impl PathConfig {
    pub fn diagonal() -> Self {
        PathConfig {
            allow_diagonal: true,
            cut_corners: false,
            heuristic: Heuristic::Octile,
            cost_fn: None,
        }
    }

    fn tile_cost(&self, tile: Tile) -> f64 {
        if let Some(ref cost_fn) = self.cost_fn {
            cost_fn(tile)
        } else {
            match tile {
                Tile::Floor | Tile::StairsUp | Tile::StairsDown => 1.0,
                Tile::Door => 1.0,
                Tile::Water => 2.0,
                Tile::Lava => 3.0,
                Tile::Wall => f64::INFINITY,
            }
        }
    }
}

pub trait PathFinder {
    fn find_path(
        &self,
        map: &DungeonMap,
        start: (usize, usize),
        end: (usize, usize),
        config: &PathConfig,
    ) -> Option<Vec<(usize, usize)>>;
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct Node {
    pos: (usize, usize),
    parent: Option<(usize, usize)>,
    g: u64,
    f: u64,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then_with(|| other.g.cmp(&self.g))
    }
}

fn reconstruct_path(
    came_from: &HashMap<(usize, usize), (usize, usize)>,
    start: (usize, usize),
    end: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    let mut current = end;
    path.push(current);
    while current != start {
        match came_from.get(&current) {
            Some(&prev) => {
                current = prev;
                path.push(current);
            }
            None => break,
        }
    }
    path.reverse();
    path
}

fn get_neighbors(
    map: &DungeonMap,
    pos: (usize, usize),
    config: &PathConfig,
) -> Vec<((usize, usize), f64)> {
    let mut neighbors = Vec::new();
    let (x, y) = pos;
    let x = x as i32;
    let y = y as i32;

    let cardinal = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let diagonal = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

    for &(dx, dy) in &cardinal {
        let nx = x + dx;
        let ny = y + dy;
        if nx >= 0 && ny >= 0 {
            let (nx, ny) = (nx as usize, ny as usize);
            if map.in_bounds(nx, ny) {
                let tile = map.tile(nx, ny);
                if tile.is_walkable() {
                    let cost = config.tile_cost(tile);
                    if cost.is_finite() {
                        neighbors.push(((nx, ny), cost));
                    }
                }
            }
        }
    }

    if config.allow_diagonal {
        for &(dx, dy) in &diagonal {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && ny >= 0 {
                let (nx, ny) = (nx as usize, ny as usize);
                if map.in_bounds(nx, ny) {
                    let tile = map.tile(nx, ny);
                    if tile.is_walkable() {
                        let cost = config.tile_cost(tile);
                        if cost.is_finite() {
                            let px = x as usize;
                            let py = y as usize;
                            if !config.cut_corners {
                                let tile_x = map.tile(px, ny as usize);
                                let tile_y = map.tile(nx as usize, py);
                                if !tile_x.is_walkable() && !tile_y.is_walkable() {
                                    continue;
                                }
                            }
                            neighbors.push(((nx, ny), cost * std::f64::consts::SQRT_2));
                        }
                    }
                }
            }
        }
    }

    neighbors
}

pub struct AStarFinder;

impl PathFinder for AStarFinder {
    fn find_path(
        &self,
        map: &DungeonMap,
        start: (usize, usize),
        end: (usize, usize),
        config: &PathConfig,
    ) -> Option<Vec<(usize, usize)>> {
        if !map.in_bounds(start.0, start.1) || !map.in_bounds(end.0, end.1) {
            return None;
        }
        if !map.tile(start.0, start.1).is_walkable() || !map.tile(end.0, end.1).is_walkable() {
            return None;
        }
        if start == end {
            return Some(vec![start]);
        }

        let heuristic_scale: f64 = 1.0;
        let mut open = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), u64> = HashMap::new();
        let mut closed: HashSet<(usize, usize)> = HashSet::new();

        let h = (config.heuristic.calculate(
            end.0 as i32 - start.0 as i32,
            end.1 as i32 - start.1 as i32,
        ) * heuristic_scale * 1000.0) as u64;

        g_score.insert(start, 0);
        open.push(Node {
            pos: start,
            parent: None,
            g: 0,
            f: h,
        });

        while let Some(current) = open.pop() {
            if current.pos == end {
                return Some(reconstruct_path(&came_from, start, end));
            }

            if closed.contains(&current.pos) {
                continue;
            }
            closed.insert(current.pos);

            let neighbors = get_neighbors(map, current.pos, config);
            for (neighbor_pos, cost) in neighbors {
                if closed.contains(&neighbor_pos) {
                    continue;
                }

                let move_cost = (cost * 1000.0) as u64;
                let tentative_g = current.g + move_cost;

                let prev_g = g_score.get(&neighbor_pos).copied().unwrap_or(u64::MAX);
                if tentative_g < prev_g {
                    came_from.insert(neighbor_pos, current.pos);
                    g_score.insert(neighbor_pos, tentative_g);

                    let h = (config.heuristic.calculate(
                        end.0 as i32 - neighbor_pos.0 as i32,
                        end.1 as i32 - neighbor_pos.1 as i32,
                    ) * heuristic_scale * 1000.0) as u64;

                    open.push(Node {
                        pos: neighbor_pos,
                        parent: Some(current.pos),
                        g: tentative_g,
                        f: tentative_g + h,
                    });
                }
            }
        }

        None
    }
}

pub struct DijkstraFinder;

impl PathFinder for DijkstraFinder {
    fn find_path(
        &self,
        map: &DungeonMap,
        start: (usize, usize),
        end: (usize, usize),
        config: &PathConfig,
    ) -> Option<Vec<(usize, usize)>> {
        if !map.in_bounds(start.0, start.1) || !map.in_bounds(end.0, end.1) {
            return None;
        }
        if !map.tile(start.0, start.1).is_walkable() || !map.tile(end.0, end.1).is_walkable() {
            return None;
        }
        if start == end {
            return Some(vec![start]);
        }

        let mut open = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut dist: HashMap<(usize, usize), u64> = HashMap::new();
        let mut visited: HashSet<(usize, usize)> = HashSet::new();

        dist.insert(start, 0);
        open.push(Node {
            pos: start,
            parent: None,
            g: 0,
            f: 0,
        });

        while let Some(current) = open.pop() {
            if current.pos == end {
                return Some(reconstruct_path(&came_from, start, end));
            }

            if visited.contains(&current.pos) {
                continue;
            }
            visited.insert(current.pos);

            let neighbors = get_neighbors(map, current.pos, config);
            for (neighbor_pos, cost) in neighbors {
                if visited.contains(&neighbor_pos) {
                    continue;
                }

                let move_cost = (cost * 1000.0) as u64;
                let tentative_g = current.g + move_cost;

                let prev_g = dist.get(&neighbor_pos).copied().unwrap_or(u64::MAX);
                if tentative_g < prev_g {
                    came_from.insert(neighbor_pos, current.pos);
                    dist.insert(neighbor_pos, tentative_g);
                    open.push(Node {
                        pos: neighbor_pos,
                        parent: Some(current.pos),
                        g: tentative_g,
                        f: tentative_g,
                    });
                }
            }
        }

        None
    }
}

pub struct JpsFinder;

impl JpsFinder {
    fn jump(
        &self,
        map: &DungeonMap,
        current: (i32, i32),
        direction: (i32, i32),
        end: (i32, i32),
        config: &PathConfig,
    ) -> Option<(usize, usize)> {
        let (cx, cy) = current;
        let (dx, dy) = direction;
        let nx = cx + dx;
        let ny = cy + dy;

        if nx < 0 || ny < 0 || nx >= map.width as i32 || ny >= map.height as i32 {
            return None;
        }

        let (nx, ny) = (nx as usize, ny as usize);
        if !map.tile(nx, ny).is_walkable() {
            return None;
        }

        if (nx, ny) == (end.0 as usize, end.1 as usize) {
            return Some((nx, ny));
        }

        if dx != 0 && dy != 0 {
            if (nx > 0 && !map.tile(nx - 1, ny).is_walkable() && nx + 1 < map.width && map.tile(nx + 1, ny).is_walkable())
                || (ny > 0 && !map.tile(nx, ny - 1).is_walkable() && ny + 1 < map.height && map.tile(nx, ny + 1).is_walkable())
            {
                return Some((nx, ny));
            }

            if self.jump(map, (nx as i32, ny as i32), (dx, 0), end, config).is_some()
                || self.jump(map, (nx as i32, ny as i32), (0, dy), end, config).is_some()
            {
                return Some((nx, ny));
            }
        } else if dx != 0 {
            if ny > 0 && ny + 1 < map.height {
                let behind_x = (nx as i32 - dx) as usize;
                if behind_x < map.width {
                    if (!map.tile(behind_x, ny - 1).is_walkable() && map.tile(nx, ny - 1).is_walkable())
                        || (!map.tile(behind_x, ny + 1).is_walkable() && map.tile(nx, ny + 1).is_walkable())
                    {
                        return Some((nx, ny));
                    }
                }
            }
        } else {
            if nx > 0 && nx + 1 < map.width {
                let behind_y = (ny as i32 - dy) as usize;
                if behind_y < map.height {
                    if (!map.tile(nx - 1, behind_y).is_walkable() && map.tile(nx - 1, ny).is_walkable())
                        || (!map.tile(nx + 1, behind_y).is_walkable() && map.tile(nx + 1, ny).is_walkable())
                    {
                        return Some((nx, ny));
                    }
                }
            }
        }

        self.jump(map, (nx as i32, ny as i32), direction, end, config)
    }

    fn get_successors(
        &self,
        map: &DungeonMap,
        current: (usize, usize),
        end: (usize, usize),
        config: &PathConfig,
    ) -> Vec<((usize, usize), f64)> {
        let mut successors = Vec::new();
        let (cx, cy) = (current.0 as i32, current.1 as i32);
        let end_i32 = (end.0 as i32, end.1 as i32);

        let directions: &[(i32, i32)] = if config.allow_diagonal {
            &[(0, -1), (0, 1), (-1, 0), (1, 0), (-1, -1), (-1, 1), (1, -1), (1, 1)]
        } else {
            &[(0, -1), (0, 1), (-1, 0), (1, 0)]
        };

        for &(dx, dy) in directions {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx < 0 || ny < 0 || nx >= map.width as i32 || ny >= map.height as i32 {
                continue;
            }
            if !map.tile(nx as usize, ny as usize).is_walkable() {
                continue;
            }

            if let Some(jump_point) = self.jump(map, (cx, cy), (dx, dy), end_i32, config) {
                let dist = if dx != 0 && dy != 0 {
                    let d = ((jump_point.0 as i32 - cx).abs() as f64)
                        .hypot((jump_point.1 as i32 - cy).abs() as f64);
                    d
                } else {
                    ((jump_point.0 as i32 - cx).abs() + (jump_point.1 as i32 - cy).abs()) as f64
                };
                successors.push((jump_point, dist));
            }
        }

        successors
    }
}

impl PathFinder for JpsFinder {
    fn find_path(
        &self,
        map: &DungeonMap,
        start: (usize, usize),
        end: (usize, usize),
        config: &PathConfig,
    ) -> Option<Vec<(usize, usize)>> {
        if !map.in_bounds(start.0, start.1) || !map.in_bounds(end.0, end.1) {
            return None;
        }
        if !map.tile(start.0, start.1).is_walkable() || !map.tile(end.0, end.1).is_walkable() {
            return None;
        }
        if start == end {
            return Some(vec![start]);
        }

        let mut open = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), u64> = HashMap::new();
        let mut closed: HashSet<(usize, usize)> = HashSet::new();

        let h = (config.heuristic.calculate(
            end.0 as i32 - start.0 as i32,
            end.1 as i32 - start.1 as i32,
        ) * 1000.0) as u64;

        g_score.insert(start, 0);
        open.push(Node {
            pos: start,
            parent: None,
            g: 0,
            f: h,
        });

        while let Some(current) = open.pop() {
            if current.pos == end {
                return Some(reconstruct_path(&came_from, start, end));
            }

            if closed.contains(&current.pos) {
                continue;
            }
            closed.insert(current.pos);

            let successors = self.get_successors(map, current.pos, end, config);
            for (successor_pos, cost) in successors {
                if closed.contains(&successor_pos) {
                    continue;
                }

                let move_cost = (cost * 1000.0) as u64;
                let tentative_g = current.g + move_cost;

                let prev_g = g_score.get(&successor_pos).copied().unwrap_or(u64::MAX);
                if tentative_g < prev_g {
                    came_from.insert(successor_pos, current.pos);
                    g_score.insert(successor_pos, tentative_g);

                    let h = (config.heuristic.calculate(
                        end.0 as i32 - successor_pos.0 as i32,
                        end.1 as i32 - successor_pos.1 as i32,
                    ) * 1000.0) as u64;

                    open.push(Node {
                        pos: successor_pos,
                        parent: Some(current.pos),
                        g: tentative_g,
                        f: tentative_g + h,
                    });
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rogue::rng::GameRng;
    use crate::rogue::dungeon::{DungeonConfig, RoomCorridorGenerator, DungeonGenerator};

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
    fn test_astar_finds_path() {
        let map = make_test_map();
        let start = map.rooms[0].rect.center();
        let end = map.rooms.last().unwrap().rect.center();
        let config = PathConfig::default();
        let path = AStarFinder.find_path(&map, start, end, &config);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path[0], start);
        assert_eq!(*path.last().unwrap(), end);
    }

    #[test]
    fn test_dijkstra_finds_path() {
        let map = make_test_map();
        let start = map.rooms[0].rect.center();
        let end = map.rooms.last().unwrap().rect.center();
        let config = PathConfig::default();
        let path = DijkstraFinder.find_path(&map, start, end, &config);
        assert!(path.is_some());
    }

    #[test]
    fn test_astar_diagonal() {
        let map = make_test_map();
        let start = map.rooms[0].rect.center();
        let end = map.rooms.last().unwrap().rect.center();
        let config = PathConfig::diagonal();
        let path = AStarFinder.find_path(&map, start, end, &config);
        assert!(path.is_some());
    }

    #[test]
    fn test_no_path_to_wall() {
        let map = make_test_map();
        let start = map.rooms[0].rect.center();
        let config = PathConfig::default();
        let path = AStarFinder.find_path(&map, start, (0, 0), &config);
        assert!(path.is_none());
    }

    #[test]
    fn test_same_start_end() {
        let map = make_test_map();
        let start = map.rooms[0].rect.center();
        let config = PathConfig::default();
        let path = AStarFinder.find_path(&map, start, start, &config);
        assert_eq!(path, Some(vec![start]));
    }

    #[test]
    fn test_heuristic_calculations() {
        assert_eq!(Heuristic::Manhattan.calculate(3, 4), 7.0);
        assert!((Heuristic::Euclidean.calculate(3, 4) - 5.0).abs() < 0.001);
        assert_eq!(Heuristic::Chebyshev.calculate(3, 4), 4.0);
    }

    #[test]
    fn test_jps_finds_path() {
        let mut map = DungeonMap::new(20, 20);
        for y in 0..20 {
            for x in 0..20 {
                map.set_tile(x, y, Tile::Floor);
            }
        }
        map.set_tile(10, 5, Tile::Wall);
        map.set_tile(10, 6, Tile::Wall);
        map.set_tile(10, 7, Tile::Wall);

        let start = (5, 10);
        let end = (15, 10);
        let config = PathConfig::default();
        let path = JpsFinder.find_path(&map, start, end, &config);
        assert!(path.is_some());
        if let Some(p) = path {
            assert_eq!(p[0], start);
            assert_eq!(*p.last().unwrap(), end);
        }
    }
}
