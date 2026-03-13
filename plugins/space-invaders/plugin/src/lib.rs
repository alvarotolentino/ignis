wit_bindgen::generate!({
    path: "wit",
    world: "ignis-game",
});

struct SpaceInvaders;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Virtual screen dimensions (must match ignis.toml).
const SCREEN_W: f32 = 320.0;
const SCREEN_H: f32 = 240.0;

// Grid layout
const INVADER_COLS: usize = 11;
const INVADER_ROWS: usize = 5;
const INVADER_W: f32 = 12.0;
const INVADER_H: f32 = 8.0;
const INVADER_PAD_X: f32 = 6.0;
const INVADER_PAD_Y: f32 = 6.0;
const INVADER_GRID_LEFT: f32 = 20.0;
const INVADER_GRID_TOP: f32 = 30.0;

// Player
const PLAYER_W: f32 = 16.0;
const PLAYER_H: f32 = 8.0;
const PLAYER_Y: f32 = SCREEN_H - 20.0;
const PLAYER_SPEED: f32 = 120.0; // px/sec

// Bullets
const PLAYER_BULLET_SPEED: f32 = 200.0; // px/sec (upward)
const INVADER_BULLET_SPEED: f32 = 100.0; // px/sec (downward)
const BULLET_W: f32 = 2.0;
const BULLET_H: f32 = 6.0;

// Invader movement
const INVADER_STEP_X: f32 = 4.0;
const INVADER_DROP_Y: f32 = 8.0;
const INVADER_STEP_INITIAL_MS: u32 = 500;
const INVADER_STEP_MIN_MS: u32 = 60;
/// Probability per bottom-row invader per tick (out of 1000).
const INVADER_SHOOT_CHANCE: u32 = 10; // ~1%

// Input action IDs (must match engine InputAction enum)
const ACTION_UP: u32 = 0;
const ACTION_DOWN: u32 = 1;
const ACTION_LEFT: u32 = 2;
const ACTION_RIGHT: u32 = 3;
const ACTION_A: u32 = 4;
const ACTION_START: u32 = 6;

// Colors (RGBA in big-endian u32)
const COLOR_BLACK: u32 = 0x000000FF;
const COLOR_GREEN: u32 = 0x00FF00FF;
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_RED: u32 = 0xFF0000FF;
const COLOR_YELLOW: u32 = 0xFFFF00FF;
const COLOR_CYAN: u32 = 0x00FFFFFF;
const COLOR_DARK_GREY: u32 = 0x222222FF;
const COLOR_ORANGE: u32 = 0xFF8800FF;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    GameOver,
    Won,
}

#[derive(Clone, Copy)]
struct Player {
    x: f32,
    lives: u8,
    score: u32,
}

#[derive(Clone, Copy)]
struct Invader {
    alive: bool,
    kind: u8, // 0..2 determines colour
}

#[derive(Clone, Copy)]
struct Bullet {
    x: f32,
    y: f32,
    active: bool,
    /// true = moving up (player), false = moving down (invader)
    going_up: bool,
}

/// Minimal xorshift32 PRNG — no allocations, no deps.
struct Rng {
    state: u32,
}

impl Rng {
    fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }
    /// Returns a u32 in [0, max) where max > 0.
    fn next_u32(&mut self, max: u32) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state % max
    }
}

// ---------------------------------------------------------------------------
// Game State (global mutable — single threaded WASM)
// ---------------------------------------------------------------------------

static mut STATE: Option<Game> = None;

struct Game {
    state: GameState,
    player: Player,
    invaders: [[Invader; INVADER_COLS]; INVADER_ROWS],
    /// Offset of the entire invader grid from the initial position.
    grid_offset_x: f32,
    grid_offset_y: f32,
    /// Current horizontal step direction: +1 or -1.
    grid_dir: f32,
    /// Milliseconds accumulated for invader step timing.
    invader_step_timer_ms: u32,
    /// Current step interval (decreases as invaders die).
    invader_step_interval_ms: u32,
    /// Player bullet (max 1 active).
    player_bullet: Bullet,
    /// Invader bullets (up to 4 active).
    invader_bullets: [Bullet; 4],
    rng: Rng,
    /// Accumulated ms since init (used only for PRNG seeding on first tick).
    seeded: bool,
    /// Input flags set each frame.
    input_left: bool,
    input_right: bool,
    input_fire: bool,
    input_start: bool,
}

