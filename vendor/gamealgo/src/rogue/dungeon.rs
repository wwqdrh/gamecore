// dungeon: 地牢/地图生成
//
// 提供4种经典地牢生成算法，统一输出 DungeonMap 结构
// - BspGenerator: BSP递归分割，适合规则地牢
// - CellularAutomataGenerator: 细胞自动机，自然洞穴风格
// - RandomWalkGenerator: 随机游走，蜿蜒通道风格
// - RoomCorridorGenerator: 房间+走廊，最灵活

use super::rng::GameRng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    Floor,
    Door,
    StairsDown,
    StairsUp,
    Water,
    Lava,
}

impl Tile {
    pub fn is_walkable(&self) -> bool {
        matches!(self, Tile::Floor | Tile::Door | Tile::StairsDown | Tile::StairsUp | Tile::Water | Tile::Lava)
    }

    pub fn is_transparent(&self) -> bool {
        !matches!(self, Tile::Wall)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomType {
    Start,
    End,
    Combat,
    Treasure,
    Shop,
    Rest,
    Boss,
    Elite,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Rect { x, y, w, h }
    }

    pub fn center(&self) -> (usize, usize) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    pub fn intersects_with_margin(&self, other: &Rect, margin: usize) -> bool {
        self.x < other.x + other.w + margin
            && self.x + self.w + margin > other.x
            && self.y < other.y + other.h + margin
            && self.y + self.h + margin > other.y
    }

    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: usize,
    pub rect: Rect,
    pub room_type: RoomType,
    pub connections: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corridor {
    pub from_room: usize,
    pub to_room: usize,
    pub points: Vec<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonMap {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
}

impl DungeonMap {
    pub fn new(width: usize, height: usize) -> Self {
        DungeonMap {
            width,
            height,
            tiles: vec![Tile::Wall; width * height],
            rooms: Vec::new(),
            corridors: Vec::new(),
        }
    }

    pub fn tile(&self, x: usize, y: usize) -> Tile {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x]
        } else {
            Tile::Wall
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    pub fn walkable_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        let dirs = [(0, 1), (0, usize::MAX), (1, 0), (usize::MAX, 0)];
        for (dx, dy) in &dirs {
            if let (Some(nx), Some(ny)) = (
                x.checked_add(*dx).or_else(|| x.checked_sub(1)),
                y.checked_add(*dy).or_else(|| y.checked_sub(1)),
            ) {
                if self.in_bounds(nx, ny) && self.tile(nx, ny).is_walkable() {
                    neighbors.push((nx, ny));
                }
            }
        }
        neighbors
    }

    pub fn walkable_neighbors_8dir(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        for dx in [-1i32, 0, 1] {
            for dy in [-1i32, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 {
                    let (nx, ny) = (nx as usize, ny as usize);
                    if self.in_bounds(nx, ny) && self.tile(nx, ny).is_walkable() {
                        neighbors.push((nx, ny));
                    }
                }
            }
        }
        neighbors
    }

    pub fn find_room_at(&self, x: usize, y: usize) -> Option<usize> {
        self.rooms.iter().position(|r| r.rect.contains(x, y))
    }

    fn fill_rect(&mut self, rect: &Rect, tile: Tile) {
        for y in rect.y..rect.y + rect.h {
            for x in rect.x..rect.x + rect.w {
                self.set_tile(x, y, tile);
            }
        }
    }

    fn carve_corridor_h(&mut self, x1: usize, x2: usize, y: usize) {
        let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        for x in start..=end {
            if self.tile(x, y) == Tile::Wall {
                self.set_tile(x, y, Tile::Floor);
            }
        }
    }

    fn carve_corridor_v(&mut self, y1: usize, y2: usize, x: usize) {
        let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        for y in start..=end {
            if self.tile(x, y) == Tile::Wall {
                self.set_tile(x, y, Tile::Floor);
            }
        }
    }

    pub fn to_string_map(&self) -> String {
        let mut result = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let ch = match self.tile(x, y) {
                    Tile::Wall => '#',
                    Tile::Floor => '.',
                    Tile::Door => '+',
                    Tile::StairsDown => '>',
                    Tile::StairsUp => '<',
                    Tile::Water => '~',
                    Tile::Lava => '^',
                };
                result.push(ch);
            }
            result.push('\n');
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonConfig {
    pub width: usize,
    pub height: usize,
    pub min_rooms: usize,
    pub max_rooms: usize,
    pub room_min_size: usize,
    pub room_max_size: usize,
    pub extra_corridor_chance: f64,
    pub room_type_weights: Vec<(RoomType, f64)>,
}

impl Default for DungeonConfig {
    fn default() -> Self {
        DungeonConfig {
            width: 80,
            height: 60,
            min_rooms: 8,
            max_rooms: 15,
            room_min_size: 5,
            room_max_size: 12,
            extra_corridor_chance: 0.1,
            room_type_weights: vec![
                (RoomType::Combat, 5.0),
                (RoomType::Treasure, 1.5),
                (RoomType::Shop, 1.0),
                (RoomType::Rest, 1.0),
                (RoomType::Elite, 0.8),
            ],
        }
    }
}

pub trait DungeonGenerator {
    fn generate(&self, rng: &mut GameRng, config: &DungeonConfig) -> DungeonMap;
}

fn assign_room_types(map: &mut DungeonMap, rng: &mut GameRng, config: &DungeonConfig) {
    if map.rooms.len() < 2 {
        return;
    }

    map.rooms[0].room_type = RoomType::Start;
    let last = map.rooms.len() - 1;
    map.rooms[last].room_type = RoomType::End;

    if config.room_type_weights.is_empty() {
        return;
    }

    let room_count = map.rooms.len();
    for room in map.rooms.iter_mut().skip(1).take(room_count - 2) {
        if let Some(room_type) = rng.choose_weighted(&config.room_type_weights) {
            room.room_type = room_type.clone();
        }
    }
}

fn connect_rooms_mst(map: &mut DungeonMap, rng: &mut GameRng, extra_chance: f64) {
    if map.rooms.len() < 2 {
        return;
    }

    let n = map.rooms.len();
    let mut in_mst = vec![false; n];
    let mut edges: Vec<(usize, usize, f64)> = Vec::new();

    in_mst[0] = true;
    let mut mst_size = 1;

    while mst_size < n {
        let mut best_edge: Option<(usize, usize, f64)> = None;
        for i in 0..n {
            if !in_mst[i] {
                continue;
            }
            for j in 0..n {
                if in_mst[j] {
                    continue;
                }
                let (xi, yi) = map.rooms[i].rect.center();
                let (xj, yj) = map.rooms[j].rect.center();
                let dist = ((xi as f64 - xj as f64).powi(2) + (yi as f64 - yj as f64).powi(2)).sqrt();
                match best_edge {
                    None => best_edge = Some((i, j, dist)),
                    Some((_, _, best_dist)) if dist < best_dist => {
                        best_edge = Some((i, j, dist));
                    }
                    _ => {}
                }
            }
        }

        if let Some((from, to, _)) = best_edge {
            in_mst[to] = true;
            mst_size += 1;
            edges.push((from, to, 0.0));
        } else {
            break;
        }
    }

    for (from, to, _) in &edges {
        let (x1, y1) = map.rooms[*from].rect.center();
        let (x2, y2) = map.rooms[*to].rect.center();
        let mut points = Vec::new();

        if rng.next_bool(0.5) {
            map.carve_corridor_h(x1, x2, y1);
            map.carve_corridor_v(y1, y2, x2);
            for x in x1.min(x2)..=x1.max(x2) {
                points.push((x, y1));
            }
            for y in y1.min(y2)..=y1.max(y2) {
                points.push((x2, y));
            }
        } else {
            map.carve_corridor_v(y1, y2, x1);
            map.carve_corridor_h(x1, x2, y2);
            for y in y1.min(y2)..=y1.max(y2) {
                points.push((x1, y));
            }
            for x in x1.min(x2)..=x1.max(x2) {
                points.push((x, y2));
            }
        }

        map.corridors.push(Corridor {
            from_room: *from,
            to_room: *to,
            points,
        });
        map.rooms[*from].connections.push(*to);
        map.rooms[*to].connections.push(*from);
    }

    for i in 0..n {
        for j in (i + 1)..n {
            if map.rooms[i].connections.contains(&j) {
                continue;
            }
            if rng.next_bool(extra_chance) {
                let (x1, y1) = map.rooms[i].rect.center();
                let (x2, y2) = map.rooms[j].rect.center();
                let points = Vec::new();

                if rng.next_bool(0.5) {
                    map.carve_corridor_h(x1, x2, y1);
                    map.carve_corridor_v(y1, y2, x2);
                } else {
                    map.carve_corridor_v(y1, y2, x1);
                    map.carve_corridor_h(x1, x2, y2);
                }

                map.corridors.push(Corridor {
                    from_room: i,
                    to_room: j,
                    points,
                });
                map.rooms[i].connections.push(j);
                map.rooms[j].connections.push(i);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BspGenerator {
    pub min_split_size: usize,
    pub split_bias: f64,
}

impl Default for BspGenerator {
    fn default() -> Self {
        BspGenerator {
            min_split_size: 8,
            split_bias: 0.5,
        }
    }
}

impl BspGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

struct BspNode {
    rect: Rect,
    left: Option<Box<BspNode>>,
    right: Option<Box<BspNode>>,
    room_id: Option<usize>,
}

impl BspGenerator {
    fn split(&self, rng: &mut GameRng, rect: Rect) -> BspNode {
        let min = self.min_split_size;
        let can_split_h = rect.w >= min * 2;
        let can_split_v = rect.h >= min * 2;

        if !can_split_h && !can_split_v {
            return BspNode {
                rect,
                left: None,
                right: None,
                room_id: None,
            };
        }

        let split_h = if can_split_h && !can_split_v {
            true
        } else if !can_split_h && can_split_v {
            false
        } else {
            rng.next_bool(self.split_bias)
        };

        let (left_rect, right_rect) = if split_h {
            let split_pos = rng.next_range(
                (rect.x + min) as i32,
                (rect.x + rect.w - min) as i32,
            ) as usize;
            (
                Rect::new(rect.x, rect.y, split_pos - rect.x, rect.h),
                Rect::new(split_pos, rect.y, rect.x + rect.w - split_pos, rect.h),
            )
        } else {
            let split_pos = rng.next_range(
                (rect.y + min) as i32,
                (rect.y + rect.h - min) as i32,
            ) as usize;
            (
                Rect::new(rect.x, rect.y, rect.w, split_pos - rect.y),
                Rect::new(rect.x, split_pos, rect.w, rect.y + rect.h - split_pos),
            )
        };

        let left = Box::new(self.split(rng, left_rect));
        let right = Box::new(self.split(rng, right_rect));

        BspNode {
            rect,
            left: Some(left),
            right: Some(right),
            room_id: None,
        }
    }

    fn create_rooms(
        &self,
        rng: &mut GameRng,
        node: &mut BspNode,
        map: &mut DungeonMap,
        config: &DungeonConfig,
        room_counter: &mut usize,
    ) {
        if node.left.is_none() && node.right.is_none() {
            let r = &node.rect;
            let w = rng.next_range(
                config.room_min_size as i32,
                (r.w.saturating_sub(2)).min(config.room_max_size) as i32 + 1,
            ) as usize;
            let h = rng.next_range(
                config.room_min_size as i32,
                (r.h.saturating_sub(2)).min(config.room_max_size) as i32 + 1,
            ) as usize;
            let x = rng.next_range(r.x as i32 + 1, (r.x + r.w - w) as i32 + 1) as usize;
            let y = rng.next_range(r.y as i32 + 1, (r.y + r.h - h) as i32 + 1) as usize;

            let room_rect = Rect::new(x, y, w, h);
            map.fill_rect(&room_rect, Tile::Floor);

            let id = *room_counter;
            *room_counter += 1;
            map.rooms.push(Room {
                id,
                rect: room_rect,
                room_type: RoomType::Combat,
                connections: Vec::new(),
            });
            node.room_id = Some(id);
            return;
        }

        if let Some(ref mut left) = node.left {
            self.create_rooms(rng, left, map, config, room_counter);
        }
        if let Some(ref mut right) = node.right {
            self.create_rooms(rng, right, map, config, room_counter);
        }
    }

    fn connect_subtree(
        &self,
        rng: &mut GameRng,
        node: &BspNode,
        map: &mut DungeonMap,
    ) {
        let left_id = node.left.as_ref().and_then(|l| self.get_room_id(l));
        let right_id = node.right.as_ref().and_then(|r| self.get_room_id(r));

        if let (Some(li), Some(ri)) = (left_id, right_id) {
            let (x1, y1) = map.rooms[li].rect.center();
            let (x2, y2) = map.rooms[ri].rect.center();
            let mut points = Vec::new();

            if rng.next_bool(0.5) {
                map.carve_corridor_h(x1, x2, y1);
                map.carve_corridor_v(y1, y2, x2);
            } else {
                map.carve_corridor_v(y1, y2, x1);
                map.carve_corridor_h(x1, x2, y2);
            }

            for x in x1.min(x2)..=x1.max(x2) {
                points.push((x, y1));
            }
            for y in y1.min(y2)..=y1.max(y2) {
                points.push((x2, y));
            }

            map.corridors.push(Corridor {
                from_room: li,
                to_room: ri,
                points,
            });
            map.rooms[li].connections.push(ri);
            map.rooms[ri].connections.push(li);
        }

        if let Some(ref left) = node.left {
            self.connect_subtree(rng, left, map);
        }
        if let Some(ref right) = node.right {
            self.connect_subtree(rng, right, map);
        }
    }

    fn get_room_id(&self, node: &BspNode) -> Option<usize> {
        node.room_id.or_else(|| {
            node.left
                .as_ref()
                .and_then(|l| self.get_room_id(l))
                .or_else(|| node.right.as_ref().and_then(|r| self.get_room_id(r)))
        })
    }
}

impl DungeonGenerator for BspGenerator {
    fn generate(&self, rng: &mut GameRng, config: &DungeonConfig) -> DungeonMap {
        let mut map = DungeonMap::new(config.width, config.height);
        let root_rect = Rect::new(1, 1, config.width - 2, config.height - 2);
        let mut root = self.split(rng, root_rect);

        let mut room_counter = 0usize;
        self.create_rooms(rng, &mut root, &mut map, config, &mut room_counter);
        self.connect_subtree(rng, &root, &mut map);

        if !map.rooms.is_empty() {
            map.rooms[0].room_type = RoomType::Start;
            if map.rooms.len() > 1 {
                let last = map.rooms.len() - 1;
                map.rooms[last].room_type = RoomType::End;
                let (cx, cy) = map.rooms[last].rect.center();
                map.set_tile(cx, cy, Tile::StairsDown);
            }
            let (sx, sy) = map.rooms[0].rect.center();
            map.set_tile(sx, sy, Tile::StairsUp);
        }

        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellularAutomataGenerator {
    pub fill_chance: f64,
    pub smoothing_passes: usize,
    pub birth_limit: usize,
    pub death_limit: usize,
}

impl Default for CellularAutomataGenerator {
    fn default() -> Self {
        CellularAutomataGenerator {
            fill_chance: 0.45,
            smoothing_passes: 5,
            birth_limit: 4,
            death_limit: 3,
        }
    }
}

impl CellularAutomataGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DungeonGenerator for CellularAutomataGenerator {
    fn generate(&self, rng: &mut GameRng, config: &DungeonConfig) -> DungeonMap {
        let mut map = DungeonMap::new(config.width, config.height);

        for y in 1..config.height - 1 {
            for x in 1..config.width - 1 {
                if rng.next_bool(self.fill_chance) {
                    map.set_tile(x, y, Tile::Floor);
                }
            }
        }

        for _ in 0..self.smoothing_passes {
            let mut new_tiles = map.tiles.clone();
            for y in 1..config.height - 1 {
                for x in 1..config.width - 1 {
                    let mut wall_count = 0usize;
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = (x as i32 + dx) as usize;
                            let ny = (y as i32 + dy) as usize;
                            if map.tile(nx, ny) == Tile::Wall {
                                wall_count += 1;
                            }
                        }
                    }

                    let idx = y * config.width + x;
                    if map.tile(x, y) == Tile::Wall {
                        if wall_count >= self.birth_limit {
                            new_tiles[idx] = Tile::Floor;
                        }
                    } else if wall_count < self.death_limit {
                        new_tiles[idx] = Tile::Wall;
                    }
                }
            }
            map.tiles = new_tiles;
        }

        let mut visited = vec![false; config.width * config.height];
        let mut regions: Vec<Vec<(usize, usize)>> = Vec::new();

        for y in 1..config.height - 1 {
            for x in 1..config.width - 1 {
                if map.tile(x, y).is_walkable() && !visited[y * config.width + x] {
                    let mut region = Vec::new();
                    let mut stack = vec![(x, y)];
                    visited[y * config.width + x] = true;

                    while let Some((cx, cy)) = stack.pop() {
                        region.push((cx, cy));
                        for (nx, ny) in map.walkable_neighbors(cx, cy) {
                            if !visited[ny * config.width + nx] {
                                visited[ny * config.width + nx] = true;
                                stack.push((nx, ny));
                            }
                        }
                    }

                    regions.push(region);
                }
            }
        }

        regions.sort_by(|a, b| b.len().cmp(&a.len()));

        if regions.is_empty() {
            return map;
        }

        let _main_region: HashSet<(usize, usize)> = regions[0].iter().copied().collect();

        let mut room_id = 0usize;
        let mut region_rooms: Vec<(usize, Rect)> = Vec::new();

        for region in &regions {
            if region.len() < config.room_min_size * config.room_min_size {
                for &(x, y) in region {
                    map.set_tile(x, y, Tile::Wall);
                }
                continue;
            }

            let min_x = region.iter().map(|(x, _)| *x).min().unwrap_or(0);
            let max_x = region.iter().map(|(x, _)| *x).max().unwrap_or(0);
            let min_y = region.iter().map(|(_, y)| *y).min().unwrap_or(0);
            let max_y = region.iter().map(|(_, y)| *y).max().unwrap_or(0);

            let rect = Rect::new(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1);
            map.rooms.push(Room {
                id: room_id,
                rect,
                room_type: RoomType::Combat,
                connections: Vec::new(),
            });
            region_rooms.push((room_id, rect));
            room_id += 1;
        }

        if !region_rooms.is_empty() {
            region_rooms[0].0 = 0;
            map.rooms[0].room_type = RoomType::Start;
            let (sx, sy) = map.rooms[0].rect.center();
            map.set_tile(sx, sy, Tile::StairsUp);

            if map.rooms.len() > 1 {
                let last = map.rooms.len() - 1;
                map.rooms[last].room_type = RoomType::End;
                let (ex, ey) = map.rooms[last].rect.center();
                map.set_tile(ex, ey, Tile::StairsDown);
            }
        }

        for i in 1..region_rooms.len() {
            let (from_id, from_rect) = region_rooms[i - 1];
            let (to_id, to_rect) = region_rooms[i];
            let (x1, y1) = from_rect.center();
            let (x2, y2) = to_rect.center();

            if rng.next_bool(0.5) {
                map.carve_corridor_h(x1, x2, y1);
                map.carve_corridor_v(y1, y2, x2);
            } else {
                map.carve_corridor_v(y1, y2, x1);
                map.carve_corridor_h(x1, x2, y2);
            }

            map.corridors.push(Corridor {
                from_room: from_id,
                to_room: to_id,
                points: Vec::new(),
            });
            map.rooms[from_id].connections.push(to_id);
            map.rooms[to_id].connections.push(from_id);
        }

        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomWalkGenerator {
    pub steps: usize,
    pub walk_chance: f64,
    pub min_room_distance: usize,
}

impl Default for RandomWalkGenerator {
    fn default() -> Self {
        RandomWalkGenerator {
            steps: 500,
            walk_chance: 0.5,
            min_room_distance: 10,
        }
    }
}

impl RandomWalkGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DungeonGenerator for RandomWalkGenerator {
    fn generate(&self, rng: &mut GameRng, config: &DungeonConfig) -> DungeonMap {
        let mut map = DungeonMap::new(config.width, config.height);

        let start_x = config.width / 2;
        let start_y = config.height / 2;
        let mut x = start_x as i32;
        let mut y = start_y as i32;

        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        for _ in 0..self.steps {
            if x > 0 && x < config.width as i32 - 1 && y > 0 && y < config.height as i32 - 1 {
                map.set_tile(x as usize, y as usize, Tile::Floor);
            }

            let dir = rng.choose(&directions).unwrap();
            let nx = x + dir.0;
            let ny = y + dir.1;

            if nx > 1 && nx < config.width as i32 - 2 && ny > 1 && ny < config.height as i32 - 2 {
                x = nx;
                y = ny;
            }
        }

        let mut visited = vec![false; config.width * config.height];
        let mut regions: Vec<Vec<(usize, usize)>> = Vec::new();

        for y in 1..config.height - 1 {
            for x in 1..config.width - 1 {
                if map.tile(x, y).is_walkable() && !visited[y * config.width + x] {
                    let mut region = Vec::new();
                    let mut stack = vec![(x, y)];
                    visited[y * config.width + x] = true;

                    while let Some((cx, cy)) = stack.pop() {
                        region.push((cx, cy));
                        for (nx, ny) in map.walkable_neighbors(cx, cy) {
                            if !visited[ny * config.width + nx] {
                                visited[ny * config.width + nx] = true;
                                stack.push((nx, ny));
                            }
                        }
                    }

                    regions.push(region);
                }
            }
        }

        regions.sort_by(|a, b| b.len().cmp(&a.len()));

        if regions.is_empty() {
            return map;
        }

        let mut room_id = 0usize;
        let mut region_rooms: Vec<(usize, Rect)> = Vec::new();

        for region in &regions {
            let min_x = region.iter().map(|(x, _)| *x).min().unwrap_or(0);
            let max_x = region.iter().map(|(x, _)| *x).max().unwrap_or(0);
            let min_y = region.iter().map(|(_, y)| *y).min().unwrap_or(0);
            let max_y = region.iter().map(|(_, y)| *y).max().unwrap_or(0);

            let rect = Rect::new(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1);
            map.rooms.push(Room {
                id: room_id,
                rect,
                room_type: RoomType::Combat,
                connections: Vec::new(),
            });
            region_rooms.push((room_id, rect));
            room_id += 1;
        }

        if !region_rooms.is_empty() {
            map.rooms[0].room_type = RoomType::Start;
            let (sx, sy) = map.rooms[0].rect.center();
            map.set_tile(sx, sy, Tile::StairsUp);

            if map.rooms.len() > 1 {
                let last = map.rooms.len() - 1;
                map.rooms[last].room_type = RoomType::End;
                let (ex, ey) = map.rooms[last].rect.center();
                map.set_tile(ex, ey, Tile::StairsDown);
            }
        }

        for i in 1..region_rooms.len() {
            let (from_id, from_rect) = region_rooms[i - 1];
            let (to_id, to_rect) = region_rooms[i];
            let (x1, y1) = from_rect.center();
            let (x2, y2) = to_rect.center();

            if rng.next_bool(0.5) {
                map.carve_corridor_h(x1, x2, y1);
                map.carve_corridor_v(y1, y2, x2);
            } else {
                map.carve_corridor_v(y1, y2, x1);
                map.carve_corridor_h(x1, x2, y2);
            }

            map.rooms[from_id].connections.push(to_id);
            map.rooms[to_id].connections.push(from_id);
        }

        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomCorridorGenerator {
    pub room_margin: usize,
    pub max_placement_attempts: usize,
}

impl Default for RoomCorridorGenerator {
    fn default() -> Self {
        RoomCorridorGenerator {
            room_margin: 2,
            max_placement_attempts: 100,
        }
    }
}

impl RoomCorridorGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DungeonGenerator for RoomCorridorGenerator {
    fn generate(&self, rng: &mut GameRng, config: &DungeonConfig) -> DungeonMap {
        let mut map = DungeonMap::new(config.width, config.height);
        let target_rooms = rng.next_range(config.min_rooms as i32, config.max_rooms as i32 + 1) as usize;

        for attempt in 0..self.max_placement_attempts {
            if map.rooms.len() >= target_rooms {
                break;
            }

            let w = rng.next_range(config.room_min_size as i32, config.room_max_size as i32 + 1) as usize;
            let h = rng.next_range(config.room_min_size as i32, config.room_max_size as i32 + 1) as usize;
            let x = rng.next_range(1, (config.width - w - 1) as i32 + 1) as usize;
            let y = rng.next_range(1, (config.height - h - 1) as i32 + 1) as usize;

            let new_rect = Rect::new(x, y, w, h);

            let overlaps = map.rooms.iter().any(|r| {
                r.rect.intersects_with_margin(&new_rect, self.room_margin)
            });

            if !overlaps {
                let id = map.rooms.len();
                map.fill_rect(&new_rect, Tile::Floor);
                map.rooms.push(Room {
                    id,
                    rect: new_rect,
                    room_type: RoomType::Combat,
                    connections: Vec::new(),
                });
            }

            if attempt > target_rooms * 10 && map.rooms.len() >= config.min_rooms {
                break;
            }
        }

        connect_rooms_mst(&mut map, rng, config.extra_corridor_chance);
        assign_room_types(&mut map, rng, config);

        if !map.rooms.is_empty() {
            let (sx, sy) = map.rooms[0].rect.center();
            map.set_tile(sx, sy, Tile::StairsUp);
            if map.rooms.len() > 1 {
                let last = map.rooms.len() - 1;
                let (ex, ey) = map.rooms[last].rect.center();
                map.set_tile(ex, ey, Tile::StairsDown);
            }
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsp_generator() {
        let mut rng = GameRng::from_seed(42);
        let config = DungeonConfig::default();
        let map = BspGenerator::new().generate(&mut rng, &config);
        assert!(!map.rooms.is_empty());
        assert!(map.rooms[0].room_type == RoomType::Start);
        assert!(map.rooms.last().unwrap().room_type == RoomType::End);
    }

    #[test]
    fn test_cellular_automata_generator() {
        let mut rng = GameRng::from_seed(42);
        let config = DungeonConfig::default();
        let map = CellularAutomataGenerator::new().generate(&mut rng, &config);
        assert!(!map.rooms.is_empty());
    }

    #[test]
    fn test_random_walk_generator() {
        let mut rng = GameRng::from_seed(42);
        let config = DungeonConfig::default();
        let map = RandomWalkGenerator::new().generate(&mut rng, &config);
        assert!(!map.rooms.is_empty());
    }

    #[test]
    fn test_room_corridor_generator() {
        let mut rng = GameRng::from_seed(42);
        let config = DungeonConfig::default();
        let map = RoomCorridorGenerator::new().generate(&mut rng, &config);
        assert!(!map.rooms.is_empty());
        assert!(map.rooms.len() >= config.min_rooms);
    }

    #[test]
    fn test_map_connectivity() {
        let mut rng = GameRng::from_seed(42);
        let config = DungeonConfig::default();
        let map = RoomCorridorGenerator::new().generate(&mut rng, &config);

        for room in &map.rooms {
            assert!(!room.connections.is_empty() || map.rooms.len() == 1);
        }
    }

    #[test]
    fn test_same_seed_same_map() {
        let config = DungeonConfig::default();
        let mut rng1 = GameRng::from_seed(42);
        let mut rng2 = GameRng::from_seed(42);
        let map1 = RoomCorridorGenerator::new().generate(&mut rng1, &config);
        let map2 = RoomCorridorGenerator::new().generate(&mut rng2, &config);
        assert_eq!(map1.rooms.len(), map2.rooms.len());
        assert_eq!(map1.to_string_map(), map2.to_string_map());
    }

    #[test]
    fn test_tile_walkable() {
        assert!(Tile::Floor.is_walkable());
        assert!(Tile::Door.is_walkable());
        assert!(!Tile::Wall.is_walkable());
    }

    #[test]
    fn test_rect_operations() {
        let r = Rect::new(5, 5, 10, 10);
        assert_eq!(r.center(), (10, 10));
        assert!(r.contains(5, 5));
        assert!(!r.contains(15, 15));
    }
}
