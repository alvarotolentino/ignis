wit_bindgen::generate!({
    path: "wit",
    world: "ignis-game",
});

struct Asteroids;

// ==== CONSTANTS ====
const SCREEN_W: f32 = 960.0;
const SCREEN_H: f32 = 720.0;

// --- Logical game area (centered within screen, 4:3 aspect) ---
const GAME_W: f32 = 320.0;
const GAME_H: f32 = 240.0;
const SCALE: f32 = SCREEN_H / GAME_H; // 3.0
const OFFSET_X: f32 = (SCREEN_W - GAME_W * SCALE) / 2.0; // center horizontally

// --- Ship ---
const SHIP_RADIUS: f32 = 8.0;
const SHIP_ROTATION_SPEED: f32 = 4.0; // rad/sec
const SHIP_ACCEL: f32 = 120.0; // units/sec²
const SHIP_MAX_SPEED: f32 = 240.0; // units/sec
const SHIP_DRAG: f32 = 0.995; // per frame at 60fps
const SHIP_RESPAWN_CLEAR_RADIUS: f32 = 50.0;

// --- Bullets ---
const BULLET_SPEED: f32 = 300.0; // units/sec
const BULLET_TTL_MS: u32 = 1500;
const MAX_PLAYER_BULLETS: usize = 4;
const BULLET_SIZE: f32 = 2.0;

// --- Asteroids ---
const MAX_ASTEROIDS: usize = 48; // enough for heavy waves
const LARGE_RADIUS: f32 = 24.0;
const MEDIUM_RADIUS: f32 = 14.0;
const SMALL_RADIUS: f32 = 8.0;
const SPLIT_SPEED_MULT: f32 = 1.3;
const SPLIT_ANGLE_MIN: f32 = 0.52; // ~30 degrees
const SPLIT_ANGLE_MAX: f32 = 1.05; // ~60 degrees
const SPAWN_SAFE_DIST: f32 = 80.0;

// --- Saucer ---
const SAUCER_LARGE_W: f32 = 20.0;
const SAUCER_LARGE_H: f32 = 10.0;
const SAUCER_SMALL_W: f32 = 10.0;
const SAUCER_SMALL_H: f32 = 6.0;
const SAUCER_SPEED: f32 = 80.0;
const SAUCER_LARGE_FIRE_MS: u32 = 1500;
const SAUCER_SMALL_FIRE_MS: u32 = 1000;
const SAUCER_BULLET_SPEED: f32 = 200.0;
const SAUCER_BULLET_TTL_MS: u32 = 2000;
const MAX_SAUCER_BULLETS: usize = 4;
const SAUCER_MIN_INTERVAL_MS: u32 = 15000;
const SAUCER_MAX_INTERVAL_MS: u32 = 30000;

// --- Timing ---
const RESPAWN_DELAY_MS: u32 = 2000;
const RESPAWN_FORCE_MS: u32 = 5000; // Force respawn even if center not clear
const INVULN_MS: u32 = 2000;
const WAVE_CLEAR_DELAY_MS: u32 = 2000;

// --- Input actions (press/release model) ---
const ACTION_UP: u32 = 0;
const ACTION_DOWN: u32 = 1;
const ACTION_LEFT: u32 = 2;
const ACTION_RIGHT: u32 = 3;
const ACTION_A: u32 = 4;
const ACTION_B: u32 = 5;
const ACTION_START: u32 = 6;
const RELEASE_OFFSET: u32 = 8;

// --- Colours (0xRRGGBBAA) ---
const COLOR_BG: u32 = 0x000000FF;
const COLOR_WHITE: u32 = 0xFFFFFFFF;
const COLOR_GRAY: u32 = 0xBBBBBBFF;
const COLOR_RED: u32 = 0xFF4444FF;
const COLOR_SAUCER_BULLET: u32 = 0xFF6666FF;
const COLOR_THRUST: u32 = 0xFFAA00FF;
const COLOR_YELLOW: u32 = 0xFFE000FF;
const COLOR_DIM: u32 = 0x777777FF;
const COLOR_OVERLAY: u32 = 0x000000CC;
const COLOR_HUD_BG: u32 = 0x0A0A14FF;


// --- Hyperspace ---
const HYPERSPACE_DEATH_CHANCE: u32 = 8; // 1 in 8

// --- Extra lives ---
const EXTRA_LIFE_INTERVAL: u32 = 10000;

// ==== MATH ====
#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    fn new(x: f32, y: f32) -> Self { Self { x, y } }

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 { x: self.x + other.x, y: self.y + other.y }
    }

    fn scale(self, s: f32) -> Vec2 {
        Vec2 { x: self.x * s, y: self.y * s }
    }

    fn magnitude(self) -> f32 {
        sqrt(self.x * self.x + self.y * self.y)
    }

    fn from_angle(angle: f32) -> Vec2 {
        Vec2 { x: cos(angle), y: sin(angle) }
    }

}

// Math helpers — delegate to f32 intrinsics.
// WASM has native f32.sqrt / f32.abs instructions (single cycle, exact).
// std provides optimized libm for sin/cos/atan2 on wasm32-wasip2.
fn sin(x: f32) -> f32 { x.sin() }
fn cos(x: f32) -> f32 { x.cos() }
fn sqrt(x: f32) -> f32 { x.max(0.0).sqrt() }
fn abs(x: f32) -> f32 { x.abs() }
fn atan2(y: f32, x: f32) -> f32 { y.atan2(x) }

// ==== PRNG (xorshift32) ====
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

    /// Returns a float in [0.0, 1.0)
    fn f32(&mut self) -> f32 {
        (self.next_u32() & 0x7FFFFF) as f32 / (0x800000 as f32)
    }

    /// Returns a float in [lo, hi)
    fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.f32() * (hi - lo)
    }

    /// Random angle in [0, 2π)
    fn angle(&mut self) -> f32 {
        self.f32() * 6.2831853
    }
}

// ==== TYPES ====
#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    Respawning,
    WaveClear,
    GameOver,
}

