use macroquad::prelude::*;
use ::rand::prelude::*;
use std::collections::HashSet;

const TILE_SIZE: f32 = 20.0;

// Add this new enum to represent different tile types
#[derive(Clone, PartialEq)]
enum Tile {
    Wall,
    Floor,
    StairsUp,
    StairsDown,
}

impl Tile {
    fn to_char(&self) -> char {
        match self {
            Tile::Wall => '#',
            Tile::Floor => '.',
            Tile::StairsUp => '<',    // Changed from > to <
            Tile::StairsDown => '>',   // This is correct
        }
    }
}

struct MapManager {
    maps: Vec<Map>,
    current_level: i32,
    config: GameConfig,
}

impl MapManager {
    fn new(config: GameConfig) -> Self {
        let initial_map = Map::new(config.map_width, config.map_height, 0, None);
        let mut maps = Vec::new();
        maps.push(initial_map);

        Self {
            maps,
            current_level: 0,
            config,
        }
    }

    fn current_map(&self) -> &Map {
        &self.maps[self.current_level as usize]
    }

    fn current_map_mut(&mut self) -> &mut Map {
        &mut self.maps[self.current_level as usize]
    }

    fn change_level(&mut self, new_level: i32) -> Option<(f32, f32)> {
        if new_level < 0 || new_level >= 10 {
            return None;
        }

        let going_down = new_level > self.current_level;
        self.current_level = new_level;

        // Generate new map if it doesn't exist
        if new_level as usize >= self.maps.len() {
            let stairs_up_pos = if going_down {
                Some(self.maps.last().unwrap().down_stairs.unwrap())
            } else {
                None
            };
            let new_map = Map::new(self.config.map_width, self.config.map_height, new_level, stairs_up_pos);
            self.maps.push(new_map);
        }

        // Return player spawn position
        if going_down {
            self.maps[new_level as usize].up_stairs.map(|(x, y)| (x as f32, y as f32))
        } else {
            self.maps[new_level as usize].down_stairs.map(|(x, y)| (x as f32, y as f32))
        }
    }
}

// Define item types
#[derive(Clone, Debug, PartialEq)]
pub enum ItemType {
    Weapon(i32),    // Attack bonus
    Armor(i32),     // Defense bonus
    Potion(i32),    // Healing amount
    Scroll(Effect), // Magic effect
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    Teleport,
    Lightning(i32), // Damage
    Fireball(i32),  // Damage and radius
    Confusion(i32), // Duration
}

#[derive(Clone, Debug)]
pub struct Item {
    name: String,
    item_type: ItemType,
    symbol: char,
    color: Color,
}

impl Item {
    fn new_sword() -> Self {
        Self {
            name: "Sword".to_string(),
            item_type: ItemType::Weapon(2),
            symbol: '/',
            color: SKYBLUE,
        }
    }

    fn new_armor() -> Self {
        Self {
            name: "Chain Mail".to_string(),
            item_type: ItemType::Armor(2),
            symbol: '[',
            color: LIGHTGRAY,
        }
    }

    fn new_health_potion() -> Self {
        Self {
            name: "Health Potion".to_string(),
            item_type: ItemType::Potion(10),
            symbol: '!',
            color: PINK,
        }
    }

    fn new_lightning_scroll() -> Self {
        Self {
            name: "Lightning Scroll".to_string(),
            item_type: ItemType::Scroll(Effect::Lightning(20)),
            symbol: '?',
            color: YELLOW,
        }
    }
}

// Inventory struct to manage items
#[derive(Clone)]
pub struct Inventory {
    items: Vec<Item>,
    capacity: usize,
    equipped_weapon: Option<Item>,
    equipped_armor: Option<Item>,
}

impl Inventory {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::new(),
            capacity,
            equipped_weapon: None,
            equipped_armor: None,
        }
    }

    pub fn add_item(&mut self, item: Item) -> Result<(), String> {
        if self.items.len() >= self.capacity {
            Err("Inventory is full!".to_string())
        } else {
            self.items.push(item);
            Ok(())
        }
    }

    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn equip_item(&mut self, index: usize) -> Result<String, String> {
        if index >= self.items.len() {
            return Err("Invalid item index!".to_string());
        }

        let item = &self.items[index];
        match item.item_type {
            ItemType::Weapon(_) => {
                let item = self.items.remove(index);
                if let Some(old_weapon) = self.equipped_weapon.replace(item) {
                    self.items.push(old_weapon);
                }
                Ok("Weapon equipped!".to_string())
            }
            ItemType::Armor(_) => {
                let item = self.items.remove(index);
                if let Some(old_armor) = self.equipped_armor.replace(item) {
                    self.items.push(old_armor);
                }
                Ok("Armor equipped!".to_string())
            }
            _ => Err("This item cannot be equipped!".to_string()),
        }
    }

    pub fn use_item(&mut self, index: usize, entity: &mut Entity, game_state: &mut GameState) -> Result<String, String> {
        if index >= self.items.len() {
            return Err("Invalid item index!".to_string());
        }

        // Clone the item type to avoid borrowing issues
        let item_type = self.items[index].item_type.clone();

        match item_type {
            ItemType::Potion(heal_amount) => {
                entity.stats.hp = (entity.stats.hp + heal_amount).min(entity.stats.max_hp);
                self.items.remove(index);
                Ok(format!("Used health potion! Healed for {} HP", heal_amount))
            }
            ItemType::Scroll(effect) => {
                match effect {
                    Effect::Lightning(damage) => {
                        if let Some(closest_monster) = game_state.find_closest_monster(entity.x, entity.y, 5.0) {
                            closest_monster.stats.hp -= damage;
                            self.items.remove(index);
                            Ok(format!("Lightning bolt hits monster for {} damage!", damage))
                        } else {
                            Err("No monster in range!".to_string())
                        }
                    }
                    // Implement other scroll effects here
                    _ => Err("Effect not implemented!".to_string()),
                }
            }
            _ => Err("This item cannot be used!".to_string()),
        }
    }

    pub fn get_equipment_bonuses(&self) -> (i32, i32) {
        let weapon_bonus = self.equipped_weapon
            .as_ref()
            .and_then(|w| match w.item_type {
                ItemType::Weapon(bonus) => Some(bonus),
                _ => None,
            })
            .unwrap_or(0);

        let armor_bonus = self.equipped_armor
            .as_ref()
            .and_then(|a| match a.item_type {
                ItemType::Armor(bonus) => Some(bonus),
                _ => None,
            })
            .unwrap_or(0);

        (weapon_bonus, armor_bonus)
    }
}

