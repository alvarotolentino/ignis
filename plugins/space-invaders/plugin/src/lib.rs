wit_bindgen::generate!({
    path: "wit",
    world: "ignis-game",
});

struct SpaceInvaders;

// ===========================================================================
//  Constants
// ===========================================================================

const SCREEN_W: f32 = 960.0;
const SCREEN_H: f32 = 720.0;

// --- Playfield (FR-PF-002: centered within 960x720) ---
const PLAY_W: f32 = 800.0;
const PLAY_H: f32 = 600.0;
const PLAY_X: f32 = (SCREEN_W - PLAY_W) / 2.0;
const PLAY_Y: f32 = (SCREEN_H - PLAY_H) / 2.0 + 20.0; // +20 for HUD at top

// --- Formation ---
const INVADER_COLS: usize = 11;
const INVADER_ROWS: usize = 5;
const INVADER_W: f32 = 36.0;
const INVADER_H: f32 = 24.0;
const INVADER_PAD_X: f32 = 16.0;
const INVADER_PAD_Y: f32 = 16.0;
const FORMATION_W: f32 = INVADER_COLS as f32 * INVADER_W + (INVADER_COLS - 1) as f32 * INVADER_PAD_X;
const FORMATION_START_X: f32 = PLAY_X + (PLAY_W - FORMATION_W) / 2.0;
const FORMATION_START_Y: f32 = PLAY_Y + 60.0;

// --- Player / Cannon ---
const PLAYER_W: f32 = 48.0;
const PLAYER_H: f32 = 24.0;
const PLAYER_Y: f32 = PLAY_Y + PLAY_H - 40.0;
const PLAYER_SPEED: f32 = 400.0; // units/sec (FR-PL-002 scaled to 960)

// --- Kill Line (FR-PF-004: ~80 units above cannon) ---
const KILL_LINE_Y: f32 = PLAYER_Y - 80.0;

// --- Projectiles ---
const LASER_W: f32 = 4.0;
const LASER_H: f32 = 18.0;
const LASER_SPEED: f32 = 720.0; // upward (FR-PL-005 scaled)
const SALVO_W: f32 = 4.0;
const SALVO_H: f32 = 18.0;
const SALVO_SPEED: f32 = 300.0; // downward (FR-EN-009 scaled)

// --- Bunkers ---
const BUNKER_COUNT: usize = 4;
const BUNKER_SEG_COLS: usize = 6;
const BUNKER_SEG_ROWS: usize = 4;
const BUNKER_SEG_W: f32 = 12.0;
const BUNKER_SEG_H: f32 = 10.0;
const BUNKER_Y: f32 = PLAYER_Y - 100.0;

// --- UFO ---
const UFO_W: f32 = 48.0;
const UFO_H: f32 = 20.0;
const UFO_Y: f32 = PLAY_Y + 10.0;
const UFO_SPEED: f32 = 225.0; // FR-UFO-001 scaled
const UFO_MIN_INTERVAL_MS: u32 = 15000;
const UFO_MAX_INTERVAL_MS: u32 = 30000;
const UFO_POINT_VALUES: [u32; 5] = [50, 100, 150, 200, 300];

// --- Invader movement ---
const INVADER_STEP_X: f32 = 10.0;
const INVADER_DROP_Y: f32 = 16.0; // FR-EN-004 scaled
const INVADER_STEP_INITIAL_MS: u32 = 500;
const INVADER_STEP_MIN_MS: u32 = 50;

// --- Timing ---
const RESPAWN_DELAY_MS: u32 = 2000;
const INVINCIBILITY_MS: u32 = 1500;
const WAVE_CLEAR_DELAY_MS: u32 = 2000;

const MAX_INVADER_SALVOS: usize = 4;

// --- Input (press/release model) ---
const ACTION_UP: u32 = 0;
const ACTION_LEFT: u32 = 2;
const ACTION_RIGHT: u32 = 3;
const ACTION_A: u32 = 4;
const ACTION_START: u32 = 6;
const RELEASE_OFFSET: u32 = 8;

// --- Colours (0xRRGGBBAA) ---
const COLOR_BG: u32 = 0x000000FF;
const COLOR_PLAY_BG: u32 = 0x0A0A1AFF;
const COLOR_HUD_BG: u32 = 0x111122FF;
const COLOR_GREEN: u32 = 0x00FF41FF;
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_RED: u32 = 0xFF2222FF;
const COLOR_CYAN: u32 = 0x00FFFFFF;
const COLOR_MAGENTA: u32 = 0xFF00FFFF;
const COLOR_ORANGE: u32 = 0xFF7700FF;
const COLOR_BUNKER: u32 = 0x00AA33FF;
const COLOR_OVERLAY: u32 = 0x000000BF;
const COLOR_KILL_LINE: u32 = 0xFF000044;