#[derive(Clone, Copy, PartialEq)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    fn radius(self) -> f32 {
        match self {
            AsteroidSize::Large => LARGE_RADIUS,
            AsteroidSize::Medium => MEDIUM_RADIUS,
            AsteroidSize::Small => SMALL_RADIUS,
        }
    }

    fn points(self) -> u32 {
        match self {
            AsteroidSize::Large => 20,
            AsteroidSize::Medium => 50,
            AsteroidSize::Small => 100,
        }
    }

    fn speed_range(self, wave: u32) -> (f32, f32) {
        let wave_mult = 1.0 + ((wave - 1) as f32 * 0.1).min(1.0); // up to 2x at wave 10+
        match self {
            AsteroidSize::Large => (20.0 * wave_mult, 60.0 * wave_mult),
            AsteroidSize::Medium => (40.0 * wave_mult, 80.0 * wave_mult),
            AsteroidSize::Small => (60.0 * wave_mult, 120.0 * wave_mult),
        }
    }

    fn draw_size(self) -> f32 {
        match self {
            AsteroidSize::Large => 48.0,
            AsteroidSize::Medium => 28.0,
            AsteroidSize::Small => 16.0,
        }
    }
}

#[derive(Clone, Copy)]
struct Asteroid {
    pos: Vec2,
    vel: Vec2,
    size: AsteroidSize,
    rotation: f32,
    spin: f32, // cosmetic rotation speed
    shape_variant: u8,
    alive: bool,
}

#[derive(Clone, Copy)]
struct Ship {
    pos: Vec2,
    vel: Vec2,
    rotation: f32,
    alive: bool,
}

#[derive(Clone, Copy)]
struct Bullet {
    pos: Vec2,
    vel: Vec2,
    ttl_ms: u32,
    active: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum SaucerType {
    Large,
    Small,
}

#[derive(Clone, Copy)]
struct Saucer {
    pos: Vec2,
    vel: Vec2,
    kind: SaucerType,
    active: bool,
    fire_timer_ms: u32,
    jog_timer_ms: u32,
}

#[derive(Clone, Copy)]
struct Debris {
    pos: Vec2,
    vel: Vec2,
    timer_ms: u32,
    active: bool,
}

#[derive(Clone, Copy)]
struct ScorePop {
    x: f32,
    y: f32,
    value: u32,
    timer_ms: u32,
    active: bool,
}

#[derive(Clone, Copy)]
struct Star {
    x: f32,
    y: f32,
    brightness: u32, // alpha component
}

const MAX_DEBRIS: usize = 32;
const MAX_POPS: usize = 8;
const MAX_STARS: usize = 40;

// ==== GLOBAL STATE ====
static mut STATE: Option<Game> = None;

struct Game {
    state: GameState,

    // Ship
    ship: Ship,
    thrusting: bool,

    // Asteroids
    asteroids: [Asteroid; MAX_ASTEROIDS],
    asteroid_count: usize,

    // Bullets
    player_bullets: [Bullet; MAX_PLAYER_BULLETS],
    saucer_bullets: [Bullet; MAX_SAUCER_BULLETS],

    // Saucer
    saucer: Saucer,
    saucer_timer_ms: u32,
    saucer_next_interval: u32,

    // Score / lives
    score: u32,
    high_score: u32,
    lives: u8,
    wave: u32,
    next_extra_life_at: u32,

    // Wave tracking
    wave_clear_timer_ms: u32,
    was_hit_this_wave: bool,

    // Respawn
    respawn_timer_ms: u32,
    invuln_ms: u32,

    // Visual
    debris: [Debris; MAX_DEBRIS],
    pops: [ScorePop; MAX_POPS],
    stars: [Star; MAX_STARS],
    frame_count: u32,

    // Input held state
    held_left: bool,
    held_right: bool,
    held_thrust: bool,
    input_fire: bool,
    input_hyperspace: bool,
    input_start: bool,

    // RNG
    rng: Rng,
    score_submitted: bool,
}

impl Game {
    fn new() -> Self {
        let mut rng = Rng::new(42);
        let mut stars = [Star { x: 0.0, y: 0.0, brightness: 0 }; MAX_STARS];
        for star in &mut stars {
            star.x = rng.range_f32(0.0, GAME_W);
            star.y = rng.range_f32(0.0, GAME_H);
            star.brightness = 0x10 + rng.range(0x30);
        }

        let mut g = Self {
            state: GameState::Playing,
            ship: Ship {
                pos: Vec2::new(GAME_W / 2.0, GAME_H / 2.0),
                vel: Vec2::ZERO,
                rotation: -3.14159265 / 2.0, // pointing up
                alive: true,
            },
            thrusting: false,
            asteroids: [Asteroid {
                pos: Vec2::ZERO, vel: Vec2::ZERO, size: AsteroidSize::Large,
                rotation: 0.0, spin: 0.0, shape_variant: 0, alive: false,
            }; MAX_ASTEROIDS],
            asteroid_count: 0,
            player_bullets: [Bullet { pos: Vec2::ZERO, vel: Vec2::ZERO, ttl_ms: 0, active: false }; MAX_PLAYER_BULLETS],
            saucer_bullets: [Bullet { pos: Vec2::ZERO, vel: Vec2::ZERO, ttl_ms: 0, active: false }; MAX_SAUCER_BULLETS],
            saucer: Saucer {
                pos: Vec2::ZERO, vel: Vec2::ZERO, kind: SaucerType::Large,
                active: false, fire_timer_ms: 0, jog_timer_ms: 0,
            },
            saucer_timer_ms: 0,
            saucer_next_interval: SAUCER_MAX_INTERVAL_MS,
            score: 0,
            high_score: 0,
            lives: 3,
            wave: 1,
            next_extra_life_at: EXTRA_LIFE_INTERVAL,
            wave_clear_timer_ms: 0,
            was_hit_this_wave: false,
            respawn_timer_ms: 0,
            invuln_ms: 0,
            debris: [Debris { pos: Vec2::ZERO, vel: Vec2::ZERO, timer_ms: 0, active: false }; MAX_DEBRIS],
            pops: [ScorePop { x: 0.0, y: 0.0, value: 0, timer_ms: 0, active: false }; MAX_POPS],
            stars,
            frame_count: 0,
            held_left: false,
            held_right: false,
            held_thrust: false,
            input_fire: false,
            input_hyperspace: false,
            input_start: false,
            rng,
            score_submitted: false,
        };
        g.spawn_wave(1);
        g.load_high_score();
        g
    }