#[derive(Clone)]
struct Stats {
    hp: i32,
    max_hp: i32,
    attack: i32,
    defense: i32,
    speed: f32,
    last_move: f32,
    perception: f32,
}

// A* Node structure for pathfinding
#[derive(Clone, Eq, PartialEq, Hash)]
struct Node {
    position: (i32, i32),
    g_cost: i32,
    f_cost: i32,
    parent: Option<(i32, i32)>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f_cost.cmp(&self.f_cost)  // Reverse for min-heap
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

#[derive(Clone)]
struct Entity {
    x: f32,
    y: f32,
    symbol: char,
    color: Color,
    stats: Stats,
    is_player: bool,
    inventory: Option<Inventory>,
}

impl Entity {
    fn new_player() -> Self {
        Self {
            x: 5.0,
            y: 5.0,
            symbol: '@',
            color: YELLOW,
            stats: Stats {
                hp: 30,
                max_hp: 30,
                attack: 5,
                defense: 2,
                speed: 10.0,
                last_move: 0.0,
                perception: 8.0,
            },
            is_player: true,
            inventory: Some(Inventory::new(20))
        }
    }

    fn new_monster(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            symbol: 'g', // goblin
            color: RED,
            stats: Stats {
                hp: 15,
                max_hp: 15,
                attack: 3,
                defense: 1,
                speed: 2.0,
                last_move: 0.0,
                perception: 8.0,
            },
            is_player: false,
            inventory: None,
        }
    }

    // Add method to check if target is within perception range
    fn can_perceive_target(&self, target_x: f32, target_y: f32) -> bool {
        let dx = target_x - self.x;
        let dy = target_y - self.y;
        let distance = (dx * dx + dy * dy).sqrt();
        distance <= self.stats.perception
    }

    fn attack(&mut self, target: &mut Entity) -> String {
        let damage = (self.stats.attack - target.stats.defense).max(1);
        target.stats.hp -= damage;

        format!("{} hits {} for {} damage!",
                if self.is_player { "Player" } else { "Monster" },
                if target.is_player { "Player" } else { "Monster" },
                damage
        )
    }

    fn is_alive(&self) -> bool {
        self.stats.hp > 0
    }

    fn can_move(&self, current_time: f32) -> bool {
        current_time - self.stats.last_move >= 1.0 / self.stats.speed
    }

    fn update_last_move(&mut self, current_time: f32) {
        self.stats.last_move = current_time;
    }

    fn with_inventory(mut self, capacity: usize) -> Self {
        self.inventory = Some(Inventory::new(capacity));
        self
    }

    fn get_total_attack(&self) -> i32 {
        let (weapon_bonus, _) = self.inventory
            .as_ref()
            .map(|inv| inv.get_equipment_bonuses())
            .unwrap_or((0, 0));
        self.stats.attack + weapon_bonus
    }

    fn get_total_defense(&self) -> i32 {
        let (_, armor_bonus) = self.inventory
            .as_ref()
            .map(|inv| inv.get_equipment_bonuses())
            .unwrap_or((0, 0));
        self.stats.defense + armor_bonus
    }
}

struct Camera {
    x: f32,
    y: f32,
    viewport_width: usize,
    viewport_height: usize,
}

impl Camera {
    fn new(viewport_width: usize, viewport_height: usize) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            viewport_width,
            viewport_height,
        }
    }

    fn follow(&mut self, target_x: f32, target_y: f32, map_width: usize, map_height: usize) {
        // Center the camera on the target
        self.x = target_x - self.viewport_width as f32 / 2.0;
        self.y = target_y - self.viewport_height as f32 / 2.0;

        // Clamp camera position to map bounds
        self.x = self.x.clamp(0.0, (map_width as f32) - self.viewport_width as f32);
        self.y = self.y.clamp(0.0, (map_height as f32) - self.viewport_height as f32);
    }

    fn world_to_screen(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        (
            (world_x - self.x) * TILE_SIZE,
            (world_y - self.y) * TILE_SIZE,
        )
    }

    fn is_visible(&self, world_x: f32, world_y: f32) -> bool {
        world_x >= self.x && world_x < self.x + self.viewport_width as f32 &&
            world_y >= self.y && world_y < self.y + self.viewport_height as f32
    }
}

struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Vec<Tile>>,
    rooms: Vec<Vec<Room>>,
    level: i32,
    up_stairs: Option<(usize, usize)>,
    down_stairs: Option<(usize, usize)>,
}