// ===========================================================================
//  Types
// ===========================================================================

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    Respawning,
    WaveClear,
    GameOver,
}

#[derive(Clone, Copy)]
struct Invader {
    alive: bool,
    kind: u8,
}

#[derive(Clone, Copy)]
struct Bullet {
    x: f32,
    y: f32,
    active: bool,
}

#[derive(Clone, Copy)]
struct Ufo {
    active: bool,
    x: f32,
    dir: f32,
}

#[derive(Clone, Copy)]
struct ScorePop {
    x: f32,
    y: f32,
    value: u32,
    timer_ms: u32,
    active: bool,
}

struct Rng(u32);

impl Rng {
    fn new(seed: u32) -> Self {
        Self(if seed == 0 { 0xDEAD_BEEF } else { seed })
    }
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    fn range(&mut self, n: u32) -> u32 {
        self.next_u32() % n
    }
}

type BunkerGrid = [[u8; BUNKER_SEG_COLS]; BUNKER_SEG_ROWS];

// ===========================================================================
//  Game
// ===========================================================================

static mut STATE: Option<Game> = None;

struct Game {
    state: GameState,

    // Player
    player_x: f32,
    lives: u8,
    score: u32,
    high_score: u32,

    // Formation
    invaders: [[Invader; INVADER_COLS]; INVADER_ROWS],
    grid_offset_x: f32,
    grid_offset_y: f32,
    grid_dir: f32,
    step_timer_ms: u32,
    step_interval_ms: u32,
    anim_frame: bool,

    // Projectiles
    player_laser: Bullet,
    salvos: [Bullet; MAX_INVADER_SALVOS],

    // Bunkers
    bunkers: [BunkerGrid; BUNKER_COUNT],

    // UFO
    ufo: Ufo,
    ufo_timer_ms: u32,
    ufo_next_interval: u32,
    ufo_dir_toggle: bool,
    shots_fired: u32,

    // Score popups
    pops: [ScorePop; 4],

    // Wave
    wave: u32,
    wave_clear_timer_ms: u32,
    extra_life_1k: bool,
    extra_life_5k: bool,

    // Respawn
    respawn_timer_ms: u32,
    invincibility_ms: u32,
    was_hit: bool,

    // Input (held state via press/release)
    held_left: bool,
    held_right: bool,
    input_fire: bool,
    input_start: bool,

    rng: Rng,
    score_submitted: bool,
}

impl Game {
    fn new() -> Self {
        let mut g = Self {
            state: GameState::Playing,
            player_x: PLAY_X + PLAY_W / 2.0 - PLAYER_W / 2.0,
            lives: 3,
            score: 0,
            high_score: 0,
            invaders: [[Invader { alive: false, kind: 0 }; INVADER_COLS]; INVADER_ROWS],
            grid_offset_x: 0.0,
            grid_offset_y: 0.0,
            grid_dir: 1.0,
            step_timer_ms: 0,
            step_interval_ms: INVADER_STEP_INITIAL_MS,
            anim_frame: false,
            player_laser: Bullet { x: 0.0, y: 0.0, active: false },
            salvos: [Bullet { x: 0.0, y: 0.0, active: false }; MAX_INVADER_SALVOS],
            bunkers: [[[0u8; BUNKER_SEG_COLS]; BUNKER_SEG_ROWS]; BUNKER_COUNT],
            ufo: Ufo { active: false, x: 0.0, dir: 1.0 },
            ufo_timer_ms: 0,
            ufo_next_interval: UFO_MAX_INTERVAL_MS,
            ufo_dir_toggle: false,
            shots_fired: 0,
            pops: [ScorePop { x: 0.0, y: 0.0, value: 0, timer_ms: 0, active: false }; 4],
            wave: 1,
            wave_clear_timer_ms: 0,
            extra_life_1k: false,
            extra_life_5k: false,
            respawn_timer_ms: 0,
            invincibility_ms: 0,
            was_hit: false,
            held_left: false,
            held_right: false,
            input_fire: false,
            input_start: false,
            rng: Rng::new(42),
            score_submitted: false,
        };
        g.init_formation(1);
        g.init_bunkers();
        g.load_high_score();
        g
    }

    fn reset(&mut self) {
        let hs = self.high_score;
        *self = Self::new();
        self.high_score = hs;
    }

    // -----------------------------------------------------------------------
    //  Initialisation helpers
    // -----------------------------------------------------------------------