    fn reset(&mut self) {
        let hs = self.high_score;
        let stars = self.stars;
        *self = Self::new();
        self.high_score = hs;
        self.stars = stars;
    }

    fn load_high_score(&mut self) {
        if let Some(val) = crate::ignis::game::host_api::get_storage("high_score") {
            self.high_score = val.parse().unwrap_or(0);
        }
    }

    fn submit_score(&mut self) {
        if self.score_submitted { return; }
        self.score_submitted = true;
        self.update_high_score();
        crate::ignis::game::host_api::set_storage("last_score", &self.score.to_string());
        crate::ignis::game::host_api::set_storage("high_score", &self.high_score.to_string());
    }

    fn update_high_score(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
    }

    fn add_score(&mut self, pts: u32) {
        self.score += pts;
        self.update_high_score();
        // Check extra life
        if self.score >= self.next_extra_life_at {
            if self.lives < 5 {
                self.lives += 1;
            }
            self.next_extra_life_at += EXTRA_LIFE_INTERVAL;
        }
    }

    // ==== WAVE SPAWNING ====
    fn spawn_wave(&mut self, wave: u32) {
        let count = (3 + wave as usize).min(8); // 4 at wave 1, up to 8
        let center = Vec2::new(GAME_W / 2.0, GAME_H / 2.0);

        for i in 0..count {
            if let Some(slot) = self.asteroids.iter_mut().find(|a| !a.alive) {
                // Spawn around edges, away from center (bail out after 50 attempts)
                let mut pos = Vec2::new(0.0, 0.0);
                for _ in 0..50 {
                    pos = Vec2::new(
                        self.rng.range_f32(0.0, GAME_W),
                        self.rng.range_f32(0.0, GAME_H),
                    );
                    let dx = wrapped_delta(pos.x, center.x, GAME_W);
                    let dy = wrapped_delta(pos.y, center.y, GAME_H);
                    if dx * dx + dy * dy > SPAWN_SAFE_DIST * SPAWN_SAFE_DIST {
                        break;
                    }
                }

                let (lo, hi) = AsteroidSize::Large.speed_range(wave);
                let speed = self.rng.range_f32(lo, hi);
                let angle = self.rng.angle();

                *slot = Asteroid {
                    pos,
                    vel: Vec2::from_angle(angle).scale(speed),
                    size: AsteroidSize::Large,
                    rotation: self.rng.angle(),
                    spin: self.rng.range_f32(-2.0, 2.0),
                    shape_variant: self.rng.range(3) as u8,
                    alive: true,
                };
                self.asteroid_count += 1;
                let _ = i; // suppress unused
            }
        }
    }

    fn split_asteroid(&mut self, idx: usize) {
        let parent = self.asteroids[idx];
        self.asteroids[idx].alive = false;
        self.asteroid_count = self.asteroid_count.saturating_sub(1);

        self.spawn_debris(parent.pos, 6);

        let child_size = match parent.size {
            AsteroidSize::Large => Some(AsteroidSize::Medium),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Small => None,
        };

        if let Some(size) = child_size {
            let parent_angle = atan2(parent.vel.y, parent.vel.x);
            let parent_speed = parent.vel.magnitude();
            let (lo, hi) = size.speed_range(self.wave);
            let child_speed = (parent_speed * SPLIT_SPEED_MULT).clamp(lo, hi);

            for i in 0..2 {
                let offset = self.rng.range_f32(SPLIT_ANGLE_MIN, SPLIT_ANGLE_MAX);
                let angle = if i == 0 {
                    parent_angle + offset
                } else {
                    parent_angle - offset
                };

                if let Some(slot) = self.asteroids.iter_mut().find(|a| !a.alive) {
                    *slot = Asteroid {
                        pos: parent.pos,
                        vel: Vec2::from_angle(angle).scale(child_speed),
                        size,
                        rotation: self.rng.angle(),
                        spin: self.rng.range_f32(-2.0, 2.0),
                        shape_variant: self.rng.range(3) as u8,
                        alive: true,
                    };
                    self.asteroid_count += 1;
                }
            }
        }
    }

    // ==== DEBRIS / VFX ====
    fn spawn_debris(&mut self, pos: Vec2, count: usize) {
        for _ in 0..count {
            if let Some(d) = self.debris.iter_mut().find(|d| !d.active) {
                let angle = self.rng.angle();
                let speed = self.rng.range_f32(30.0, 80.0);
                *d = Debris {
                    pos,
                    vel: Vec2::from_angle(angle).scale(speed),
                    timer_ms: 0,
                    active: true,
                };
            }
        }
    }

    fn add_pop(&mut self, x: f32, y: f32, value: u32) {
        if let Some(p) = self.pops.iter_mut().find(|p| !p.active) {
            *p = ScorePop { x, y, value, timer_ms: 0, active: true };
        }
    }

    // ==== SAUCER ====
    fn spawn_saucer(&mut self) {
        let is_small = if self.wave >= 8 {
            true
        } else if self.wave >= 3 {
            self.rng.range(100) < (25 + (self.wave - 3) * 12).min(75)
        } else {
            false
        };
        let kind = if is_small { SaucerType::Small } else { SaucerType::Large };

        let from_left = self.rng.range(2) == 0;
        let x = if from_left { -20.0 } else { GAME_W + 20.0 };
        let dir = if from_left { 1.0 } else { -1.0 };
        let y = self.rng.range_f32(20.0, GAME_H - 20.0);

        self.saucer = Saucer {
            pos: Vec2::new(x, y),
            vel: Vec2::new(SAUCER_SPEED * dir, 0.0),
            kind,
            active: true,
            fire_timer_ms: 0,
            jog_timer_ms: 0,
        };
    }