impl Map {
    fn new(width: usize, height: usize, level: i32, stairs_up_pos: Option<(usize, usize)>) -> Self {
        let mut map = Map {
            width,
            height,
            tiles: vec![vec![Tile::Wall; width]; height],
            rooms: Vec::new(),
            level,
            up_stairs: stairs_up_pos,
            down_stairs: None,
        };

        // Use level as seed for consistent but different layouts per level
        let seed = level as u64;
        let rng = StdRng::seed_from_u64(seed);
        map.generate_dungeon_with_stairs_seeded(rng);
        map
    }

    fn generate_dungeon_with_stairs_seeded(&mut self, mut rng: impl Rng) {
        // Existing generate_dungeon_with_stairs logic but using provided rng
        let max_rooms = 15;
        let min_room_size = 5;
        let max_room_size = 10;

        let mut temp_rooms = Vec::new();
        self.tiles = vec![vec![Tile::Wall; self.width]; self.height];
        self.rooms.clear();

        for _ in 0..max_rooms {
            let w = rng.gen_range(min_room_size..max_room_size);
            let h = rng.gen_range(min_room_size..max_room_size);
            let x = rng.gen_range(1..self.width as i32 - w - 1);
            let y = rng.gen_range(1..self.height as i32 - h - 1);

            let new_room = Room::new(x, y, w, h);

            if !temp_rooms.iter().any(|r: &Room| r.intersects(&new_room)) {
                self.create_room(&new_room);

                if let Some(prev_room) = temp_rooms.last() {
                    let (prev_x, prev_y) = prev_room.center();
                    let (new_x, new_y) = new_room.center();

                    if rng.gen_bool(0.5) {
                        self.create_horizontal_tunnel(prev_x, new_x, prev_y);
                        self.create_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        self.create_vertical_tunnel(prev_y, new_y, prev_x);
                        self.create_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                temp_rooms.push(new_room);
            }
        }

        self.rooms = vec![temp_rooms];

        // Place stairs
        if self.level > 0 {
            if let Some((x, y)) = self.up_stairs {
                self.tiles[y][x] = Tile::StairsUp;
            } else if let Some(first_row) = self.rooms.first() {
                if let Some(first_room) = first_row.first() {
                    let (x, y) = first_room.center();
                    let (x, y) = (x as usize, y as usize);
                    self.tiles[y][x] = Tile::StairsUp;
                    self.up_stairs = Some((x, y));
                }
            }
        }

        if self.level < 9 {
            if let Some(last_row) = self.rooms.last() {
                if let Some(last_room) = last_row.last() {
                    let (x, y) = last_room.center();
                    let (x, y) = (x as usize, y as usize);
                    self.tiles[y][x] = Tile::StairsDown;
                    self.down_stairs = Some((x, y));
                }
            }
        }
    }

    fn check_for_stairs(&self, x: f32, y: f32) -> Option<i32> {
        let x = x as usize;
        let y = y as usize;

        if y >= self.height || x >= self.width {
            return None;
        }

        match self.tiles[y][x] {
            Tile::StairsDown => Some(self.level + 1),
            Tile::StairsUp => Some(self.level - 1),
            _ => None,
        }
    }

    fn organize_rooms(&mut self, temp_rooms: Vec<Room>) {
        let mut organized_rooms: Vec<Vec<Room>> = Vec::new();
        let room_height = 10;

        let mut sorted_rooms = temp_rooms;
        sorted_rooms.sort_by_key(|room| room.y);

        if sorted_rooms.is_empty() {
            self.rooms = Vec::new();
            return;
        }

        let mut current_row: Vec<Room> = Vec::new();
        let mut current_y = sorted_rooms[0].y;

        for room in sorted_rooms {
            if (room.y - current_y).abs() > room_height {
                if !current_row.is_empty() {
                    organized_rooms.push(current_row);
                    current_row = Vec::new();
                }
                current_y = room.y;
            }
            current_row.push(room);
        }

        if !current_row.is_empty() {
            organized_rooms.push(current_row);
        }

        for row in &mut organized_rooms {
            row.sort_by_key(|room| room.x);
        }

        self.rooms = organized_rooms;
    }

    fn generate_dungeon_with_stairs(&mut self) {
        let mut rng = thread_rng();
        let max_rooms = 15;
        let min_room_size = 5;
        let max_room_size = 10;

        let mut temp_rooms = Vec::new();
        self.tiles = vec![vec![Tile::Wall; self.width]; self.height];
        self.rooms.clear();

        for _ in 0..max_rooms {
            let w = rng.gen_range(min_room_size..max_room_size);
            let h = rng.gen_range(min_room_size..max_room_size);
            let x = rng.gen_range(1..self.width as i32 - w - 1);
            let y = rng.gen_range(1..self.height as i32 - h - 1);

            let new_room = Room::new(x, y, w, h);

            if !temp_rooms.iter().any(|r: &Room| r.intersects(&new_room)) {
                self.create_room(&new_room);

                if let Some(prev_room) = temp_rooms.last() {
                    let (prev_x, prev_y) = prev_room.center();
                    let (new_x, new_y) = new_room.center();

                    if rng.gen_bool(0.5) {
                        self.create_horizontal_tunnel(prev_x, new_x, prev_y);
                        self.create_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        self.create_vertical_tunnel(prev_y, new_y, prev_x);
                        self.create_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                temp_rooms.push(new_room);
            }
        }

        self.rooms = vec![temp_rooms];

        // Place stairs
        if self.level > 0 {
            if let Some((x, y)) = self.up_stairs {
                self.tiles[y][x] = Tile::StairsUp;
            }
        }

        if self.level < 9 {
            if let Some(first_row) = self.rooms.first() {
                if let Some(last_room) = first_row.last() {
                    let (x, y) = last_room.center();
                    let (x, y) = (x as usize, y as usize);
                    self.tiles[y][x] = Tile::StairsDown;
                    self.down_stairs = Some((x, y));
                }
            }
        }
    }

    fn place_stairs(&mut self) {
        if self.level > 0 {
            if let Some((x, y)) = self.up_stairs {
                self.tiles[y][x] = Tile::StairsUp;
            } else if let Some(first_row) = self.rooms.first() {
                if let Some(first_room) = first_row.first() {
                    let (x, y) = first_room.center();
                    let (x, y) = (x as usize, y as usize);
                    if y < self.height && x < self.width {
                        self.tiles[y][x] = Tile::StairsUp;
                        self.up_stairs = Some((x, y));
                    }
                }
            }
        }

        if self.level < 9 {
            if let Some(last_row) = self.rooms.last() {
                if let Some(last_room) = last_row.last() {
                    let (x, y) = last_room.center();
                    let (x, y) = (x as usize, y as usize);
                    if y < self.height && x < self.width {
                        self.tiles[y][x] = Tile::StairsDown;
                        self.down_stairs = Some((x, y));
                    }
                }
            }
        }
    }

    fn generate_dungeon(&mut self) {
        let mut rng = thread_rng();
        let max_rooms = 15;
        let min_room_size = 5;
        let max_room_size = 10;

        let mut temp_rooms = Vec::new();

        for _ in 0..max_rooms {
            let w = rng.gen_range(min_room_size..max_room_size);
            let h = rng.gen_range(min_room_size..max_room_size);
            let x = rng.gen_range(1..self.width as i32 - w - 1);
            let y = rng.gen_range(1..self.height as i32 - h - 1);

            let new_room = Room::new(x, y, w, h);

            if !temp_rooms.iter().any(|r: &Room| r.intersects(&new_room)) {
                self.create_room(&new_room);

                if let Some(prev_room) = temp_rooms.last() {
                    let (prev_x, prev_y) = prev_room.center();
                    let (new_x, new_y) = new_room.center();

                    if rng.gen_bool(0.5) {
                        self.create_horizontal_tunnel(prev_x, new_x, prev_y);
                        self.create_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        self.create_vertical_tunnel(prev_y, new_y, prev_x);
                        self.create_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                temp_rooms.push(new_room);
            }
        }

        self.organize_rooms(temp_rooms);
    }

    fn create_room(&mut self, room: &Room) {
        for y in room.y..room.y + room.height {
            let y_idx = y as usize;
            if y_idx >= self.height {
                continue;
            }
            for x in room.x..room.x + room.width {
                let x_idx = x as usize;
                if x_idx >= self.width {
                    continue;
                }
                self.tiles[y_idx][x_idx] = Tile::Floor;
            }
        }
    }

    fn create_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        let y_idx = y as usize;
        if y_idx >= self.height {
            return;
        }
        for x in x1.min(x2)..=x1.max(x2) {
            let x_idx = x as usize;
            if x_idx >= self.width {
                continue;
            }
            self.tiles[y_idx][x_idx] = Tile::Floor;
        }
    }

    fn create_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        let x_idx = x as usize;
        if x_idx >= self.width {
            return;
        }
        for y in y1.min(y2)..=y1.max(y2) {
            let y_idx = y as usize;
            if y_idx >= self.height {
                continue;
            }
            self.tiles[y_idx][x_idx] = Tile::Floor;
        }
    }

    fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        match self.tiles[y as usize][x as usize] {
            Tile::Floor | Tile::StairsUp | Tile::StairsDown => true,
            Tile::Wall => false,
        }
    }

    fn is_wall(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return true;
        }
        self.tiles[y][x] == Tile::Wall
    }

    fn place_monsters(&self) -> (Option<(f32, f32)>, Vec<(f32, f32)>) {
        let mut monster_positions = Vec::new();
        let mut rng = thread_rng();
        let mut used_positions = HashSet::new();

        // Get first room from first row
        let player_spawn = self.rooms.get(0)
            .and_then(|row| row.get(0))
            .and_then(|room| {
                let center = room.center();
                if self.is_walkable(center.0, center.1) {
                    used_positions.insert((center.0, center.1));
                    Some((center.0 as f32, center.1 as f32))
                } else {
                    room.inner_tiles().into_iter()
                        .find(|&(x, y)| self.is_walkable(x, y))
                        .map(|(x, y)| {
                            used_positions.insert((x, y));
                            (x as f32, y as f32)
                        })
                }
            });

        // Skip first room when placing monsters
        for room_row in self.rooms.iter().skip(1) {
            for room in room_row {
                let num_monsters = rng.gen_range(1..4);
                let walkable_tiles: Vec<_> = room.inner_tiles().into_iter()
                    .filter(|&(x, y)|
                        self.is_walkable(x, y) &&
                            !used_positions.contains(&(x, y))
                    )
                    .collect();

                if walkable_tiles.is_empty() {
                    continue;
                }

                for _ in 0..num_monsters {
                    if let Some(&(x, y)) = walkable_tiles.choose(&mut rng) {
                        if used_positions.insert((x, y)) {
                            monster_positions.push((x as f32, y as f32));
                        }
                    }
                }
            }
        }

        (player_spawn, monster_positions)
    }

    fn find_path(&self, start: (i32, i32), goal: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        use std::collections::{BinaryHeap, HashSet};

        let mut open_set = BinaryHeap::new();
        let mut closed_set: HashSet<Node> = HashSet::new();

        // Initialize start node
        let start_node = Node {
            position: start,
            g_cost: 0,
            f_cost: manhattan_distance(start, goal),
            parent: None,
        };

        open_set.push(start_node);

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current_pos = current.position;
                let mut current_node = Some(current);

                while let Some(node) = current_node {
                    path.push(node.position);
                    if let Some(parent_pos) = node.parent {
                        current_pos = parent_pos;
                        current_node = closed_set.iter()
                            .find(|n| n.position == parent_pos)
                            .cloned();
                    } else {
                        break;
                    }
                }

                path.reverse();
                return Some(path);
            }

            closed_set.insert(current.clone());

            // Check neighbors
            for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let next_pos = (
                    current.position.0 + dx,
                    current.position.1 + dy
                );

                if !self.is_walkable(next_pos.0, next_pos.1) {
                    continue;
                }

                if closed_set.iter().any(|n| n.position == next_pos) {
                    continue;
                }

                let g_cost = current.g_cost + 1;
                let h_cost = manhattan_distance(next_pos, goal);
                let f_cost = g_cost + h_cost;

                let next_node = Node {
                    position: next_pos,
                    g_cost,
                    f_cost,
                    parent: Some(current.position),
                };

                open_set.push(next_node);
            }
        }

