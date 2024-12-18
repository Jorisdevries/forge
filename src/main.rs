use macroquad::prelude::*;
use ::rand::prelude::*;
use std::collections::HashSet;

const TILE_SIZE: f32 = 20.0;

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
                speed: 5.0,
                last_move: 0.0,
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
            },
            is_player: false,
            inventory: None,
        }
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
    tiles: Vec<char>,
    rooms: Vec<Room>,
}

impl Map {
    fn new(width: usize, height: usize) -> Self {
        let mut map = Map {
            width,
            height,
            tiles: vec!['#'; width * height],
            rooms: Vec::new(),
        };
        map.generate_dungeon();
        map
    }

    fn generate_dungeon(&mut self) {
        let mut rng = thread_rng();
        let max_rooms = 15;
        let min_room_size = 5;
        let max_room_size = 10;

        // Generate rooms
        for _ in 0..max_rooms {
            let w = rng.gen_range(min_room_size..max_room_size);
            let h = rng.gen_range(min_room_size..max_room_size);
            let x = rng.gen_range(1..self.width as i32 - w - 1);
            let y = rng.gen_range(1..self.height as i32 - h - 1);

            let new_room = Room::new(x, y, w, h);

            // Check if room intersects with existing rooms
            let mut intersects = false;
            for other_room in &self.rooms {
                if new_room.intersects(other_room) {
                    intersects = true;
                    break;
                }
            }

            if !intersects {
                self.create_room(&new_room);

                if !self.rooms.is_empty() {
                    // Connect to previous room
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = self.rooms[self.rooms.len() - 1].center();

                    // Randomly choose horizontal or vertical corridor first
                    if rng.gen_bool(0.5) {
                        self.create_horizontal_tunnel(prev_x, new_x, prev_y);
                        self.create_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        self.create_vertical_tunnel(prev_y, new_y, prev_x);
                        self.create_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                self.rooms.push(new_room);
            }
        }

        // No need to store rooms separately as they're now part of the struct
    }

    fn create_room(&mut self, room: &Room) {
        for y in room.y..room.y + room.height {
            for x in room.x..room.x + room.width {
                let idx = y as usize * self.width + x as usize;
                if idx < self.tiles.len() {
                    self.tiles[idx] = '.';
                }
            }
        }
    }

    fn create_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in x1.min(x2)..=x1.max(x2) {
            let idx = y as usize * self.width + x as usize;
            if idx < self.tiles.len() {
                self.tiles[idx] = '.';
            }
        }
    }

    fn create_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in y1.min(y2)..=y1.max(y2) {
            let idx = y as usize * self.width + x as usize;
            if idx < self.tiles.len() {
                self.tiles[idx] = '.';
            }
        }
    }

    fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        let idx = y as usize * self.width + x as usize;
        self.tiles[idx] == '.'
    }

    fn place_monsters(&self, rooms: &[Room]) -> (Option<(f32, f32)>, Vec<(f32, f32)>) {
        let mut monster_positions = Vec::new();
        let mut rng = thread_rng();
        let mut used_positions = HashSet::new();

        // Get player spawn position from first room
        let player_spawn = rooms.first().and_then(|room| {
            let center = room.center();
            if self.is_walkable(center.0, center.1) {
                used_positions.insert((center.0, center.1));
                Some((center.0 as f32, center.1 as f32))
            } else {
                // Fallback: find any walkable tile in the first room
                room.inner_tiles().into_iter()
                    .find(|&(x, y)| self.is_walkable(x, y))
                    .map(|(x, y)| {
                        used_positions.insert((x, y));
                        (x as f32, y as f32)
                    })
            }
        });

        // Skip the first room (player spawn) when placing monsters
        for room in rooms.iter().skip(1) {
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

        (player_spawn, monster_positions)
    }

    fn is_wall(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return true;
        }
        self.tiles[y * self.width + x] == '#'
    }

    fn draw(&self, camera: &Camera) {
        let start_x = camera.x.floor() as usize;
        let start_y = camera.y.floor() as usize;
        let end_x = (camera.x + camera.viewport_width as f32).ceil() as usize;
        let end_y = (camera.y + camera.viewport_height as f32).ceil() as usize;

        // Only draw tiles that are visible in the viewport
        for y in start_y..end_y.min(self.height) {
            for x in start_x..end_x.min(self.width) {
                let tile = self.tiles[y * self.width + x];
                let (screen_x, screen_y) = camera.world_to_screen(x as f32, y as f32);
                draw_text(
                    &tile.to_string(),
                    screen_x,
                    screen_y + TILE_SIZE, // Add TILE_SIZE to account for text height
                    TILE_SIZE,
                    WHITE,
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
    inventory_open: bool,  // Add this line
}

impl GameState {
    fn new() -> Self {
        let mut monsters = Vec::new();

        // Add some initial monsters
        for i in 0..5 {
            monsters.push(Entity::new_monster(10.0 + i as f32 * 2.0, 10.0));
        }

        Self {
            player: Entity::new_player(),
            monsters,
            combat_log: Vec::new(),
            player_turn: true,
            ground_items: Vec::new(),
            inventory_open: false,
        }
    }

    fn spawn_entities(&mut self, map: &Map) {
        let (player_spawn, monster_positions) = map.place_monsters(&map.rooms);

        // Set player position with fallback
        if let Some((x, y)) = player_spawn {
            self.player.x = x;
            self.player.y = y;
        } else {
            // Fallback: find any walkable position
            for y in 0..map.height as i32 {
                for x in 0..map.width as i32 {
                    if map.is_walkable(x, y) {
                        self.player.x = x as f32;
                        self.player.y = y as f32;
                        break;
                    }
                }
            }
        }

        // Spawn monsters only in valid positions
        self.monsters.clear();
        for (x, y) in monster_positions {
            if map.is_walkable(x as i32, y as i32) {
                self.monsters.push(Entity::new_monster(x, y));
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

    fn process_monster_turns(&mut self, map: &Map, current_time: f32) {
        let player_pos = (self.player.x, self.player.y);
        let monster_positions: Vec<(f32, f32)> = self.monsters.iter()
            .filter(|m| m.is_alive())
            .map(|m| (m.x, m.y))
            .collect();

        for i in 0..self.monsters.len() {
            if !self.monsters[i].is_alive() || !self.monsters[i].can_move(current_time) {
                continue;
            }

            let monster = &mut self.monsters[i];
            let dx = player_pos.0 - monster.x;
            let dy = player_pos.1 - monster.y;
            let distance = (dx * dx + dy * dy).sqrt();

            let mut new_x = monster.x;
            let mut new_y = monster.y;

            if distance <= 5.0 {
                // Move towards player if nearby
                new_x += dx.signum();
                new_y += dy.signum();
            } else {
                // Random movement
                let direction = rand::gen_range(0, 4);
                match direction {
                    0 => new_x += 1.0,
                    1 => new_x -= 1.0,
                    2 => new_y += 1.0,
                    _ => new_y -= 1.0,
                }
            }

            // Check for collision with walls or other monsters
            if !map.is_wall(new_x as usize, new_y as usize) {
                let mut can_move = true;

                // Check collision with other monsters using our pre-collected positions
                for (pos_x, pos_y) in &monster_positions {
                    if *pos_x == new_x && *pos_y == new_y && (*pos_x != monster.x || *pos_y != monster.y) {
                        can_move = false;
                        break;
                    }
                }

                // Check collision with player
                if self.player.x == new_x && self.player.y == new_y {
                    let message = monster.attack(&mut self.player);
                    //self.add_log_message(message);
                    can_move = false;
                }

                if can_move {
                    monster.x = new_x;
                    monster.y = new_y;
                }
            }

            monster.update_last_move(current_time);
        }
    }

    fn spawn_items(&mut self, map: &Map) {
        let mut rng = thread_rng();

        for room in &map.rooms {
            // 60% chance to spawn an item in each room
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
    let map = Map::new(config.map_width, config.map_height);
    let mut game_state = GameState::new();
    game_state.spawn_entities(&map);
    game_state.spawn_items(&map);

    let viewport_width = (screen_width() / TILE_SIZE).floor() as usize;
    let viewport_height = (screen_height() / TILE_SIZE).floor() as usize;
    let mut camera = Camera::new(viewport_width, viewport_height);

    loop {
        let current_time = get_time() as f32;

        // In your main game loop where you handle player movement:
        if game_state.player.is_alive() {
            let mut new_x = game_state.player.x;
            let mut new_y = game_state.player.y;
            let mut moved = false;

            if is_key_pressed(KeyCode::W) {
                new_y -= 1.0;
                moved = true;
            }
            if is_key_pressed(KeyCode::S) {
                new_y += 1.0;
                moved = true;
            }
            if is_key_pressed(KeyCode::A) {
                new_x -= 1.0;
                moved = true;
            }
            if is_key_pressed(KeyCode::D) {
                new_x += 1.0;
                moved = true;
            }

            if moved {
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

                // Move if no combat and no wall
                if !combat_occurred && !map.is_wall(new_x as usize, new_y as usize) {
                    game_state.player.x = new_x;
                    game_state.player.y = new_y;

                    // Check for items at the new position
                    game_state.check_and_pickup_items();
                }
            }
        }

        game_state.process_monster_turns(&map, current_time);

        // Remove dead monsters
        game_state.monsters.retain(|m| m.is_alive());

        // Update camera to follow player
        camera.follow(game_state.player.x, game_state.player.y, config.map_width, config.map_height);

        // Clear screen
        clear_background(BLACK);

        // Draw the map
        map.draw(&camera);

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
            &format!("HP: {}/{} ATK: {} DEF: {}",
                     game_state.player.stats.hp,
                     game_state.player.stats.max_hp,
                     game_state.player.stats.attack,
                     game_state.player.stats.defense
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