    fn update_saucer(&mut self, delta_ms: u32) {
        let dt = delta_ms as f32 / 1000.0;

        if !self.saucer.active {
            if self.asteroid_count > 0 {
                self.saucer_timer_ms += delta_ms;
                if self.saucer_timer_ms >= self.saucer_next_interval {
                    self.saucer_timer_ms = 0;
                    self.spawn_saucer();
                    let min_int = SAUCER_MIN_INTERVAL_MS.saturating_sub(self.wave.saturating_sub(1) * 2000).max(10000);
                    self.saucer_next_interval = min_int + self.rng.range(SAUCER_MAX_INTERVAL_MS - min_int + 1);
                }
            }
            return;
        }

        // Move saucer
        self.saucer.pos = self.saucer.pos.add(self.saucer.vel.scale(dt));

        // Vertical wrapping
        self.saucer.pos.y = wrap(self.saucer.pos.y, GAME_H);

        // Vertical jog for movement variety
        self.saucer.jog_timer_ms += delta_ms;
        let jog_interval = if self.saucer.kind == SaucerType::Small { 600 } else { 1000 };
        if self.saucer.jog_timer_ms >= jog_interval {
            self.saucer.jog_timer_ms = 0;
            let jog_y = self.rng.range_f32(-30.0, 30.0);
            self.saucer.vel.y = jog_y;
        }

        // Exit check (horizontal only — saucers don't wrap horizontally)
        if self.saucer.pos.x < -30.0 || self.saucer.pos.x > GAME_W + 30.0 {
            self.saucer.active = false;
            return;
        }

        // Firing
        self.saucer.fire_timer_ms += delta_ms;
        let fire_interval = if self.saucer.kind == SaucerType::Small {
            SAUCER_SMALL_FIRE_MS
        } else {
            SAUCER_LARGE_FIRE_MS
        };
        if self.saucer.fire_timer_ms >= fire_interval && self.ship.alive {
            self.saucer.fire_timer_ms = 0;
            self.fire_saucer_bullet();
        }
    }

    fn fire_saucer_bullet(&mut self) {
        let slot = self.saucer_bullets.iter_mut().find(|b| !b.active);
        let slot = match slot {
            Some(s) => s,
            None => return,
        };

        let angle = if self.saucer.kind == SaucerType::Small {
            // Aimed at player
            let dx = wrapped_delta(self.ship.pos.x, self.saucer.pos.x, GAME_W);
            let dy = wrapped_delta(self.ship.pos.y, self.saucer.pos.y, GAME_H);
            atan2(dy, dx)
        } else {
            // Random direction
            self.rng.angle()
        };

        *slot = Bullet {
            pos: self.saucer.pos,
            vel: Vec2::from_angle(angle).scale(SAUCER_BULLET_SPEED),
            ttl_ms: SAUCER_BULLET_TTL_MS,
            active: true,
        };
    }

    // ==== PHYSICS UPDATE ====
    fn update_playing(&mut self, delta_ms: u32) {
        let dt = delta_ms as f32 / 1000.0;

        // --- Ship rotation ---
        if self.held_left {
            self.ship.rotation -= SHIP_ROTATION_SPEED * dt;
        }
        if self.held_right {
            self.ship.rotation += SHIP_ROTATION_SPEED * dt;
        }

        // --- Ship thrust ---
        self.thrusting = self.held_thrust;
        if self.thrusting {
            let dir = Vec2::from_angle(self.ship.rotation);
            self.ship.vel = self.ship.vel.add(dir.scale(SHIP_ACCEL * dt));
            // Clamp speed
            let spd = self.ship.vel.magnitude();
            if spd > SHIP_MAX_SPEED {
                self.ship.vel = self.ship.vel.scale(SHIP_MAX_SPEED / spd);
            }
        }

        // --- Ship drag ---
        self.ship.vel = self.ship.vel.scale(SHIP_DRAG);

        // --- Ship position ---
        self.ship.pos = self.ship.pos.add(self.ship.vel.scale(dt));
        self.ship.pos.x = wrap(self.ship.pos.x, GAME_W);
        self.ship.pos.y = wrap(self.ship.pos.y, GAME_H);

        // --- Invulnerability countdown ---
        if self.invuln_ms > 0 {
            self.invuln_ms = self.invuln_ms.saturating_sub(delta_ms);
        }

        // --- Fire bullet ---
        if self.input_fire {
            self.input_fire = false;
            let active_count = self.player_bullets.iter().filter(|b| b.active).count();
            if active_count < MAX_PLAYER_BULLETS {
                if let Some(slot) = self.player_bullets.iter_mut().find(|b| !b.active) {
                    let dir = Vec2::from_angle(self.ship.rotation);
                    let nose = self.ship.pos.add(dir.scale(SHIP_RADIUS));
                    *slot = Bullet {
                        pos: nose,
                        vel: dir.scale(BULLET_SPEED),
                        ttl_ms: BULLET_TTL_MS,
                        active: true,
                    };
                }
            }
        }

        // --- Hyperspace ---
        if self.input_hyperspace {
            self.input_hyperspace = false;
            self.ship.pos = Vec2::new(
                self.rng.range_f32(0.0, GAME_W),
                self.rng.range_f32(0.0, GAME_H),
            );
            self.ship.vel = Vec2::ZERO;
            // Risk of destruction
            if self.rng.range(HYPERSPACE_DEATH_CHANCE) == 0 {
                self.kill_ship();
                return;
            }
        }

        // --- Update bullets ---
        for b in &mut self.player_bullets {
            if !b.active { continue; }
            b.pos = b.pos.add(b.vel.scale(dt));
            b.pos.x = wrap(b.pos.x, GAME_W);
            b.pos.y = wrap(b.pos.y, GAME_H);
            b.ttl_ms = b.ttl_ms.saturating_sub(delta_ms);
            if b.ttl_ms == 0 { b.active = false; }
        }

        for b in &mut self.saucer_bullets {
            if !b.active { continue; }
            b.pos = b.pos.add(b.vel.scale(dt));
            b.pos.x = wrap(b.pos.x, GAME_W);
            b.pos.y = wrap(b.pos.y, GAME_H);
            b.ttl_ms = b.ttl_ms.saturating_sub(delta_ms);
            if b.ttl_ms == 0 { b.active = false; }
        }

        // --- Update asteroids ---
        for a in &mut self.asteroids {
            if !a.alive { continue; }
            a.pos = a.pos.add(a.vel.scale(dt));
            a.pos.x = wrap(a.pos.x, GAME_W);
            a.pos.y = wrap(a.pos.y, GAME_H);
            a.rotation += a.spin * dt;
        }

        // --- Update saucer ---
        self.update_saucer(delta_ms);

        // --- Collisions ---
        self.collide_bullets_asteroids();
        self.collide_bullets_saucer();
        self.collide_saucer_bullets_asteroids();
        self.collide_ship_asteroids();
        self.collide_ship_saucer();
        self.collide_saucer_bullets_ship();

        // --- Update VFX ---
        for d in &mut self.debris {
            if !d.active { continue; }
            d.pos = d.pos.add(d.vel.scale(dt));
            d.timer_ms += delta_ms;
            if d.timer_ms >= 400 { d.active = false; }
        }

        for p in &mut self.pops {
            if !p.active { continue; }
            p.timer_ms += delta_ms;
            p.y -= 20.0 * dt;
            if p.timer_ms >= 600 { p.active = false; }
        }

        // --- Check wave clear ---
        if self.asteroid_count == 0 && !self.saucer.active {
            // Wave bonus
            self.add_score(200 * self.wave);
            if !self.was_hit_this_wave {
                self.add_score(500);
            }
            self.state = GameState::WaveClear;
            self.wave_clear_timer_ms = 0;
        }
    }