        None
    }

    // Update the draw method to use different colors for different tiles
    fn draw(&self, camera: &Camera) {
        let start_x = camera.x.floor() as usize;
        let start_y = camera.y.floor() as usize;
        let end_x = (camera.x + camera.viewport_width as f32).ceil() as usize;
        let end_y = (camera.y + camera.viewport_height as f32).ceil() as usize;

        for y in start_y..end_y.min(self.height) {
            for x in start_x..end_x.min(self.width) {
                let tile = &self.tiles[y][x];
                let (screen_x, screen_y) = camera.world_to_screen(x as f32, y as f32);

                let (char, color) = match tile {
                    Tile::Wall => ('#', DARKGRAY),
                    Tile::Floor => ('.', GRAY),
                    Tile::StairsUp => ('<', YELLOW),
                    Tile::StairsDown => ('>', YELLOW),
                };

                draw_text(
                    &char.to_string(),
                    screen_x,
                    screen_y + TILE_SIZE,
                    TILE_SIZE,
                    color,
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Room {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Room {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Room { x, y, width, height }
    }

    fn random_position(&self, rng: &mut impl Rng) -> (i32, i32) {
        // Get inner positions to avoid placing items on walls
        let x = rng.gen_range((self.x + 1)..(self.x + self.width - 1));
        let y = rng.gen_range((self.y + 1)..(self.y + self.height - 1));
        (x, y)
    }

    fn center(&self) -> (i32, i32) {
        (
            self.x + self.width / 2,
            self.y + self.height / 2,
        )
    }

    fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.width &&
            self.x + self.width >= other.x &&
            self.y <= other.y + other.height &&
            self.y + self.height >= other.y
    }

    fn inner_tiles(&self) -> Vec<(i32, i32)> {
        let mut tiles = Vec::new();
        for y in (self.y + 1)..(self.y + self.height - 1) {
            for x in (self.x + 1)..(self.x + self.width - 1) {
                tiles.push((x, y));
            }
        }
        tiles
    }
}

struct GameState {
    player: Entity,
    monsters: Vec<Entity>,
    combat_log: Vec<String>,
    player_turn: bool,
    ground_items: Vec<(f32, f32, Item)>,
    inventory_open: bool,
    map_manager: MapManager,
    level_states: Vec<LevelState>,
}

impl GameState {
    fn new(config: GameConfig) -> Self {
        let map_manager = MapManager::new(config);
        let mut game_state = Self {
            player: Entity::new_player(),
            monsters: Vec::new(),
            combat_log: Vec::new(),
            player_turn: true,
            ground_items: Vec::new(),
            inventory_open: false,
            map_manager,
            level_states: vec![],
        };

        game_state.initialize_current_level();
        game_state
    }

    fn save_current_level_state(&mut self) {
        let current_level = self.map_manager.current_level as usize;
        // Ensure we have space for this level
        while self.level_states.len() <= current_level {
            self.level_states.push(LevelState {
                monsters: Vec::new(),
                ground_items: Vec::new(),
            });
        }

        // Create new state with cloned data
        let new_state = LevelState {
            monsters: self.monsters.clone(),
            ground_items: self.ground_items.clone(),
        };

        // Save the state
        self.level_states[current_level] = new_state;
    }

    fn load_level_state(&mut self, level: usize) {
        if level < self.level_states.len() {
            let state = &self.level_states[level];
            self.monsters = state.monsters.clone();
            self.ground_items = state.ground_items.clone();
        }
    }

    fn get_current_level_state(&self) -> Option<&LevelState> {
        self.level_states.get(self.map_manager.current_level as usize)
    }

    // Helper method to get current level state mutably
    fn get_current_level_state_mut(&mut self) -> Option<&mut LevelState> {
        self.level_states.get_mut(self.map_manager.current_level as usize)
    }

    fn initialize_current_level(&mut self) {
        if let Some(first_row) = self.map_manager.current_map().rooms.first() {
            if let Some(first_room) = first_row.first() {
                let (center_x, center_y) = first_room.center();
                self.player.x = center_x as f32;
                self.player.y = center_y as f32;
            }
        }

        let mut rng = thread_rng();
        let mut new_monsters = Vec::new();
        let map = self.map_manager.current_map();

        for row in &map.rooms {
            for room in row.iter().skip(1) {
                let num_monsters = rng.gen_range(0..3);
                for _ in 0..num_monsters {
                    let (x, y) = room.random_position(&mut rng);
                    if map.is_walkable(x, y) {
                        new_monsters.push(Entity::new_monster(x as f32, y as f32));
                    }
                }
            }
        }

        self.monsters = new_monsters;
        self.spawn_items_for_current_level();
    }

    fn spawn_items_for_current_level(&mut self) {
        let mut rng = thread_rng();
        self.ground_items.clear();

        let rooms = self.map_manager.current_map().rooms.clone();

        for row in &rooms {
            for room in row {
                if rng.gen_bool(0.6) {
                    let (x, y) = room.random_position(&mut rng);
                    let item = match rng.gen_range(0..4) {
                        0 => Item::new_sword(),
                        1 => Item::new_armor(),
                        2 => Item::new_health_potion(),
                        _ => Item::new_lightning_scroll(),
                    };
                    self.ground_items.push((x as f32, y as f32, item));
                }
            }
        }
    }

    fn handle_level_transition(&mut self) {
        let player_pos = (self.player.x as usize, self.player.y as usize);
        let current_level = self.map_manager.current_level;
        let map = self.map_manager.current_map();

        if player_pos.1 >= map.height || player_pos.0 >= map.width {
            return;
        }

        match map.tiles[player_pos.1][player_pos.0] {
            Tile::StairsDown => {
                if is_key_pressed(KeyCode::Period) {
                    self.save_current_level_state();
                    let next_level = current_level + 1;
                    let is_new_level = next_level as usize >= self.level_states.len();

                    if let Some((new_x, new_y)) = self.map_manager.change_level(next_level) {
                        self.player.x = new_x;
                        self.player.y = new_y;

                        if is_new_level {
                            self.initialize_current_level();
                        } else {
                            self.load_level_state(next_level as usize);
                        }

                        self.add_log_message(format!("Descended to level {}", next_level + 1));
                    }
                }
            },
            Tile::StairsUp => {
                if is_key_pressed(KeyCode::Comma) {
                    self.save_current_level_state();
                    let prev_level = current_level - 1;
                    if let Some((new_x, new_y)) = self.map_manager.change_level(prev_level) {
                        self.player.x = new_x;
                        self.player.y = new_y;
                        self.load_level_state(prev_level as usize);
                        self.add_log_message(format!("Ascended to level {}", prev_level + 1));
                    }
                }
            },
            _ => {}
        }
    }

    fn spawn_entities(&mut self, map: &Map) {
        // Spawn player in first room of first row
        if let Some(first_row) = map.rooms.first() {
            if let Some(first_room) = first_row.first() {
                let (center_x, center_y) = first_room.center();
                self.player.x = center_x as f32;
                self.player.y = center_y as f32;
            } else {
                // Fallback: scan for walkable tile
                for y in 0..map.height {
                    for x in 0..map.width {
                        if map.is_walkable(x as i32, y as i32) {
                            self.player.x = x as f32;
                            self.player.y = y as f32;
                            break;
                        }
                    }
                }
            }
        }

        self.monsters.clear();
        let rooms = map.rooms.clone();
        let mut rng = thread_rng();

        // Skip first row for monster spawning
        for row in rooms.iter().skip(1) {
            for room in row.iter() { // Changed from row to row.iter()
                let num_monsters = rng.gen_range(0..3);
                for _ in 0..num_monsters {
                    let mut tries = 0;
                    let max_tries = 10;

                    while tries < max_tries {
                        let (x, y) = room.random_position(&mut rng);
                        if map.is_walkable(x, y) {
                            let is_occupied = self.monsters.iter()
                                .any(|m| m.x == x as f32 && m.y == y as f32);

                            if !is_occupied {
                                self.monsters.push(Entity::new_monster(x as f32, y as f32));
                                break;
                            }
                        }
                        tries += 1;
                    }
                }
            }
        }
    }

    fn check_and_pickup_items(&mut self) {
        let mut items_to_pickup = Vec::new();

        // Find all items at player's position
        for (i, (x, y, _)) in self.ground_items.iter().enumerate() {
            if *x == self.player.x && *y == self.player.y {
                items_to_pickup.push(i);
            }
        }

        for &i in items_to_pickup.iter().rev() {
            if let Some((_, _, item)) = self.ground_items.get(i) {
                if let Some(ref mut inventory) = self.player.inventory {
                    match inventory.add_item(item.clone()) {
                        Ok(_) => {
                            self.add_log_message(format!("Picked up {}!", item.name));
                            self.ground_items.remove(i);
                        }
                        Err(e) => {
                            self.add_log_message(e);
                            break; // Stop picking up if inventory is full
                        }
                    }
                }
            }
        }
    }

    // Add this method to display inventory
    fn draw_inventory(&self) {
        if let Some(ref inventory) = self.player.inventory {
            // Draw semi-transparent background
            draw_rectangle(
                screen_width() * 0.1,
                screen_height() * 0.1,
                screen_width() * 0.8,
                screen_height() * 0.8,
                Color::new(0.0, 0.0, 0.0, 0.9),
            );

            // Draw title
            draw_text(
                "Inventory",
                screen_width() * 0.15,
                screen_height() * 0.15,
                30.0,
                WHITE,
            );

            // Draw equipped items
            let equipped_y = screen_height() * 0.2;
            draw_text(
                "Equipped:",
                screen_width() * 0.15,
                equipped_y,
                20.0,
                LIGHTGRAY,
            );

            if let Some(ref weapon) = inventory.equipped_weapon {
                draw_text(
                    &format!("Weapon: {}", weapon.name),
                    screen_width() * 0.15,
                    equipped_y + 25.0,
                    20.0,
                    weapon.color,
                );
            }

            if let Some(ref armor) = inventory.equipped_armor {
                draw_text(
                    &format!("Armor: {}", armor.name),
                    screen_width() * 0.15,
                    equipped_y + 50.0,
                    20.0,
                    armor.color,
                );
            }

            // Draw inventory items
            draw_text(
                "Items:",
                screen_width() * 0.15,
                equipped_y + 90.0,
                20.0,
                LIGHTGRAY,
            );

            for (i, item) in inventory.items.iter().enumerate() {
                let y_pos = equipped_y + 115.0 + (i as f32 * 25.0);
                draw_text(
                    &format!("{}) {} {}",
                             i + 1,
                             item.symbol,
                             item.name
                    ),
                    screen_width() * 0.15,
                    y_pos,
                    20.0,
                    item.color,
                );
            }

            // Draw usage instructions
            draw_text(
                "[E] Equip  [U] Use  [D] Drop  [Esc] Close",
                screen_width() * 0.15,
                screen_height() * 0.85,
                20.0,
                LIGHTGRAY,
            );
        }
    }

    fn add_log_message(&mut self, message: String) {
        self.combat_log.push(message);
        if self.combat_log.len() > 5 {
            self.combat_log.remove(0);
        }
    }

    fn process_monster_turns(&mut self, current_time: f32) {
        let player_pos = (self.player.x, self.player.y);
        let map = self.map_manager.current_map();

        let monster_positions: Vec<(f32, f32)> = self.monsters.iter()
            .filter(|m| m.is_alive())
            .map(|m| (m.x, m.y))
            .collect();

        for i in 0..self.monsters.len() {
            if !self.monsters[i].is_alive() || !self.monsters[i].can_move(current_time) {
                continue;
            }

            let monster = &mut self.monsters[i];
            let monster_pos = (monster.x as i32, monster.y as i32);
            let player_grid_pos = (player_pos.0 as i32, player_pos.1 as i32);

            let mut new_pos = monster_pos;

            if monster.can_perceive_target(player_pos.0, player_pos.1) {
                // Use A* pathfinding when player is within perception range
                if let Some(path) = map.find_path(monster_pos, player_grid_pos) {
                    if path.len() > 1 {  // Check if we have a next step
                        new_pos = path[1];  // Get the next position in the path
                    }
                }
            } else {
                // Random movement when player is not perceived
                let mut rng = thread_rng();
                let direction = rng.gen_range(0..4);
                new_pos = match direction {
                    0 => (monster_pos.0 + 1, monster_pos.1),
                    1 => (monster_pos.0 - 1, monster_pos.1),
                    2 => (monster_pos.0, monster_pos.1 + 1),
                    _ => (monster_pos.0, monster_pos.1 - 1),
                };
            }

            // Check if the new position is valid
            if map.is_walkable(new_pos.0, new_pos.1) {
                let new_pos_f = (new_pos.0 as f32, new_pos.1 as f32);

                // Check for collisions with other monsters
                let is_collision = monster_positions.iter()
                    .any(|&pos| pos.0 == new_pos_f.0 && pos.1 == new_pos_f.1);

                // Check for collision with player
                if player_pos.0 == new_pos_f.0 && player_pos.1 == new_pos_f.1 {
                    let message = monster.attack(&mut self.player);
                    if monster.is_alive() { // Only update if we haven't processed this monster in combat
                        monster.update_last_move(current_time);
                    }
                    drop(monster); // Release the monster borrow before modifying self
                    //self.add_log_message(message);
                    continue;
                } else if !is_collision {
                    monster.x = new_pos_f.0;
                    monster.y = new_pos_f.1;
                }
            }

            monster.update_last_move(current_time);
        }
    }

    fn spawn_items(&mut self, map: &Map) {
        let mut rng = thread_rng();

        for room_row in &map.rooms {
            for room in room_row {
                if rng.gen_bool(0.6) {
                    let (x, y) = room.random_position(&mut rng);
                    let item = match rng.gen_range(0..4) {
                        0 => Item::new_sword(),
                        1 => Item::new_armor(),
                        2 => Item::new_health_potion(),
                        _ => Item::new_lightning_scroll(),
                    };
                    self.ground_items.push((x as f32, y as f32, item));
                }
            }
        }
    }

    fn pick_up_item(&mut self, x: f32, y: f32) -> Option<String> {
        if let Some(index) = self.ground_items
            .iter()
            .position(|(ix, iy, _)| *ix == x && *iy == y)
        {
            let (_, _, item) = self.ground_items.remove(index);
            if let Some(ref mut inventory) = self.player.inventory {
                match inventory.add_item(item.clone()) {
                    Ok(_) => Some(format!("Picked up {}!", item.name)),
                    Err(e) => Some(e),
                }
            } else {
                Some("No inventory available!".to_string())
            }
        } else {
            None
        }
    }

    fn find_closest_monster(&mut self, x: f32, y: f32, max_range: f32) -> Option<&mut Entity> {
        self.monsters
            .iter_mut()
            .filter(|m| m.is_alive())
            .min_by_key(|m| {
                let distance = ((m.x - x).powi(2) + (m.y - y).powi(2)).sqrt();
                if distance <= max_range {
                    (distance * 100.0) as i32
                } else {
                    i32::MAX
                }
            })
    }
}

struct LevelState {
    monsters: Vec<Entity>,
    ground_items: Vec<(f32, f32, Item)>,
}

struct GameConfig {
    map_width: usize,
    map_height: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            map_width: 50,  // Larger map
            map_height: 40, // Larger map
        }
    }
}

#[macroquad::main("Roguelike")]
async fn main() {
    let config = GameConfig::default();
    let mut game_state = GameState::new(config);

    let viewport_width = (screen_width() / TILE_SIZE).floor() as usize;
    let viewport_height = (screen_height() / TILE_SIZE).floor() as usize;
    let mut camera = Camera::new(viewport_width, viewport_height);

    loop {
        let current_time = get_time() as f32;

        if game_state.player.is_alive() && game_state.player.can_move(current_time)  {
            let mut new_x = game_state.player.x;
            let mut new_y = game_state.player.y;
            let mut moved = false;

            if is_key_pressed(KeyCode::W) || is_key_down(KeyCode::W)  {
                new_y -= 1.0;
                moved = true;
            }
            if is_key_pressed(KeyCode::S) || is_key_down(KeyCode::S) {
                new_y += 1.0;
                moved = true;
            }
            if is_key_pressed(KeyCode::A) || is_key_down(KeyCode::A) {
                new_x -= 1.0;
                moved = true;
            }
            if is_key_pressed(KeyCode::D) || is_key_down(KeyCode::D) {
                new_x += 1.0;
                moved = true;
            }

            if moved {
                game_state.player.update_last_move(current_time);
                let mut combat_occurred = false;

                // Check for combat
                for monster in &mut game_state.monsters {
                    if monster.is_alive() && new_x == monster.x && new_y == monster.y {
                        let message = game_state.player.attack(monster);
                        game_state.add_log_message(message);
                        combat_occurred = true;
                        break;
                    }
                }

                // Move if no combat and the tile is walkable
                if !combat_occurred && game_state.map_manager.current_map().is_walkable(new_x as i32, new_y as i32) {
                    game_state.player.x = new_x;
                    game_state.player.y = new_y;

                    // Check for items at the new position
                    game_state.check_and_pickup_items();
                }
            }

            // Check for level transition
            game_state.handle_level_transition();
        }

        game_state.process_monster_turns(current_time);

        // Remove dead monsters
        game_state.monsters.retain(|m| m.is_alive());

        // Update camera to follow player
        camera.follow(
            game_state.player.x,
            game_state.player.y,
            game_state.map_manager.current_map().width,
            game_state.map_manager.current_map().height
        );

        // Clear screen
        clear_background(BLACK);

        // Draw the current map
        game_state.map_manager.current_map().draw(&camera);

        // Draw monsters
        for monster in &game_state.monsters {
            if monster.is_alive() && camera.is_visible(monster.x, monster.y) {
                let (screen_x, screen_y) = camera.world_to_screen(monster.x, monster.y);
                draw_text(
                    &monster.symbol.to_string(),
                    screen_x,
                    screen_y + TILE_SIZE,
                    TILE_SIZE,
                    monster.color,
                );
            }
        }

        // Draw items on ground
        for (x, y, item) in &game_state.ground_items {
            if camera.is_visible(*x, *y) {
                let (screen_x, screen_y) = camera.world_to_screen(*x, *y);
                draw_text(
                    &item.symbol.to_string(),
                    screen_x,
                    screen_y + TILE_SIZE,
                    TILE_SIZE,
                    item.color,
                );
            }
        }

        // Draw the player
        if camera.is_visible(game_state.player.x, game_state.player.y) {
            let (screen_x, screen_y) = camera.world_to_screen(game_state.player.x, game_state.player.y);
            draw_text(
                &game_state.player.symbol.to_string(),
                screen_x,
                screen_y + TILE_SIZE,
                TILE_SIZE,
                game_state.player.color,
            );
        }

        // Draw UI
        draw_rectangle(0.0, 0.0, screen_width(), 30.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_text(
            &format!("HP: {}/{} ATK: {} DEF: {} Level: {}",
                     game_state.player.stats.hp,
                     game_state.player.stats.max_hp,
                     game_state.player.stats.attack,
                     game_state.player.stats.defense,
                     game_state.map_manager.current_level + 1  // Add current level to UI
            ),
            10.0,
            20.0,
            15.0,
            GREEN,
        );

        // Draw combat log
        for (i, message) in game_state.combat_log.iter().enumerate() {
            draw_text(
                message,
                10.0,
                screen_height() - 20.0 * (game_state.combat_log.len() - i) as f32,
                15.0,
                GRAY,
            );
        }

        // Toggle inventory with 'I' key
        if is_key_pressed(KeyCode::I) {
            game_state.inventory_open = !game_state.inventory_open;
        }

        // If inventory is open, draw it
        if game_state.inventory_open {
            game_state.draw_inventory();
            // Close inventory with Escape
            if is_key_pressed(KeyCode::Escape) {
                game_state.inventory_open = false;
            }
        }

        next_frame().await;
    }
}