    fn init_formation(&mut self, wave: u32) {
        for (row_idx, row) in self.invaders.iter_mut().enumerate() {
            let kind: u8 = match row_idx {
                0 => 2,       // Type C (top)
                1 | 2 => 1,   // Type B (middle)
                _ => 0,       // Type A (bottom)
            };
            for inv in row.iter_mut() {
                *inv = Invader { alive: true, kind };
            }
        }
        self.grid_offset_x = 0.0;
        // FR-WV-001: formation starts lower per wave (max 5 increments)
        let descent = ((wave.saturating_sub(1)).min(5) as f32) * 16.0;
        self.grid_offset_y = descent;
        self.grid_dir = 1.0;
        self.step_timer_ms = 0;
        self.step_interval_ms = INVADER_STEP_INITIAL_MS;
        self.anim_frame = false;
    }

    fn init_bunkers(&mut self) {
        for bunker in &mut self.bunkers {
            for (row, segs) in bunker.iter_mut().enumerate() {
                for (col, seg) in segs.iter_mut().enumerate() {
                    let is_arch_gap = row >= 2 && (2..=3).contains(&col);
                    *seg = if is_arch_gap { 0 } else { 3 };
                }
            }
        }
    }

    fn load_high_score(&mut self) {
        if let Some(val) = crate::ignis::game::host_api::get_storage("high_score") {
            self.high_score = val.parse().unwrap_or(0);
        }
    }

    // -----------------------------------------------------------------------
    //  Helpers
    // -----------------------------------------------------------------------

    fn alive_count(&self) -> u32 {
        self.invaders.iter()
            .flat_map(|r| r.iter())
            .filter(|i| i.alive)
            .count() as u32
    }

    fn invader_pos(&self, row: usize, col: usize) -> (f32, f32) {
        let x = FORMATION_START_X + col as f32 * (INVADER_W + INVADER_PAD_X) + self.grid_offset_x;
        let y = FORMATION_START_Y + row as f32 * (INVADER_H + INVADER_PAD_Y) + self.grid_offset_y;
        (x, y)
    }

    fn invader_color(kind: u8) -> u32 {
        match kind {
            0 => COLOR_GREEN,    // Type A
            1 => COLOR_ORANGE,   // Type B
            _ => COLOR_MAGENTA,  // Type C
        }
    }

    fn invader_points(kind: u8) -> u32 {
        match kind {
            0 => 10,
            1 => 20,
            _ => 30,
        }
    }

    fn bottom_alive_in_col(&self, col: usize) -> Option<usize> {
        (0..INVADER_ROWS).rev().find(|&r| self.invaders[r][col].alive)
    }

    fn add_pop(&mut self, x: f32, y: f32, value: u32) {
        if let Some(p) = self.pops.iter_mut().find(|p| !p.active) {
            *p = ScorePop { x, y, value, timer_ms: 0, active: true };
        }
    }

    fn check_extra_lives(&mut self) {
        if !self.extra_life_1k && self.score >= 1000 {
            self.extra_life_1k = true;
            if self.lives < 5 { self.lives += 1; }
        }
        if !self.extra_life_5k && self.score >= 5000 {
            self.extra_life_5k = true;
            if self.lives < 5 { self.lives += 1; }
        }
    }

    fn fire_interval_ms(&self) -> u32 {
        let base = 1500u32.saturating_sub((self.wave - 1) * 200).max(500);
        let alive = self.alive_count();
        let total = (INVADER_ROWS * INVADER_COLS) as u32;
        if alive == 0 { return base; }
        let ratio = alive as f32 / total as f32;
        (base as f32 * ratio).max(200.0) as u32
    }

    fn step_interval(&self) -> u32 {
        let alive = self.alive_count();
        if alive == 0 { return INVADER_STEP_INITIAL_MS; }
        let total = (INVADER_ROWS * INVADER_COLS) as f32;
        let ratio = alive as f32 / total;
        let range = (INVADER_STEP_INITIAL_MS - INVADER_STEP_MIN_MS) as f32;
        let base = if ratio < 0.25 {
            INVADER_STEP_MIN_MS as f32 + (ratio / 0.25).powi(2) * range
        } else {
            INVADER_STEP_MIN_MS as f32 + ratio * range
        };
        (base / (1.0 + (self.wave - 1) as f32 * 0.15)).max(INVADER_STEP_MIN_MS as f32) as u32
    }