    fn kill_ship(&mut self) {
        self.spawn_debris(self.ship.pos, 8);
        self.was_hit_this_wave = true;

        if self.lives > 1 {
            self.lives -= 1;
            self.state = GameState::Respawning;
            self.respawn_timer_ms = 0;
            self.ship.alive = false;
        } else {
            self.lives = 0;
            self.ship.alive = false;
            self.state = GameState::GameOver;
            self.submit_score();
        }
    }

    // ==== COLLISIONS (circle-based with wrapping) ====
    fn collide_bullets_asteroids(&mut self) {
        for bi in 0..MAX_PLAYER_BULLETS {
            if !self.player_bullets[bi].active { continue; }
            let bpos = self.player_bullets[bi].pos;

            for ai in 0..MAX_ASTEROIDS {
                if !self.asteroids[ai].alive { continue; }
                let apos = self.asteroids[ai].pos;
                let r = self.asteroids[ai].size.radius();

                if wrapped_dist_sq(bpos, apos) < r * r {
                    self.player_bullets[bi].active = false;
                    let pts = self.asteroids[ai].size.points();
                    self.add_score(pts);
                    self.add_pop(apos.x, apos.y, pts);
                    self.split_asteroid(ai);
                    break;
                }
            }
        }
    }

    fn collide_bullets_saucer(&mut self) {
        if !self.saucer.active { return; }
        let sr = if self.saucer.kind == SaucerType::Large {
            SAUCER_LARGE_W / 2.0
        } else {
            SAUCER_SMALL_W / 2.0
        };

        for b in &mut self.player_bullets {
            if !b.active { continue; }
            if wrapped_dist_sq(b.pos, self.saucer.pos) < sr * sr {
                b.active = false;
                let pts = if self.saucer.kind == SaucerType::Large { 200 } else { 1000 };
                self.add_score(pts);
                self.add_pop(self.saucer.pos.x, self.saucer.pos.y, pts);
                self.spawn_debris(self.saucer.pos, 6);
                self.saucer.active = false;
                return;
            }
        }
    }

    fn collide_saucer_bullets_asteroids(&mut self) {
        for bi in 0..MAX_SAUCER_BULLETS {
            if !self.saucer_bullets[bi].active { continue; }
            let bpos = self.saucer_bullets[bi].pos;

            for ai in 0..MAX_ASTEROIDS {
                if !self.asteroids[ai].alive { continue; }
                let apos = self.asteroids[ai].pos;
                let r = self.asteroids[ai].size.radius();

                if wrapped_dist_sq(bpos, apos) < r * r {
                    self.saucer_bullets[bi].active = false;
                    // Saucer bullet destroys asteroid but no player score
                    self.split_asteroid(ai);
                    break;
                }
            }
        }
    }

    fn collide_ship_asteroids(&mut self) {
        if !self.ship.alive || self.invuln_ms > 0 { return; }

        for ai in 0..MAX_ASTEROIDS {
            if !self.asteroids[ai].alive { continue; }
            let r = self.asteroids[ai].size.radius() + SHIP_RADIUS;
            if wrapped_dist_sq(self.ship.pos, self.asteroids[ai].pos) < r * r {
                self.kill_ship();
                return;
            }
        }
    }

    fn collide_ship_saucer(&mut self) {
        if !self.ship.alive || !self.saucer.active || self.invuln_ms > 0 { return; }
        let sr = if self.saucer.kind == SaucerType::Large {
            SAUCER_LARGE_W / 2.0
        } else {
            SAUCER_SMALL_W / 2.0
        };
        let r = sr + SHIP_RADIUS;
        if wrapped_dist_sq(self.ship.pos, self.saucer.pos) < r * r {
            self.spawn_debris(self.saucer.pos, 6);
            self.saucer.active = false;
            self.kill_ship();
        }
    }

    fn collide_saucer_bullets_ship(&mut self) {
        if !self.ship.alive || self.invuln_ms > 0 { return; }

        for b in &mut self.saucer_bullets {
            if !b.active { continue; }
            if wrapped_dist_sq(b.pos, self.ship.pos) < SHIP_RADIUS * SHIP_RADIUS {
                b.active = false;
                self.kill_ship();
                return;
            }
        }
    }