impl Game {
    fn new() -> Self {
        let mut invaders = [[Invader { alive: true, kind: 0 }; INVADER_COLS]; INVADER_ROWS];
        for (row_idx, row) in invaders.iter_mut().enumerate() {
            let kind = match row_idx {
                0 => 0,       // top row
                1 | 2 => 1,   // middle rows
                _ => 2,       // bottom rows
            };
            for inv in row.iter_mut() {
                inv.kind = kind;
            }
        }

        Self {
            state: GameState::Playing,
            player: Player {
                x: SCREEN_W / 2.0 - PLAYER_W / 2.0,
                lives: 3,
                score: 0,
            },
            invaders,
            grid_offset_x: 0.0,
            grid_offset_y: 0.0,
            grid_dir: 1.0,
            invader_step_timer_ms: 0,
            invader_step_interval_ms: INVADER_STEP_INITIAL_MS,
            player_bullet: Bullet { x: 0.0, y: 0.0, active: false, going_up: true },
            invader_bullets: [Bullet { x: 0.0, y: 0.0, active: false, going_up: false }; 4],
            rng: Rng::new(12345),
            seeded: false,
            input_left: false,
            input_right: false,
            input_fire: false,
            input_start: false,
        }
    }

    fn reset(&mut self) {
        *self = Game::new();
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn alive_count(&self) -> u32 {
        let mut count = 0u32;
        for row in &self.invaders {
            for inv in row {
                if inv.alive {
                    count += 1;
                }
            }
        }
        count
    }

    fn invader_world_pos(&self, row: usize, col: usize) -> (f32, f32) {
        let x = INVADER_GRID_LEFT
            + col as f32 * (INVADER_W + INVADER_PAD_X)
            + self.grid_offset_x;
        let y = INVADER_GRID_TOP
            + row as f32 * (INVADER_H + INVADER_PAD_Y)
            + self.grid_offset_y;
        (x, y)
    }

    fn invader_color(kind: u8) -> u32 {
        match kind {
            0 => COLOR_RED,
            1 => COLOR_YELLOW,
            _ => COLOR_CYAN,
        }
    }

    /// Returns the bottom-most alive row index for a given column, if any.
    fn bottom_alive_in_col(&self, col: usize) -> Option<usize> {
        for row in (0..INVADER_ROWS).rev() {
            if self.invaders[row][col].alive {
                return Some(row);
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Update steps
    // -----------------------------------------------------------------------

    fn update_playing(&mut self, delta_ms: u32) {
        // Seed PRNG from first delta_ms for slight randomness per session
        if !self.seeded {
            self.rng = Rng::new(delta_ms.wrapping_mul(7919).wrapping_add(42));
            self.seeded = true;
        }

        let dt = delta_ms as f32 / 1000.0;

        // --- Player movement ---
        if self.input_left {
            self.player.x -= PLAYER_SPEED * dt;
        }
        if self.input_right {
            self.player.x += PLAYER_SPEED * dt;
        }
        self.player.x = self.player.x.clamp(0.0, SCREEN_W - PLAYER_W);

        // --- Player fire ---
        if self.input_fire && !self.player_bullet.active {
            self.player_bullet = Bullet {
                x: self.player.x + PLAYER_W / 2.0 - BULLET_W / 2.0,
                y: PLAYER_Y - BULLET_H,
                active: true,
                going_up: true,
            };
        }

        // Clear per-frame input flags
        self.input_left = false;
        self.input_right = false;
        self.input_fire = false;
        self.input_start = false;

        // --- Move player bullet ---
        if self.player_bullet.active {
            self.player_bullet.y -= PLAYER_BULLET_SPEED * dt;
            if self.player_bullet.y + BULLET_H < 0.0 {
                self.player_bullet.active = false;
            }
        }

        // --- Move invader bullets ---
        for bullet in &mut self.invader_bullets {
            if bullet.active {
                bullet.y += INVADER_BULLET_SPEED * dt;
                if bullet.y > SCREEN_H {
                    bullet.active = false;
                }
            }
        }

        // --- Invader grid movement (step-based) ---
        self.invader_step_timer_ms += delta_ms;

        // Recalculate step interval based on alive count
        let alive = self.alive_count();
        let total = (INVADER_ROWS * INVADER_COLS) as u32;
        if total > 0 && alive > 0 {
            // Linearly decrease interval as invaders die
            let ratio = alive as f32 / total as f32;
            let interval = INVADER_STEP_MIN_MS as f32
                + ratio * (INVADER_STEP_INITIAL_MS - INVADER_STEP_MIN_MS) as f32;
            self.invader_step_interval_ms = interval as u32;
        }

        if self.invader_step_timer_ms >= self.invader_step_interval_ms {
            self.invader_step_timer_ms = 0;

            // Calculate grid extents to check if we need to reverse direction
            let mut min_col = INVADER_COLS;
            let mut max_col = 0;
            for row in &self.invaders {
                for (c, inv) in row.iter().enumerate() {
                    if inv.alive {
                        if c < min_col { min_col = c; }
                        if c > max_col { max_col = c; }
                    }
                }
            }

            if min_col <= max_col {
                let left_edge = INVADER_GRID_LEFT
                    + min_col as f32 * (INVADER_W + INVADER_PAD_X)
                    + self.grid_offset_x
                    + self.grid_dir * INVADER_STEP_X;
                let right_edge = INVADER_GRID_LEFT
                    + max_col as f32 * (INVADER_W + INVADER_PAD_X)
                    + INVADER_W
                    + self.grid_offset_x
                    + self.grid_dir * INVADER_STEP_X;

                if left_edge < 0.0 || right_edge > SCREEN_W {
                    // Reverse direction and drop
                    self.grid_dir = -self.grid_dir;
                    self.grid_offset_y += INVADER_DROP_Y;
                } else {
                    self.grid_offset_x += self.grid_dir * INVADER_STEP_X;
                }
            }
        }

        // --- Invader shooting ---
        // Pre-compute which columns will shoot (avoids borrow conflict with rng + bullets)
        let mut shots_to_fire: [(usize, usize); INVADER_COLS] = [(0, 0); INVADER_COLS];
        let mut shot_count = 0usize;
        for col in 0..INVADER_COLS {
            if let Some(row) = self.bottom_alive_in_col(col) {
                if self.rng.next_u32(1000) < INVADER_SHOOT_CHANCE {
                    shots_to_fire[shot_count] = (row, col);
                    shot_count += 1;
                }
            }
        }
        for i in 0..shot_count {
            let (row, col) = shots_to_fire[i];
            let (ix, iy) = self.invader_world_pos(row, col);
            if let Some(slot) = self.invader_bullets.iter_mut().find(|b| !b.active) {
                *slot = Bullet {
                    x: ix + INVADER_W / 2.0 - BULLET_W / 2.0,
                    y: iy + INVADER_H,
                    active: true,
                    going_up: false,
                };
            }
        }

        // --- Collision: player bullet → invaders ---
        if self.player_bullet.active {
            let bx = self.player_bullet.x;
            let by = self.player_bullet.y;
            'outer: for row in 0..INVADER_ROWS {
                for col in 0..INVADER_COLS {
                    if !self.invaders[row][col].alive {
                        continue;
                    }
                    let (ix, iy) = self.invader_world_pos(row, col);
                    if aabb_overlap(bx, by, BULLET_W, BULLET_H, ix, iy, INVADER_W, INVADER_H) {
                        self.invaders[row][col].alive = false;
                        self.player_bullet.active = false;
                        let points = match self.invaders[row][col].kind {
                            0 => 30,
                            1 => 20,
                            _ => 10,
                        };
                        self.player.score += points;
                        break 'outer;
                    }
                }
            }
        }

        // --- Collision: invader bullets → player ---
        for bullet in &mut self.invader_bullets {
            if !bullet.active {
                continue;
            }
            if aabb_overlap(
                bullet.x, bullet.y, BULLET_W, BULLET_H,
                self.player.x, PLAYER_Y, PLAYER_W, PLAYER_H,
            ) {
                bullet.active = false;
                if self.player.lives > 0 {
                    self.player.lives -= 1;
                }
                if self.player.lives == 0 {
                    self.state = GameState::GameOver;
                    // Persist last score
                    ignis::game::host_api::set_storage(
                        "last_score",
                        &self.player.score.to_string(),
                    );
                    return;
                }
            }
        }

        // --- Check invader reached player line ---
        for row in 0..INVADER_ROWS {
            for col in 0..INVADER_COLS {
                if self.invaders[row][col].alive {
                    let (_ix, iy) = self.invader_world_pos(row, col);
                    if iy + INVADER_H >= PLAYER_Y {
                        self.state = GameState::GameOver;
                        ignis::game::host_api::set_storage(
                            "last_score",
                            &self.player.score.to_string(),
                        );
                        return;
                    }
                }
            }
        }

        // --- Check win ---
        if self.alive_count() == 0 {
            self.state = GameState::Won;
            ignis::game::host_api::set_storage(
                "last_score",
                &self.player.score.to_string(),
            );
        }
    }

    // -----------------------------------------------------------------------
    // Rendering
    // -----------------------------------------------------------------------

    fn render(&self) {
        // Background
        ignis::game::host_api::draw_rect(0.0, 0.0, SCREEN_W, SCREEN_H, COLOR_BLACK);

        // Invaders
        for row in 0..INVADER_ROWS {
            for col in 0..INVADER_COLS {
                let inv = &self.invaders[row][col];
                if !inv.alive {
                    continue;
                }
                let (x, y) = self.invader_world_pos(row, col);
                ignis::game::host_api::draw_rect(x, y, INVADER_W, INVADER_H, Game::invader_color(inv.kind));
            }
        }

        // Player ship
        ignis::game::host_api::draw_rect(self.player.x, PLAYER_Y, PLAYER_W, PLAYER_H, COLOR_GREEN);

        // Player bullet
        if self.player_bullet.active {
            ignis::game::host_api::draw_rect(
                self.player_bullet.x,
                self.player_bullet.y,
                BULLET_W,
                BULLET_H,
                COLOR_WHITE,
            );
        }

        // Invader bullets
        for bullet in &self.invader_bullets {
            if bullet.active {
                ignis::game::host_api::draw_rect(
                    bullet.x,
                    bullet.y,
                    BULLET_W,
                    BULLET_H,
                    COLOR_ORANGE,
                );
            }
        }

        // HUD: Score (top-left)
        let score_text = format!("SCORE: {}", self.player.score);
        ignis::game::host_api::draw_text(&score_text, 4.0, 4.0, 8);

        // HUD: Lives (top-right)
        let lives_text = format!("LIVES: {}", self.player.lives);
        ignis::game::host_api::draw_text(&lives_text, SCREEN_W - 70.0, 4.0, 8);

        // Overlays
        match self.state {
            GameState::GameOver => {
                // Semi-transparent overlay
                ignis::game::host_api::draw_rect(0.0, 80.0, SCREEN_W, 80.0, 0x00000099);
                ignis::game::host_api::draw_text("GAME OVER", 100.0, 100.0, 20);
                ignis::game::host_api::draw_text("PRESS START", 108.0, 130.0, 10);
            }
            GameState::Won => {
                ignis::game::host_api::draw_rect(0.0, 80.0, SCREEN_W, 80.0, 0x00000099);
                ignis::game::host_api::draw_text("YOU WIN!", 110.0, 100.0, 20);
                ignis::game::host_api::draw_text("PRESS START", 108.0, 130.0, 10);
            }
            GameState::Playing => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Axis-aligned bounding box overlap check
// ---------------------------------------------------------------------------

fn aabb_overlap(
    ax: f32, ay: f32, aw: f32, ah: f32,
    bx: f32, by: f32, bw: f32, bh: f32,
) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

// ---------------------------------------------------------------------------
// WIT Guest Implementation
// ---------------------------------------------------------------------------

impl Guest for SpaceInvaders {
    fn init() {
        unsafe { STATE = Some(Game::new()); }
    }

    fn update(delta_ms: u32) {
        let game = unsafe { STATE.as_mut().expect("init not called") };

        match game.state {
            GameState::Playing => game.update_playing(delta_ms),
            GameState::GameOver | GameState::Won => {
                // Wait for Start to reset
                if game.input_start {
                    game.reset();
                }
            }
        }

        game.render();
    }

    fn handle_input(action: u32) {
        let game = unsafe { STATE.as_mut().expect("init not called") };
        match action {
            ACTION_LEFT => game.input_left = true,
            ACTION_RIGHT => game.input_right = true,
            ACTION_A | ACTION_UP => game.input_fire = true,
            ACTION_START => game.input_start = true,
            _ => {}
        }
    }

    fn get_name() -> String {
        "Space Invaders".to_string()
    }

    fn get_version() -> String {
        "1.0.0".to_string()
    }

    fn get_author() -> String {
        "Ignis Team".to_string()
    }
}

export!(SpaceInvaders);