    /// Advance salvos and formation — shared by Playing and Respawning states.
    fn tick_world(&mut self, delta_ms: u32) {
        let dt = delta_ms as f32 / 1000.0;
        for s in &mut self.salvos {
            if s.active {
                s.y += SALVO_SPEED * dt;
                if s.y > PLAY_Y + PLAY_H { s.active = false; }
            }
        }
        self.step_interval_ms = self.step_interval();
        self.step_timer_ms += delta_ms;
        if self.step_timer_ms >= self.step_interval_ms {
            self.step_timer_ms = 0;
            self.anim_frame = !self.anim_frame;
            self.step_formation();
        }
    }

    // -----------------------------------------------------------------------
    //  Update — Playing
    // -----------------------------------------------------------------------

    fn update_playing(&mut self, delta_ms: u32) {
        let dt = delta_ms as f32 / 1000.0;

        // --- Player movement (continuous via held keys) ---
        if self.held_left {
            self.player_x -= PLAYER_SPEED * dt;
        }
        if self.held_right {
            self.player_x += PLAYER_SPEED * dt;
        }
        self.player_x = self.player_x.clamp(PLAY_X, PLAY_X + PLAY_W - PLAYER_W);

        // --- Invincibility countdown ---
        if self.invincibility_ms > 0 {
            self.invincibility_ms = self.invincibility_ms.saturating_sub(delta_ms);
        }

        // --- Player fire (one laser at a time: FR-PL-004) ---
        if self.input_fire && !self.player_laser.active {
            self.player_laser = Bullet {
                x: self.player_x + PLAYER_W / 2.0 - LASER_W / 2.0,
                y: PLAYER_Y - LASER_H,
                active: true,
            };
            self.shots_fired += 1;
        }
        self.input_fire = false;

        // --- Move player laser ---
        if self.player_laser.active {
            self.player_laser.y -= LASER_SPEED * dt;
            if self.player_laser.y + LASER_H < PLAY_Y {
                self.player_laser.active = false;
            }
        }

        // --- Salvos + formation ---
        self.tick_world(delta_ms);

        // --- Invader firing ---
        self.try_invader_fire();

        // --- UFO ---
        self.update_ufo(delta_ms);

        // --- Collisions ---
        self.collide_laser_invaders();
        self.collide_laser_ufo();
        self.collide_laser_bunkers();
        self.collide_salvos_player();
        self.collide_salvos_bunkers();
        self.collide_laser_salvos();
        self.collide_invaders_bunkers();

        // --- Kill Line check (FR-LF-006) ---
        if self.check_kill_line() {
            self.state = GameState::GameOver;
            self.submit_score();
            return;
        }

        // --- Extra lives ---
        self.check_extra_lives();

        // --- Wave cleared? ---
        if self.alive_count() == 0 {
            self.score += 500 * self.wave;
            if !self.was_hit {
                self.score += 1000;
            }
            self.update_high_score();
            self.state = GameState::WaveClear;
            self.wave_clear_timer_ms = 0;
        }

        // --- Score popups ---
        for p in &mut self.pops {
            if p.active {
                p.timer_ms += delta_ms;
                p.y -= 40.0 * dt;
                if p.timer_ms >= 800 {
                    p.active = false;
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    //  Formation movement
    // -----------------------------------------------------------------------

    fn step_formation(&mut self) {
        let (mut min_col, mut max_col) = (INVADER_COLS, 0);
        for row in &self.invaders {
            for (c, inv) in row.iter().enumerate() {
                if inv.alive {
                    if c < min_col { min_col = c; }
                    if c > max_col { max_col = c; }
                }
            }
        }
        if min_col > max_col { return; }

        let left = FORMATION_START_X + min_col as f32 * (INVADER_W + INVADER_PAD_X)
            + self.grid_offset_x + self.grid_dir * INVADER_STEP_X;
        let right = FORMATION_START_X + max_col as f32 * (INVADER_W + INVADER_PAD_X) + INVADER_W
            + self.grid_offset_x + self.grid_dir * INVADER_STEP_X;

        if left < PLAY_X || right > PLAY_X + PLAY_W {
            self.grid_dir = -self.grid_dir;
            self.grid_offset_y += INVADER_DROP_Y;
        } else {
            self.grid_offset_x += self.grid_dir * INVADER_STEP_X;
        }
    }

    // -----------------------------------------------------------------------
    //  Invader firing
    // -----------------------------------------------------------------------

    fn try_invader_fire(&mut self) {
        let max_salvos = (self.wave as usize).min(MAX_INVADER_SALVOS);
        let active_salvos = self.salvos.iter().filter(|s| s.active).count();
        if active_salvos >= max_salvos { return; }

        let interval = self.fire_interval_ms();
        let chance = (16000 / interval.max(1)).max(5);

        for col in 0..INVADER_COLS {
            if let Some(row) = self.bottom_alive_in_col(col) {
                if self.rng.range(1000) < chance {
                    let (ix, iy) = self.invader_pos(row, col);
                    if let Some(slot) = self.salvos.iter_mut().find(|s| !s.active) {
                        *slot = Bullet {
                            x: ix + INVADER_W / 2.0 - SALVO_W / 2.0,
                            y: iy + INVADER_H,
                            active: true,
                        };
                    }
                    return;
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    //  UFO
    // -----------------------------------------------------------------------

    fn update_ufo(&mut self, delta_ms: u32) {
        let dt = delta_ms as f32 / 1000.0;
        if self.ufo.active {
            self.ufo.x += UFO_SPEED * self.ufo.dir * dt;
            if self.ufo.x < PLAY_X - UFO_W || self.ufo.x > PLAY_X + PLAY_W + UFO_W {
                self.ufo.active = false;
            }
        } else {
            self.ufo_timer_ms += delta_ms;
            if self.ufo_timer_ms >= self.ufo_next_interval {
                self.ufo_timer_ms = 0;
                self.ufo.active = true;
                self.ufo_dir_toggle = !self.ufo_dir_toggle;
                if self.ufo_dir_toggle {
                    self.ufo.dir = 1.0;
                    self.ufo.x = PLAY_X - UFO_W;
                } else {
                    self.ufo.dir = -1.0;
                    self.ufo.x = PLAY_X + PLAY_W;
                }
                let min_int = UFO_MIN_INTERVAL_MS.saturating_sub(self.wave.saturating_sub(3) * 5000).max(10000);
                self.ufo_next_interval = min_int + self.rng.range(UFO_MAX_INTERVAL_MS - min_int + 1);
            }
        }
    }

    // -----------------------------------------------------------------------
    //  Collisions
    // -----------------------------------------------------------------------

    fn collide_laser_invaders(&mut self) {
        if !self.player_laser.active { return; }
        let (bx, by) = (self.player_laser.x, self.player_laser.y);
        for row in 0..INVADER_ROWS {
            for col in 0..INVADER_COLS {
                if !self.invaders[row][col].alive { continue; }
                let (ix, iy) = self.invader_pos(row, col);
                if aabb_overlap(bx, by, LASER_W, LASER_H, ix, iy, INVADER_W, INVADER_H) {
                    let kind = self.invaders[row][col].kind;
                    self.invaders[row][col].alive = false;
                    self.player_laser.active = false;
                    let pts = Self::invader_points(kind);
                    self.score += pts;
                    self.add_pop(ix, iy, pts);
                    self.update_high_score();
                    return;
                }
            }
        }
    }

    fn collide_laser_ufo(&mut self) {
        if !self.player_laser.active || !self.ufo.active { return; }
        let (bx, by) = (self.player_laser.x, self.player_laser.y);
        if aabb_overlap(bx, by, LASER_W, LASER_H, self.ufo.x, UFO_Y, UFO_W, UFO_H) {
            self.ufo.active = false;
            self.player_laser.active = false;
            let idx = (self.shots_fired as usize) % UFO_POINT_VALUES.len();
            let pts = UFO_POINT_VALUES[idx];
            self.score += pts;
            self.add_pop(self.ufo.x, UFO_Y, pts);
            self.update_high_score();
        }
    }

    fn collide_laser_bunkers(&mut self) {
        if !self.player_laser.active { return; }
        if hit_bunker_seg(&mut self.bunkers, self.player_laser.x, self.player_laser.y, LASER_W, LASER_H) {
            self.player_laser.active = false;
        }
    }

    fn collide_salvos_player(&mut self) {
        if self.invincibility_ms > 0 { return; }
        for s in &mut self.salvos {
            if !s.active { continue; }
            if aabb_overlap(s.x, s.y, SALVO_W, SALVO_H, self.player_x, PLAYER_Y, PLAYER_W, PLAYER_H) {
                s.active = false;
                self.was_hit = true;
                if self.lives > 1 {
                    self.lives -= 1;
                    self.state = GameState::Respawning;
                    self.respawn_timer_ms = 0;
                } else {
                    self.lives = 0;
                    self.state = GameState::GameOver;
                    self.submit_score();
                }
                return;
            }
        }
    }

    fn collide_salvos_bunkers(&mut self) {
        for s in &mut self.salvos {
            if !s.active { continue; }
            for bi in 0..BUNKER_COUNT {
                let bkx = bunker_x(bi);
                for row in 0..BUNKER_SEG_ROWS {
                    for col in 0..BUNKER_SEG_COLS {
                        if self.bunkers[bi][row][col] == 0 { continue; }
                        let sx = bkx + col as f32 * BUNKER_SEG_W;
                        let sy = BUNKER_Y + row as f32 * BUNKER_SEG_H;
                        if aabb_overlap(s.x, s.y, SALVO_W, SALVO_H, sx, sy, BUNKER_SEG_W, BUNKER_SEG_H) {
                            self.bunkers[bi][row][col] = self.bunkers[bi][row][col].saturating_sub(1);
                            s.active = false;
                        }
                    }
                }
            }
        }
    }

    fn collide_laser_salvos(&mut self) {
        if !self.player_laser.active { return; }
        let (bx, by) = (self.player_laser.x, self.player_laser.y);
        for s in &mut self.salvos {
            if !s.active { continue; }
            if aabb_overlap(bx, by, LASER_W, LASER_H, s.x, s.y, SALVO_W, SALVO_H) {
                self.player_laser.active = false;
                s.active = false;
                return;
            }
        }
    }

    fn collide_invaders_bunkers(&mut self) {
        for row in 0..INVADER_ROWS {
            for col in 0..INVADER_COLS {
                if !self.invaders[row][col].alive { continue; }
                let (ix, iy) = self.invader_pos(row, col);
                for bi in 0..BUNKER_COUNT {
                    let bkx = bunker_x(bi);
                    for br in 0..BUNKER_SEG_ROWS {
                        for bc in 0..BUNKER_SEG_COLS {
                            if self.bunkers[bi][br][bc] == 0 { continue; }
                            let sx = bkx + bc as f32 * BUNKER_SEG_W;
                            let sy = BUNKER_Y + br as f32 * BUNKER_SEG_H;
                            if aabb_overlap(ix, iy, INVADER_W, INVADER_H, sx, sy, BUNKER_SEG_W, BUNKER_SEG_H) {
                                self.bunkers[bi][br][bc] = 0;
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_kill_line(&self) -> bool {
        (0..INVADER_ROWS).any(|r| {
            (0..INVADER_COLS).any(|c| {
                self.invaders[r][c].alive && self.invader_pos(r, c).1 + INVADER_H >= KILL_LINE_Y
            })
        })
    }

    // -----------------------------------------------------------------------
    //  Score persistence
    // -----------------------------------------------------------------------

    fn update_high_score(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
    }

    fn submit_score(&mut self) {
        if self.score_submitted { return; }
        self.score_submitted = true;
        self.update_high_score();
        crate::ignis::game::host_api::set_storage("last_score", &self.score.to_string());
        crate::ignis::game::host_api::set_storage("high_score", &self.high_score.to_string());
    }

    // -----------------------------------------------------------------------
    //  State updates for non-Playing states
    // -----------------------------------------------------------------------

    fn update_respawning(&mut self, delta_ms: u32) {
        self.respawn_timer_ms += delta_ms;
        if self.respawn_timer_ms >= RESPAWN_DELAY_MS {
            self.state = GameState::Playing;
            self.player_x = PLAY_X + PLAY_W / 2.0 - PLAYER_W / 2.0;
            self.invincibility_ms = INVINCIBILITY_MS;
            for s in &mut self.salvos {
                s.active = false;
            }
        }
    }

    fn update_wave_clear(&mut self, delta_ms: u32) {
        self.wave_clear_timer_ms += delta_ms;
        if self.wave_clear_timer_ms >= WAVE_CLEAR_DELAY_MS {
            self.wave += 1;
            self.init_formation(self.wave);
            self.player_laser.active = false;
            for s in &mut self.salvos { s.active = false; }
            self.ufo.active = false;
            self.was_hit = false;
            self.state = GameState::Playing;
        }
    }

    // -----------------------------------------------------------------------
    //  Rendering
    // -----------------------------------------------------------------------

    fn render(&self) {
        use crate::ignis::game::host_api;

        // Full background
        host_api::draw_rect(0.0, 0.0, SCREEN_W, SCREEN_H, COLOR_BG);

        // HUD bar at top
        host_api::draw_rect(0.0, 0.0, SCREEN_W, PLAY_Y - 4.0, COLOR_HUD_BG);
        host_api::draw_text(&format!("SCORE: {}", self.score), 30.0, 14.0, 18);
        host_api::draw_text(&format!("HI: {}", self.high_score), SCREEN_W / 2.0 - 60.0, 14.0, 18);
        host_api::draw_text(&format!("WAVE {}", self.wave), SCREEN_W - 200.0, 14.0, 18);

        // Lives as small cannon icons
        for i in 0..self.lives {
            let lx = SCREEN_W - 100.0 + i as f32 * 22.0;
            host_api::draw_rect(lx, 40.0, 16.0, 8.0, COLOR_GREEN);
        }

        // Playfield background
        host_api::draw_rect(PLAY_X, PLAY_Y, PLAY_W, PLAY_H, COLOR_PLAY_BG);

        // --- Bunkers ---
        for bi in 0..BUNKER_COUNT {
            let bkx = bunker_x(bi);
            for row in 0..BUNKER_SEG_ROWS {
                for col in 0..BUNKER_SEG_COLS {
                    let hp = self.bunkers[bi][row][col];
                    if hp == 0 { continue; }
                    let sx = bkx + col as f32 * BUNKER_SEG_W;
                    let sy = BUNKER_Y + row as f32 * BUNKER_SEG_H;
                    let color = match hp {
                        3 => COLOR_BUNKER,
                        2 => 0x007722FF,
                        _ => 0x004411FF,
                    };
                    host_api::draw_rect(sx, sy, BUNKER_SEG_W - 1.0, BUNKER_SEG_H - 1.0, color);
                }
            }
        }

        // --- Invaders ---
        for row in 0..INVADER_ROWS {
            for col in 0..INVADER_COLS {
                if !self.invaders[row][col].alive { continue; }
                let (x, y) = self.invader_pos(row, col);
                let color = Self::invader_color(self.invaders[row][col].kind);
                host_api::draw_rect(x + 4.0, y, INVADER_W - 8.0, INVADER_H, color);
                if self.anim_frame {
                    host_api::draw_rect(x, y + INVADER_H * 0.3, 4.0, INVADER_H * 0.5, color);
                    host_api::draw_rect(x + INVADER_W - 4.0, y + INVADER_H * 0.3, 4.0, INVADER_H * 0.5, color);
                } else {
                    host_api::draw_rect(x, y + INVADER_H * 0.5, 4.0, INVADER_H * 0.5, color);
                    host_api::draw_rect(x + INVADER_W - 4.0, y, 4.0, INVADER_H * 0.5, color);
                }
            }
        }

        // --- UFO ---
        if self.ufo.active {
            host_api::draw_rect(self.ufo.x, UFO_Y, UFO_W, UFO_H, COLOR_CYAN);
            host_api::draw_rect(self.ufo.x + UFO_W * 0.3, UFO_Y - 6.0, UFO_W * 0.4, 8.0, COLOR_CYAN);
        }

        // --- Player Cannon ---
        let show_player = match self.state {
            GameState::Respawning => false,
            GameState::Playing if self.invincibility_ms > 0 => {
                (self.invincibility_ms / 100).is_multiple_of(2)
            }
            _ => true,
        };
        if show_player && self.state != GameState::GameOver {
            host_api::draw_rect(self.player_x, PLAYER_Y, PLAYER_W, PLAYER_H, COLOR_GREEN);
            host_api::draw_rect(
                self.player_x + PLAYER_W / 2.0 - 3.0,
                PLAYER_Y - 8.0,
                6.0,
                10.0,
                COLOR_GREEN,
            );
        }

        // --- Player laser ---
        if self.player_laser.active {
            host_api::draw_rect(self.player_laser.x, self.player_laser.y, LASER_W, LASER_H, COLOR_WHITE);
        }

        // --- Invader salvos ---
        for s in &self.salvos {
            if s.active {
                host_api::draw_rect(s.x, s.y, SALVO_W, SALVO_H, COLOR_RED);
            }
        }

        // --- Kill Line warning ---
        let danger_y = KILL_LINE_Y - 3.0 * INVADER_DROP_Y;
        let danger = (0..INVADER_ROWS).any(|r| {
            (0..INVADER_COLS).any(|c| {
                self.invaders[r][c].alive && self.invader_pos(r, c).1 + INVADER_H >= danger_y
            })
        });
        if danger {
            host_api::draw_rect(PLAY_X, KILL_LINE_Y, PLAY_W, 2.0, COLOR_KILL_LINE);
        }

        // --- Score popups ---
        for p in &self.pops {
            if p.active {
                host_api::draw_text(&format!("+{}", p.value), p.x, p.y, 14);
            }
        }

        // --- Wave Clear overlay ---
        if self.state == GameState::WaveClear {
            host_api::draw_rect(PLAY_X, PLAY_Y + PLAY_H / 2.0 - 60.0, PLAY_W, 120.0, COLOR_OVERLAY);
            host_api::draw_text(
                &format!("WAVE {} CLEARED!", self.wave),
                SCREEN_W / 2.0 - 120.0,
                PLAY_Y + PLAY_H / 2.0 - 30.0,
                22,
            );
            host_api::draw_text(
                &format!("Score: {}", self.score),
                SCREEN_W / 2.0 - 60.0,
                PLAY_Y + PLAY_H / 2.0 + 10.0,
                16,
            );
        }

        // --- Game Over overlay ---
        if self.state == GameState::GameOver {
            host_api::draw_rect(PLAY_X, PLAY_Y + PLAY_H / 2.0 - 80.0, PLAY_W, 160.0, COLOR_OVERLAY);
            let cx = SCREEN_W / 2.0 - 100.0;
            let cy = PLAY_Y + PLAY_H / 2.0 - 50.0;
            host_api::draw_text("GAME OVER", cx, cy, 28);
            host_api::draw_text(&format!("Final Score: {}", self.score), cx - 10.0, cy + 40.0, 16);
            host_api::draw_text(&format!("Wave: {}", self.wave), cx + 30.0, cy + 65.0, 14);
            host_api::draw_text("Press START to retry", cx - 10.0, cy + 95.0, 14);
        }

        // --- Controls hint ---
        host_api::draw_text(
            "Arrows: Move  |  A/Z: Fire  |  Start: Restart",
            PLAY_X + 10.0,
            SCREEN_H - 20.0,
            12,
        );
    }
}

// ===========================================================================
//  Free helpers
// ===========================================================================

fn bunker_x(idx: usize) -> f32 {
    let spacing = PLAY_W / (BUNKER_COUNT as f32 + 1.0);
    let bunker_w = BUNKER_SEG_COLS as f32 * BUNKER_SEG_W;
    PLAY_X + spacing * (idx as f32 + 1.0) - bunker_w / 2.0
}

fn hit_bunker_seg(
    bunkers: &mut [BunkerGrid; BUNKER_COUNT],
    x: f32, y: f32, w: f32, h: f32,
) -> bool {
    let proj = Rect { x, y, w, h };
    for (bi, grid) in bunkers.iter_mut().enumerate() {
        let bkx = bunker_x(bi);
        for (r, row) in grid.iter_mut().enumerate() {
            for (c, hp) in row.iter_mut().enumerate() {
                if *hp == 0 { continue; }
                let seg = Rect {
                    x: bkx + c as f32 * BUNKER_SEG_W,
                    y: BUNKER_Y + r as f32 * BUNKER_SEG_H,
                    w: BUNKER_SEG_W,
                    h: BUNKER_SEG_H,
                };
                if proj.overlaps(&seg) {
                    *hp = hp.saturating_sub(1);
                    return true;
                }
            }
        }
    }
    false
}

#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn overlaps(&self, other: &Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}

/// Legacy 8-param AABB helper — used by collision methods that already have
/// individual coordinates unpacked.  Thin wrapper around [`Rect::overlaps`].
#[allow(clippy::too_many_arguments)]
fn aabb_overlap(
    ax: f32, ay: f32, aw: f32, ah: f32,
    bx: f32, by: f32, bw: f32, bh: f32,
) -> bool {
    Rect { x: ax, y: ay, w: aw, h: ah }
        .overlaps(&Rect { x: bx, y: by, w: bw, h: bh })
}

// ===========================================================================
//  WIT guest implementation
// ===========================================================================

impl Guest for SpaceInvaders {
    fn init() {
        unsafe { STATE = Some(Game::new()); }
    }

    fn update(delta_ms: u32) {
        #[allow(static_mut_refs)]
        let game = unsafe { STATE.as_mut().expect("init not called") };

        match game.state {
            GameState::Playing => game.update_playing(delta_ms),
            GameState::Respawning => {
                game.update_respawning(delta_ms);
                game.tick_world(delta_ms);
            }
            GameState::WaveClear => game.update_wave_clear(delta_ms),
            GameState::GameOver => {
                if !game.score_submitted {
                    game.submit_score();
                }
                if game.input_start {
                    game.reset();
                }
                game.input_start = false;
            }
        }

        game.render();
    }

    fn handle_input(action: u32) {
        #[allow(static_mut_refs)]
        let game = unsafe { STATE.as_mut().expect("init not called") };

        if action < RELEASE_OFFSET {
            match action {
                ACTION_LEFT => game.held_left = true,
                ACTION_RIGHT => game.held_right = true,
                ACTION_A | ACTION_UP => game.input_fire = true,
                ACTION_START => game.input_start = true,
                _ => {}
            }
        } else {
            match action - RELEASE_OFFSET {
                ACTION_LEFT => game.held_left = false,
                ACTION_RIGHT => game.held_right = false,
                _ => {}
            }
        }
    }

    fn get_name() -> String { "Space Invaders".to_string() }
    fn get_version() -> String { "2.0.0".to_string() }
    fn get_author() -> String { "Ignis Team".to_string() }
}

export!(SpaceInvaders);