    // ==== RESPAWN ====
    fn update_respawning(&mut self, delta_ms: u32) {
        let dt = delta_ms as f32 / 1000.0;

        // Keep asteroids and saucer moving during respawn
        for a in &mut self.asteroids {
            if !a.alive { continue; }
            a.pos = a.pos.add(a.vel.scale(dt));
            a.pos.x = wrap(a.pos.x, GAME_W);
            a.pos.y = wrap(a.pos.y, GAME_H);
            a.rotation += a.spin * dt;
        }
        for b in &mut self.saucer_bullets {
            if !b.active { continue; }
            b.pos = b.pos.add(b.vel.scale(dt));
            b.pos.x = wrap(b.pos.x, GAME_W);
            b.pos.y = wrap(b.pos.y, GAME_H);
            b.ttl_ms = b.ttl_ms.saturating_sub(delta_ms);
            if b.ttl_ms == 0 { b.active = false; }
        }
        self.update_saucer(delta_ms);

        // Update VFX
        for d in &mut self.debris {
            if !d.active { continue; }
            d.pos = d.pos.add(d.vel.scale(dt));
            d.timer_ms += delta_ms;
            if d.timer_ms >= 400 { d.active = false; }
        }

        self.respawn_timer_ms += delta_ms;
        if self.respawn_timer_ms >= RESPAWN_DELAY_MS {
            // Check if center is clear, or force after timeout to prevent soft-hang
            let center = Vec2::new(GAME_W / 2.0, GAME_H / 2.0);
            let force = self.respawn_timer_ms >= RESPAWN_FORCE_MS;
            let clear = force || (self.asteroids.iter().all(|a| {
                !a.alive || wrapped_dist_sq(a.pos, center) > SHIP_RESPAWN_CLEAR_RADIUS * SHIP_RESPAWN_CLEAR_RADIUS
            }) && (!self.saucer.active || wrapped_dist_sq(self.saucer.pos, center) > SHIP_RESPAWN_CLEAR_RADIUS * SHIP_RESPAWN_CLEAR_RADIUS));

            if clear {
                self.ship = Ship {
                    pos: center,
                    vel: Vec2::ZERO,
                    rotation: -3.14159265 / 2.0,
                    alive: true,
                };
                self.invuln_ms = INVULN_MS;
                self.state = GameState::Playing;
                // Clear saucer bullets on respawn
                for b in &mut self.saucer_bullets { b.active = false; }
            }
        }
    }

    fn update_wave_clear(&mut self, delta_ms: u32) {
        self.wave_clear_timer_ms += delta_ms;
        if self.wave_clear_timer_ms >= WAVE_CLEAR_DELAY_MS {
            self.wave += 1;
            self.spawn_wave(self.wave);
            self.saucer.active = false;
            self.saucer_timer_ms = 0;
            for b in &mut self.player_bullets { b.active = false; }
            for b in &mut self.saucer_bullets { b.active = false; }
            self.was_hit_this_wave = false;
            self.state = GameState::Playing;
        }
    }

    // ==== RENDERING ====
    fn render(&self) {
        use crate::ignis::game::host_api as api;

        // Background
        api::draw_rect(0.0, 0.0, SCREEN_W, SCREEN_H, COLOR_BG);

        // Starfield
        for star in &self.stars {
            let (sx, sy) = to_screen(star.x, star.y);
            let alpha = star.brightness & 0xFF;
            let color = 0xFFFFFF00 | alpha;
            api::draw_rect(sx, sy, SCALE, SCALE, color);
        }

        // HUD background
        api::draw_rect(0.0, 0.0, SCREEN_W, 30.0, COLOR_HUD_BG);

        // Score
        api::draw_text(&format!("SCORE: {}", self.score), 20.0, 8.0, 16);

        // Hi-Score
        let hi_text = format!("HI: {}", self.high_score);
        api::draw_text(&hi_text, SCREEN_W / 2.0 - 60.0, 8.0, 16);

        // Wave
        api::draw_text(&format!("WAVE {}", self.wave), SCREEN_W - 250.0, 8.0, 16);

        // Lives as small ship icons
        for i in 0..self.lives {
            let lx = SCREEN_W - 120.0 + i as f32 * 20.0;
            draw_ship_icon(lx, 8.0, 6.0);
        }

        // Asteroids
        for a in &self.asteroids {
            if !a.alive { continue; }
            self.draw_asteroid(a);
        }

        // Saucer
        if self.saucer.active {
            self.draw_saucer();
        }

        // Saucer bullets
        for b in &self.saucer_bullets {
            if !b.active { continue; }
            let (sx, sy) = to_screen(b.pos.x, b.pos.y);
            api::draw_rect(sx - SCALE, sy - SCALE, BULLET_SIZE * SCALE, BULLET_SIZE * SCALE, COLOR_SAUCER_BULLET);
        }

        // Player bullets
        for b in &self.player_bullets {
            if !b.active { continue; }
            let (sx, sy) = to_screen(b.pos.x, b.pos.y);
            api::draw_rect(sx - SCALE, sy - SCALE, BULLET_SIZE * SCALE, BULLET_SIZE * SCALE, COLOR_WHITE);
        }

        // Ship
        if self.ship.alive {
            let show = if self.invuln_ms > 0 {
                (self.frame_count / 4) % 2 == 0
            } else {
                true
            };
            if show {
                self.draw_ship();
            }
        }

        // Debris
        for d in &self.debris {
            if !d.active { continue; }
            let (sx, sy) = to_screen(d.pos.x, d.pos.y);
            let alpha = 0xFF - ((d.timer_ms * 0xFF / 400).min(0xFF) as u32);
            let color = 0xFFFFFF00 | alpha;
            api::draw_rect(sx, sy, 3.0, 1.0, color);
        }

        // Score popups
        for p in &self.pops {
            if !p.active { continue; }
            let (sx, sy) = to_screen(p.x, p.y);
            api::draw_text(&format!("+{}", p.value), sx, sy, 10);
        }

        // Wave Clear overlay
        if self.state == GameState::WaveClear {
            let cx = SCREEN_W / 2.0;
            let cy = SCREEN_H / 2.0;
            api::draw_rect(cx - 180.0, cy - 50.0, 360.0, 100.0, COLOR_OVERLAY);
            api::draw_text(
                &format!("WAVE {} CLEARED!", self.wave),
                cx - 120.0, cy - 30.0, 20,
            );
            api::draw_text(
                &format!("Score: {}", self.score),
                cx - 60.0, cy + 5.0, 14,
            );
        }

        // Game Over overlay
        if self.state == GameState::GameOver {
            let cx = SCREEN_W / 2.0;
            let cy = SCREEN_H / 2.0;
            api::draw_rect(cx - 200.0, cy - 80.0, 400.0, 180.0, COLOR_OVERLAY);
            api::draw_text("GAME OVER", cx - 100.0, cy - 55.0, 28);
            api::draw_text(
                &format!("Final Score: {}", self.score),
                cx - 90.0, cy - 10.0, 16,
            );
            api::draw_text(
                &format!("Wave: {}", self.wave),
                cx - 40.0, cy + 20.0, 14,
            );
            if self.score == self.high_score && self.score > 0 {
                api::draw_text("NEW HIGH SCORE!", cx - 80.0, cy + 45.0, 14);
            }
            api::draw_text("Press START to retry", cx - 100.0, cy + 70.0, 12);
        }

        // Controls hint
        api::draw_text(
            "Arrows: Rotate/Thrust  |  A/Z: Fire  |  Shift/X: Hyperspace  |  Start: Restart",
            OFFSET_X + 10.0, SCREEN_H - 18.0, 10,
        );
    }

    fn draw_ship(&self) {
        let p = self.ship.pos;
        let r = self.ship.rotation;

        // Classic ship: nose, two wings, rear notch
        let nose  = p.add(Vec2::from_angle(r).scale(SHIP_RADIUS));
        let left  = p.add(Vec2::from_angle(r + 2.356).scale(SHIP_RADIUS * 0.8));
        let right = p.add(Vec2::from_angle(r - 2.356).scale(SHIP_RADIUS * 0.8));
        let notch = p.add(Vec2::from_angle(r + 3.14159265).scale(SHIP_RADIUS * 0.35));

        draw_line_seg(nose, left, COLOR_WHITE);
        draw_line_seg(left, notch, COLOR_WHITE);
        draw_line_seg(notch, right, COLOR_WHITE);
        draw_line_seg(right, nose, COLOR_WHITE);

        // Thrust flame
        if self.thrusting && self.frame_count % 4 < 3 {
            let rear = p.add(Vec2::from_angle(r + 3.14159265).scale(SHIP_RADIUS * 1.2));
            let fl = p.add(Vec2::from_angle(r + 2.8).scale(SHIP_RADIUS * 0.5));
            let fr = p.add(Vec2::from_angle(r - 2.8).scale(SHIP_RADIUS * 0.5));
            draw_line_seg(fl, rear, COLOR_THRUST);
            draw_line_seg(fr, rear, COLOR_THRUST);
        }

        // Ghost rendering at edges
        let ghost_threshold = SHIP_RADIUS + 4.0;
        let gx = if p.x < ghost_threshold { Some(p.x + GAME_W) }
            else if p.x > GAME_W - ghost_threshold { Some(p.x - GAME_W) }
            else { None };
        let gy = if p.y < ghost_threshold { Some(p.y + GAME_H) }
            else if p.y > GAME_H - ghost_threshold { Some(p.y - GAME_H) }
            else { None };

        if let Some(gx) = gx {
            let gp = Vec2::new(gx, p.y);
            let gnose  = gp.add(Vec2::from_angle(r).scale(SHIP_RADIUS));
            let gleft  = gp.add(Vec2::from_angle(r + 2.356).scale(SHIP_RADIUS * 0.8));
            let gright = gp.add(Vec2::from_angle(r - 2.356).scale(SHIP_RADIUS * 0.8));
            let gnotch = gp.add(Vec2::from_angle(r + 3.14159265).scale(SHIP_RADIUS * 0.35));
            draw_line_seg(gnose, gleft, COLOR_DIM);
            draw_line_seg(gleft, gnotch, COLOR_DIM);
            draw_line_seg(gnotch, gright, COLOR_DIM);
            draw_line_seg(gright, gnose, COLOR_DIM);
        }
        if let Some(gy) = gy {
            let gp = Vec2::new(p.x, gy);
            let gnose  = gp.add(Vec2::from_angle(r).scale(SHIP_RADIUS));
            let gleft  = gp.add(Vec2::from_angle(r + 2.356).scale(SHIP_RADIUS * 0.8));
            let gright = gp.add(Vec2::from_angle(r - 2.356).scale(SHIP_RADIUS * 0.8));
            let gnotch = gp.add(Vec2::from_angle(r + 3.14159265).scale(SHIP_RADIUS * 0.35));
            draw_line_seg(gnose, gleft, COLOR_DIM);
            draw_line_seg(gleft, gnotch, COLOR_DIM);
            draw_line_seg(gnotch, gright, COLOR_DIM);
            draw_line_seg(gright, gnose, COLOR_DIM);
        }
    }

    fn draw_asteroid(&self, a: &Asteroid) {
        let sz = a.size.draw_size();
        let half = sz / 2.0;

        // Draw as a rotated polygon outline using line segments
        let num_verts: usize = match a.size {
            AsteroidSize::Large => 9,
            AsteroidSize::Medium => 7,
            AsteroidSize::Small => 5,
        };

        // Generate asteroid shape vertices
        let mut verts = [(0.0f32, 0.0f32); 9];
        let base_angle = 6.2831853 / num_verts as f32;

        for i in 0..num_verts {
            let angle = a.rotation + base_angle * i as f32;
            let r_factor = match (a.shape_variant, i % 4) {
                (0, 0) => 0.8,  (0, 1) => 1.0,  (0, 2) => 0.75, (0, 3) => 0.95,
                (1, 0) => 0.9,  (1, 1) => 0.7,  (1, 2) => 1.0,  (1, 3) => 0.85,
                (_, 0) => 1.0,  (_, 1) => 0.85, (_, 2) => 0.9,  (_, _) => 0.75,
            };
            let r = half * r_factor;
            verts[i] = (a.pos.x + cos(angle) * r, a.pos.y + sin(angle) * r);
        }

        let color = match a.size {
            AsteroidSize::Large => COLOR_GRAY,
            AsteroidSize::Medium => COLOR_GRAY,
            AsteroidSize::Small => 0xCCCCCCFF,
        };

        // Draw outline
        for i in 0..num_verts {
            let j = (i + 1) % num_verts;
            draw_line_seg(
                Vec2::new(verts[i].0, verts[i].1),
                Vec2::new(verts[j].0, verts[j].1),
                color,
            );
        }

        // Ghost rendering: only for asteroids actually near edges
        let ghost_threshold = half + 4.0;
        let near_left = a.pos.x < ghost_threshold;
        let near_right = a.pos.x > GAME_W - ghost_threshold;
        let near_top = a.pos.y < ghost_threshold;
        let near_bottom = a.pos.y > GAME_H - ghost_threshold;
        if !(near_left || near_right || near_top || near_bottom) {
            return; // Most asteroids skip ghost rendering entirely
        }

        // Draw ghost copies for edge-adjacent asteroids
        let offsets = ghost_offsets(a.pos, ghost_threshold);
        for &(ox, oy) in &offsets {
            if ox == 0.0 && oy == 0.0 { continue; }
            for i in 0..num_verts {
                let j = (i + 1) % num_verts;
                draw_line_seg(
                    Vec2::new(verts[i].0 + ox, verts[i].1 + oy),
                    Vec2::new(verts[j].0 + ox, verts[j].1 + oy),
                    COLOR_DIM,
                );
            }
        }
    }

    fn draw_saucer(&self) {
        use crate::ignis::game::host_api as api;
        let (w, h) = if self.saucer.kind == SaucerType::Large {
            (SAUCER_LARGE_W, SAUCER_LARGE_H)
        } else {
            (SAUCER_SMALL_W, SAUCER_SMALL_H)
        };

        let p = self.saucer.pos;
        let (sx, sy) = to_screen(p.x - w / 2.0, p.y - h / 2.0);

        // Main body — diamond shape using 3 rects
        let sw = w * SCALE;
        let sh = h * SCALE;
        api::draw_rect(sx + sw * 0.1, sy + sh * 0.3, sw * 0.8, sh * 0.4, COLOR_RED);
        // Dome on top
        api::draw_rect(sx + sw * 0.3, sy, sw * 0.4, sh * 0.35, COLOR_RED);
        // Flashing indicator
        if self.frame_count % 8 < 4 {
            api::draw_rect(sx + sw * 0.45, sy + sh * 0.05, sw * 0.1, sh * 0.15, COLOR_YELLOW);
        }
    }
}

// ==== FREE HELPERS ====

/// Wrap a coordinate into [0, max)
fn wrap(val: f32, max: f32) -> f32 {
    let v = val % max;
    if v < 0.0 { v + max } else { v }
}

/// Wrapped delta: shortest distance along a wrapped axis (signed)
fn wrapped_delta(a: f32, b: f32, size: f32) -> f32 {
    let mut d = a - b;
    if d > size / 2.0 { d -= size; }
    if d < -size / 2.0 { d += size; }
    d
}

/// Wrapped squared distance between two points
fn wrapped_dist_sq(a: Vec2, b: Vec2) -> f32 {
    let dx = wrapped_delta(a.x, b.x, GAME_W);
    let dy = wrapped_delta(a.y, b.y, GAME_H);
    dx * dx + dy * dy
}

/// Convert game coordinates to screen coordinates
fn to_screen(gx: f32, gy: f32) -> (f32, f32) {
    (OFFSET_X + gx * SCALE, 30.0 + gy * SCALE) // 30px reserved for HUD
}

/// Draw a line segment as a series of SCALE×SCALE rects between two game-space points.
/// Uses major-axis stepping to minimize draw_rect host calls while keeping vertex accuracy.
fn draw_line_seg(a: Vec2, b: Vec2, color: u32) {
    use crate::ignis::game::host_api as api;

    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let adx = abs(dx);
    let ady = abs(dy);
    let major = if adx > ady { adx } else { ady };

    if major < 1.5 {
        let (sx, sy) = to_screen(a.x, a.y);
        api::draw_rect(sx, sy, SCALE, SCALE, color);
        return;
    }

    // 1 rect per ~2.5 game units along major axis, capped at 10 to bound host calls.
    // Endpoints (t=0, t=1) are always drawn so vertex positions are exact.
    let steps = ((major * 0.4).ceil() as u32).clamp(2, 10);
    let inv = 1.0 / steps as f32;

    for i in 0..=steps {
        let t = i as f32 * inv;
        let (sx, sy) = to_screen(a.x + dx * t, a.y + dy * t);
        api::draw_rect(sx, sy, SCALE, SCALE, color);
    }
}

/// Draw a small ship icon at screen coordinates (for lives display)
fn draw_ship_icon(sx: f32, sy: f32, size: f32) {
    use crate::ignis::game::host_api as api;
    // Simple upward-pointing triangle
    api::draw_rect(sx + size / 2.0 - 1.0, sy, 2.0, size, COLOR_WHITE);
    api::draw_rect(sx, sy + size * 0.5, size, 2.0, COLOR_WHITE);
}

/// Returns ghost offsets for edge rendering
fn ghost_offsets(pos: Vec2, threshold: f32) -> [(f32, f32); 4] {
    let mut offsets = [(0.0f32, 0.0f32); 4];
    let mut idx = 0;

    if pos.x < threshold {
        offsets[idx] = (GAME_W, 0.0);
        idx += 1;
    } else if pos.x > GAME_W - threshold {
        offsets[idx] = (-GAME_W, 0.0);
        idx += 1;
    }

    if pos.y < threshold {
        offsets[idx] = (0.0, GAME_H);
        idx += 1;
    } else if pos.y > GAME_H - threshold {
        offsets[idx] = (0.0, -GAME_H);
        idx += 1;
    }

    // Corner case: both axes need a ghost
    if idx == 2 {
        offsets[2] = (offsets[0].0 + offsets[1].0, offsets[0].1 + offsets[1].1);
    }

    offsets
}

// ==== WIT GUEST IMPLEMENTATION ====
impl Guest for Asteroids {
    fn init() {
        unsafe { STATE = Some(Game::new()); }
    }

    fn update(delta_ms: u32) {
        #[allow(static_mut_refs)]
        let game = unsafe { STATE.as_mut().expect("init not called") };
        game.frame_count += 1;

        match game.state {
            GameState::Playing => game.update_playing(delta_ms),
            GameState::Respawning => game.update_respawning(delta_ms),
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
            // Press
            match action {
                ACTION_LEFT => game.held_left = true,
                ACTION_RIGHT => game.held_right = true,
                ACTION_UP | ACTION_A => game.held_thrust = true,
                ACTION_B => game.input_fire = true,
                ACTION_DOWN => game.input_hyperspace = true,
                ACTION_START => game.input_start = true,
                _ => {}
            }
        } else {
            // Release
            match action - RELEASE_OFFSET {
                ACTION_LEFT => game.held_left = false,
                ACTION_RIGHT => game.held_right = false,
                ACTION_UP | ACTION_A => game.held_thrust = false,
                _ => {}
            }
        }
    }

    fn get_name() -> String { "Asteroids".to_string() }
    fn get_version() -> String { "1.0.0".to_string() }
    fn get_author() -> String { "Ignis Team".to_string() }
}

export!(Asteroids);
