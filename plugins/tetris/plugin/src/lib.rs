wit_bindgen::generate!({
    path: "wit",
    world: "ignis-game",
});

struct Tetris;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Virtual screen dimensions (must match ignis.toml).
const SCREEN_W: f32 = 320.0;
const SCREEN_H: f32 = 240.0;

/// Board dimensions (standard Tetris: 10 wide × 20 tall, plus 2 hidden rows).
const BOARD_W: usize = 10;
const BOARD_H: usize = 22; // rows 20-21 are the hidden spawn area
const VISIBLE_H: usize = 20;

/// Cell size in pixels (fits 10×20 into 120×200 — centered on screen).
const CELL: f32 = 10.0;

/// Board rendering offset — centered horizontally, flush with bottom.
const BOARD_X: f32 = (SCREEN_W - BOARD_W as f32 * CELL) / 2.0;
const BOARD_Y: f32 = SCREEN_H - VISIBLE_H as f32 * CELL - 10.0;

/// Input action codes (must match host).
const ACTION_UP: u32 = 0;
const ACTION_DOWN: u32 = 1;
const ACTION_LEFT: u32 = 2;
const ACTION_RIGHT: u32 = 3;
const ACTION_A: u32 = 4;
const ACTION_B: u32 = 5;
const ACTION_START: u32 = 6;

/// Colors (0xRRGGBB).
const COLOR_BG: u32 = 0x0A0A1A;
const COLOR_BOARD_BG: u32 = 0x111122;
const COLOR_GRID_LINE: u32 = 0x1A1A33;
const COLOR_GHOST: u32 = 0x333355;
const COLOR_OVERLAY: u32 = 0x000000;

/// Piece colors (SRS standard palette).
const PIECE_COLORS: [u32; 7] = [
    0x00FFFF, // I — cyan
    0xFFFF00, // O — yellow
    0xAA00FF, // T — purple
    0x00FF00, // S — green
    0xFF0000, // Z — red
    0x0000FF, // J — blue
    0xFF8800, // L — orange
];

/// Gravity: milliseconds per automatic drop, indexed by level (0..=29).
/// Approximation of NES Tetris gravity curve.
const GRAVITY_MS: [u32; 30] = [
    800, 717, 633, 550, 467, 383, 300, 217, 133, 100, // 0-9
     83,  83,  83,  67,  67,  67,  50,  50,  50,  33, // 10-19
     33,  33,  33,  33,  33,  33,  33,  33,  33,  17, // 20-29
];

/// Lock delay: ms after a piece lands before it locks.
const LOCK_DELAY_MS: u32 = 500;

/// DAS: Delayed Auto Shift for left/right movement.
const DAS_INITIAL_MS: u32 = 167; // ~10 frames at 60Hz
const DAS_REPEAT_MS: u32 = 33;   // ~2 frames at 60Hz

/// Soft drop speed multiplier (20× gravity).
const SOFT_DROP_FACTOR: u32 = 20;

// ---------------------------------------------------------------------------
// SRS Tetromino Data
// ---------------------------------------------------------------------------

/// Each piece has 4 rotation states (0=spawn, 1=R, 2=180, 3=L).
/// Stored as 4×4 bitmasks — bit `(row * 4 + col)` indicates a filled cell.
/// row 0 is top, col 0 is left.
///
/// We store [piece_id][rotation] = array of 4 (col, row) offsets from the
/// piece origin (top-left of the 4×4 bounding box).
type CellOffsets = [(i32, i32); 4];

/// I piece rotations.
const I_ROTATIONS: [CellOffsets; 4] = [
    [(0, 1), (1, 1), (2, 1), (3, 1)],  // 0: flat
    [(2, 0), (2, 1), (2, 2), (2, 3)],  // R
    [(0, 2), (1, 2), (2, 2), (3, 2)],  // 2
    [(1, 0), (1, 1), (1, 2), (1, 3)],  // L
];

/// O piece rotations (same shape all 4 states).
const O_ROTATIONS: [CellOffsets; 4] = [
    [(1, 0), (2, 0), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (2, 1)],
];

/// T piece rotations.
const T_ROTATIONS: [CellOffsets; 4] = [
    [(1, 0), (0, 1), (1, 1), (2, 1)],  // 0
    [(1, 0), (1, 1), (2, 1), (1, 2)],  // R
    [(0, 1), (1, 1), (2, 1), (1, 2)],  // 2
    [(1, 0), (0, 1), (1, 1), (1, 2)],  // L
];

/// S piece rotations.
const S_ROTATIONS: [CellOffsets; 4] = [
    [(1, 0), (2, 0), (0, 1), (1, 1)],  // 0
    [(1, 0), (1, 1), (2, 1), (2, 2)],  // R
    [(1, 1), (2, 1), (0, 2), (1, 2)],  // 2
    [(0, 0), (0, 1), (1, 1), (1, 2)],  // L
];

/// Z piece rotations.
const Z_ROTATIONS: [CellOffsets; 4] = [
    [(0, 0), (1, 0), (1, 1), (2, 1)],  // 0
    [(2, 0), (1, 1), (2, 1), (1, 2)],  // R
    [(0, 1), (1, 1), (1, 2), (2, 2)],  // 2
    [(1, 0), (0, 1), (1, 1), (0, 2)],  // L
];

/// J piece rotations.
const J_ROTATIONS: [CellOffsets; 4] = [
    [(0, 0), (0, 1), (1, 1), (2, 1)],  // 0
    [(1, 0), (2, 0), (1, 1), (1, 2)],  // R
    [(0, 1), (1, 1), (2, 1), (2, 2)],  // 2
    [(1, 0), (1, 1), (0, 2), (1, 2)],  // L
];

/// L piece rotations.
const L_ROTATIONS: [CellOffsets; 4] = [
    [(2, 0), (0, 1), (1, 1), (2, 1)],  // 0
    [(1, 0), (1, 1), (1, 2), (2, 2)],  // R
    [(0, 1), (1, 1), (2, 1), (0, 2)],  // 2
    [(0, 0), (1, 0), (1, 1), (1, 2)],  // L
];

/// All 7 pieces indexed by PieceKind.
const ALL_ROTATIONS: [[CellOffsets; 4]; 7] = [
    I_ROTATIONS,
    O_ROTATIONS,
    T_ROTATIONS,
    S_ROTATIONS,
    Z_ROTATIONS,
    J_ROTATIONS,
    L_ROTATIONS,
];

// ---------------------------------------------------------------------------
// SRS Wall Kick Data
// ---------------------------------------------------------------------------

/// Wall kick offsets for J, L, S, T, Z pieces.
/// Index: [from_rotation][test_index] -> (dx, dy).
/// For a rotation 0→R, use JLSTZ_KICKS[0]; for R→2, use JLSTZ_KICKS[1]; etc.
/// Note: dy is positive downward in our coordinate system.
const JLSTZ_KICKS: [[[(i32, i32); 5]; 2]; 4] = [
    // 0→R and 0→L
    [
        [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],  // 0→R
        [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],      // 0→L (reverse)
    ],
    // R→2 and R→0
    [
        [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],     // R→2
        [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],     // R→0
    ],
    // 2→L and 2→R
    [
        [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],     // 2→L
        [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],  // 2→R
    ],
    // L→0 and L→2
    [
        [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],  // L→0
        [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],  // L→2
    ],
];

/// Wall kick offsets for I piece — unique table.
const I_KICKS: [[[(i32, i32); 5]; 2]; 4] = [
    // 0→R and 0→L
    [
        [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],   // 0→R
        [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],   // 0→L
    ],
    // R→2 and R→0
    [
        [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],   // R→2
        [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],   // R→0
    ],
    // 2→L and 2→R
    [
        [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],   // 2→L
        [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],   // 2→R
    ],
    // L→0 and L→2
    [
        [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],   // L→0
        [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],   // L→2
    ],
];

// ---------------------------------------------------------------------------
// Game Types
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

/// Simple xorshift32 RNG.
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

/// 7-bag randomizer: ensures each of the 7 pieces appears once per bag.
struct BagRandomizer {
    bag: [u8; 7],
    index: usize,
    rng: Rng,
}

impl BagRandomizer {
    fn new(seed: u32) -> Self {
        let mut s = Self {
            bag: [0; 7],
            index: 7, // Force refill on first call
            rng: Rng::new(seed),
        };
        s.refill();
        s
    }

    fn refill(&mut self) {
        for i in 0..7u8 {
            self.bag[i as usize] = i;
        }
        // Fisher-Yates shuffle
        for i in (1..7).rev() {
            let j = self.rng.range(i as u32 + 1) as usize;
            self.bag.swap(i, j);
        }
        self.index = 0;
    }

    fn next(&mut self) -> u8 {
        if self.index >= 7 {
            self.refill();
        }
        let piece = self.bag[self.index];
        self.index += 1;
        piece
    }

    fn peek(&self) -> u8 {
        if self.index >= 7 {
            // Would refill — peek at what would be bag[0] after refill.
            // For simplicity, return the first element (won't be perfectly accurate
            // but is good enough since we always peek before next()).
            self.bag[0]
        } else {
            self.bag[self.index]
        }
    }
}

/// Active piece state.
struct ActivePiece {
    kind: u8,       // 0-6 index into ALL_ROTATIONS / PIECE_COLORS
    rotation: u8,   // 0-3
    x: i32,         // board column of piece origin (top-left of 4×4 box)
    y: i32,         // board row of piece origin
}

impl ActivePiece {
    fn cells(&self) -> CellOffsets {
        ALL_ROTATIONS[self.kind as usize][self.rotation as usize]
    }

    fn world_cells(&self) -> [(i32, i32); 4] {
        let offsets = self.cells();
        [
            (self.x + offsets[0].0, self.y + offsets[0].1),
            (self.x + offsets[1].0, self.y + offsets[1].1),
            (self.x + offsets[2].0, self.y + offsets[2].1),
            (self.x + offsets[3].0, self.y + offsets[3].1),
        ]
    }
}

/// The board: each cell is 0 (empty) or a color piece-kind+1.
type Board = [[u8; BOARD_W]; BOARD_H];

/// Full game state.
struct Game {
    state: GameState,
    board: Board,
    piece: ActivePiece,
    randomizer: BagRandomizer,

    // Scoring (NES-style)
    score: u32,
    level: u32,
    lines_cleared: u32,
    lines_until_next_level: u32,

    // Timing
    gravity_timer_ms: u32,
    lock_timer_ms: u32,
    is_locking: bool,
    lock_moves_remaining: u8, // max 15 moves/rotations during lock delay

    // DAS (Delayed Auto Shift)
    das_direction: i32, // -1, 0, +1
    das_timer_ms: u32,
    das_charged: bool,

    // Input flags (consumed each frame)
    input_left: bool,
    input_right: bool,
    input_down: bool,
    input_rotate_cw: bool,
    input_rotate_ccw: bool,
    input_hard_drop: bool,
    input_start: bool,

    // Statistics
    piece_count: u32,
    score_submitted: bool,
}

impl Game {
    fn new() -> Self {
        let mut randomizer = BagRandomizer::new(42);
        let first_kind = randomizer.next();
        Self {
            state: GameState::Playing,
            board: [[0; BOARD_W]; BOARD_H],
            piece: Self::spawn_piece(first_kind),
            randomizer,
            score: 0,
            level: 0,
            lines_cleared: 0,
            lines_until_next_level: 10,
            gravity_timer_ms: 0,
            lock_timer_ms: 0,
            is_locking: false,
            lock_moves_remaining: 15,
            das_direction: 0,
            das_timer_ms: 0,
            das_charged: false,
            input_left: false,
            input_right: false,
            input_down: false,
            input_rotate_cw: false,
            input_rotate_ccw: false,
            input_hard_drop: false,
            input_start: false,
            piece_count: 1,
            score_submitted: false,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn spawn_piece(kind: u8) -> ActivePiece {
        ActivePiece {
            kind,
            rotation: 0,
            x: 3, // Center on a 10-wide board (4×4 box left edge at col 3)
            y: BOARD_H as i32 - 2 - 4, // Spawn at row 18 (just below hidden area)
        }
    }

    // ---------------------------------------------------------------------------
    // Collision Detection
    // ---------------------------------------------------------------------------

    fn fits(&self, kind: u8, rotation: u8, x: i32, y: i32) -> bool {
        let offsets = ALL_ROTATIONS[kind as usize][rotation as usize];
        for &(dx, dy) in &offsets {
            let bx = x + dx;
            let by = y + dy;
            if bx < 0 || bx >= BOARD_W as i32 || by < 0 || by >= BOARD_H as i32 {
                return false;
            }
            if self.board[by as usize][bx as usize] != 0 {
                return false;
            }
        }
        true
    }

    /// Returns the y position for the ghost piece (hard drop target).
    fn ghost_y(&self) -> i32 {
        let mut gy = self.piece.y;
        while self.fits(self.piece.kind, self.piece.rotation, self.piece.x, gy - 1) {
            gy -= 1;
        }
        gy
    }

    // ---------------------------------------------------------------------------
    // SRS Rotation with Wall Kicks
    // ---------------------------------------------------------------------------

    fn try_rotate(&mut self, clockwise: bool) {
        let from = self.piece.rotation;
        let to = if clockwise {
            (from + 1) % 4
        } else {
            (from + 3) % 4 // +3 mod 4 = -1 mod 4
        };

        let kick_table = if self.piece.kind == 0 {
            &I_KICKS
        } else {
            &JLSTZ_KICKS
        };

        // Select kick sub-table: direction index is 0 for CW, 1 for CCW.
        let dir_idx = if clockwise { 0 } else { 1 };
        let kicks = &kick_table[from as usize][dir_idx];

        for &(dx, dy) in kicks {
            let nx = self.piece.x + dx;
            // SRS kick dy: in our coordinate system row 0 = bottom, increasing y = up.
            // SRS convention: positive dy = up, which matches our system directly.
            let ny = self.piece.y + dy;
            if self.fits(self.piece.kind, to, nx, ny) {
                self.piece.rotation = to;
                self.piece.x = nx;
                self.piece.y = ny;
                // Reset lock delay on successful rotation
                if self.is_locking && self.lock_moves_remaining > 0 {
                    self.lock_timer_ms = 0;
                    self.lock_moves_remaining -= 1;
                }
                return;
            }
        }
        // All kicks failed — rotation blocked.
    }

    // ---------------------------------------------------------------------------
    // Movement
    // ---------------------------------------------------------------------------

    fn move_horizontal(&mut self, dx: i32) -> bool {
        let nx = self.piece.x + dx;
        if self.fits(self.piece.kind, self.piece.rotation, nx, self.piece.y) {
            self.piece.x = nx;
            // Reset lock delay on successful move
            if self.is_locking && self.lock_moves_remaining > 0 {
                self.lock_timer_ms = 0;
                self.lock_moves_remaining -= 1;
            }
            true
        } else {
            false
        }
    }

    fn move_down(&mut self) -> bool {
        if self.fits(self.piece.kind, self.piece.rotation, self.piece.x, self.piece.y - 1) {
            self.piece.y -= 1;
            true
        } else {
            false
        }
    }

    fn hard_drop(&mut self) {
        let target_y = self.ghost_y();
        let drop_rows = (self.piece.y - target_y) as u32;
        self.score += drop_rows * 2; // 2 points per row for hard drop
        self.piece.y = target_y;
        self.lock_piece();
    }

    // ---------------------------------------------------------------------------
    // Locking & Line Clear
    // ---------------------------------------------------------------------------

    fn lock_piece(&mut self) {
        let cells = self.piece.world_cells();
        let kind_marker = self.piece.kind + 1; // 1-7 in board cells

        for &(cx, cy) in &cells {
            if cx >= 0 && cx < BOARD_W as i32 && cy >= 0 && cy < BOARD_H as i32 {
                self.board[cy as usize][cx as usize] = kind_marker;
            }
        }

        // Check for line clears
        let lines = self.clear_lines();
        self.add_score(lines);

        // Spawn next piece
        self.spawn_next();

        // Reset lock state
        self.is_locking = false;
        self.lock_timer_ms = 0;
        self.lock_moves_remaining = 15;
    }

    fn clear_lines(&mut self) -> u32 {
        let mut lines = 0;
        let mut dst = 0usize;

        for src in 0..BOARD_H {
            let full = self.board[src].iter().all(|&c| c != 0);
            if full {
                lines += 1;
            } else {
                if dst != src {
                    self.board[dst] = self.board[src];
                }
                dst += 1;
            }
        }

        // Clear remaining top rows
        for row in dst..BOARD_H {
            self.board[row] = [0; BOARD_W];
        }

        lines
    }

    fn add_score(&mut self, lines: u32) {
        if lines == 0 {
            return;
        }

        self.lines_cleared += lines;

        // Nintendo scoring: level multiplier × base
        let base = match lines {
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200, // Tetris!
            _ => 0,
        };
        self.score += base * (self.level + 1);

        // Level up
        if lines >= self.lines_until_next_level {
            self.level += 1;
            let overflow = lines - self.lines_until_next_level;
            self.lines_until_next_level = 10u32.saturating_sub(overflow);
        } else {
            self.lines_until_next_level -= lines;
        }
    }

    fn spawn_next(&mut self) {
        let next_kind = self.randomizer.next();
        self.piece = Self::spawn_piece(next_kind);
        self.piece_count += 1;
        self.gravity_timer_ms = 0;

        // Check game over: if the new piece doesn't fit, game is over
        if !self.fits(self.piece.kind, self.piece.rotation, self.piece.x, self.piece.y) {
            self.state = GameState::GameOver;
        }
    }

    // ---------------------------------------------------------------------------
    // Game Update
    // ---------------------------------------------------------------------------

    fn current_gravity_ms(&self) -> u32 {
        let idx = (self.level as usize).min(GRAVITY_MS.len() - 1);
        GRAVITY_MS[idx]
    }

    fn update_playing(&mut self, delta_ms: u32) {
        // --- Process input ---
        self.process_input(delta_ms);

        // --- Gravity ---
        let gravity = if self.input_down {
            self.current_gravity_ms() / SOFT_DROP_FACTOR
        } else {
            self.current_gravity_ms()
        };

        self.gravity_timer_ms += delta_ms;
        while self.gravity_timer_ms >= gravity {
            self.gravity_timer_ms -= gravity;
            let moved = self.move_down();
            if !moved {
                break;
            }
            if self.input_down {
                self.score += 1; // 1 point per soft-drop row
            }
        }

        // --- Lock delay ---
        let on_ground = !self.fits(
            self.piece.kind,
            self.piece.rotation,
            self.piece.x,
            self.piece.y - 1,
        );

        if on_ground {
            if !self.is_locking {
                self.is_locking = true;
                self.lock_timer_ms = 0;
            }
            self.lock_timer_ms += delta_ms;
            if self.lock_timer_ms >= LOCK_DELAY_MS || self.lock_moves_remaining == 0 {
                self.lock_piece();
            }
        } else {
            self.is_locking = false;
            self.lock_timer_ms = 0;
        }

        // Clear consumed input
        self.input_left = false;
        self.input_right = false;
        self.input_down = false;
        self.input_rotate_cw = false;
        self.input_rotate_ccw = false;
        self.input_hard_drop = false;
        self.input_start = false;
    }

    fn process_input(&mut self, delta_ms: u32) {
        // Rotation (consume immediately)
        if self.input_rotate_cw {
            self.try_rotate(true);
        }
        if self.input_rotate_ccw {
            self.try_rotate(false);
        }

        // Hard drop
        if self.input_hard_drop {
            self.hard_drop();
            return; // Piece locked, skip movement
        }

        // Horizontal movement with DAS
        let dir = if self.input_left {
            -1
        } else if self.input_right {
            1
        } else {
            0
        };

        if dir != 0 {
            if dir != self.das_direction {
                // New direction: move immediately, reset DAS
                self.das_direction = dir;
                self.das_timer_ms = 0;
                self.das_charged = false;
                self.move_horizontal(dir);
            } else {
                // Same direction: DAS logic
                self.das_timer_ms += delta_ms;
                if !self.das_charged {
                    if self.das_timer_ms >= DAS_INITIAL_MS {
                        self.das_charged = true;
                        self.das_timer_ms -= DAS_INITIAL_MS;
                        self.move_horizontal(dir);
                    }
                } else {
                    while self.das_timer_ms >= DAS_REPEAT_MS {
                        self.das_timer_ms -= DAS_REPEAT_MS;
                        self.move_horizontal(dir);
                    }
                }
            }
        } else {
            self.das_direction = 0;
            self.das_timer_ms = 0;
            self.das_charged = false;
        }
    }

    // ---------------------------------------------------------------------------
    // Rendering
    // ---------------------------------------------------------------------------

    fn render(&self) {
        use crate::ignis::game::host_api;

        // Background
        host_api::draw_rect(0.0, 0.0, SCREEN_W, SCREEN_H, COLOR_BG);

        // Board background
        host_api::draw_rect(
            BOARD_X - 1.0,
            BOARD_Y - 1.0,
            BOARD_W as f32 * CELL + 2.0,
            VISIBLE_H as f32 * CELL + 2.0,
            COLOR_BOARD_BG,
        );

        // Grid lines (subtle)
        for col in 0..=BOARD_W {
            let x = BOARD_X + col as f32 * CELL;
            host_api::draw_rect(x, BOARD_Y, 1.0, VISIBLE_H as f32 * CELL, COLOR_GRID_LINE);
        }
        for row in 0..=VISIBLE_H {
            let y = BOARD_Y + row as f32 * CELL;
            host_api::draw_rect(BOARD_X, y, BOARD_W as f32 * CELL, 1.0, COLOR_GRID_LINE);
        }

        // Locked cells (only visible rows: 0..VISIBLE_H)
        for row in 0..VISIBLE_H {
            for col in 0..BOARD_W {
                let cell = self.board[row][col];
                if cell != 0 {
                    let color = PIECE_COLORS[(cell - 1) as usize];
                    let px = BOARD_X + col as f32 * CELL + 1.0;
                    // Board row 0 = bottom → screen y = BOARD_Y + (VISIBLE_H - 1 - row) * CELL
                    let py = BOARD_Y + (VISIBLE_H - 1 - row) as f32 * CELL + 1.0;
                    host_api::draw_rect(px, py, CELL - 2.0, CELL - 2.0, color);
                }
            }
        }

        // Ghost piece
        if self.state == GameState::Playing {
            let gy = self.ghost_y();
            if gy != self.piece.y {
                let offsets = self.piece.cells();
                for &(dx, dy) in &offsets {
                    let bx = self.piece.x + dx;
                    let by = gy + dy;
                    if by >= 0 && (by as usize) < VISIBLE_H && bx >= 0 && bx < BOARD_W as i32 {
                        let px = BOARD_X + bx as f32 * CELL + 1.0;
                        let py = BOARD_Y + (VISIBLE_H as i32 - 1 - by) as f32 * CELL + 1.0;
                        host_api::draw_rect(px, py, CELL - 2.0, CELL - 2.0, COLOR_GHOST);
                    }
                }
            }
        }

        // Active piece
        if self.state == GameState::Playing {
            let cells = self.piece.world_cells();
            let color = PIECE_COLORS[self.piece.kind as usize];
            for &(cx, cy) in &cells {
                if cy >= 0 && (cy as usize) < VISIBLE_H && cx >= 0 && cx < BOARD_W as i32 {
                    let px = BOARD_X + cx as f32 * CELL + 1.0;
                    let py = BOARD_Y + (VISIBLE_H as i32 - 1 - cy) as f32 * CELL + 1.0;
                    host_api::draw_rect(px, py, CELL - 2.0, CELL - 2.0, color);
                }
            }
        }

        // --- HUD (right side) ---
        let hud_x = BOARD_X + BOARD_W as f32 * CELL + 15.0;

        // Next piece preview
        host_api::draw_text("NEXT", hud_x, BOARD_Y, 8);
        self.render_preview(hud_x, BOARD_Y + 14.0);

        // Score
        host_api::draw_text("SCORE", hud_x, BOARD_Y + 60.0, 8);
        let score_str = self.score.to_string();
        host_api::draw_text(&score_str, hud_x, BOARD_Y + 72.0, 8);

        // Level
        host_api::draw_text("LEVEL", hud_x, BOARD_Y + 96.0, 8);
        let level_str = self.level.to_string();
        host_api::draw_text(&level_str, hud_x, BOARD_Y + 108.0, 8);

        // Lines
        host_api::draw_text("LINES", hud_x, BOARD_Y + 132.0, 8);
        let lines_str = self.lines_cleared.to_string();
        host_api::draw_text(&lines_str, hud_x, BOARD_Y + 144.0, 8);

        // --- Left side: controls hint ---
        let left_x = 5.0;
        host_api::draw_text("CONTROLS", left_x, BOARD_Y, 6);
        host_api::draw_text("LEFT/RIGHT Move", left_x, BOARD_Y + 12.0, 5);
        host_api::draw_text("DOWN  Soft drop", left_x, BOARD_Y + 22.0, 5);
        host_api::draw_text("UP    Hard drop", left_x, BOARD_Y + 32.0, 5);
        host_api::draw_text("A     Rotate CW", left_x, BOARD_Y + 42.0, 5);
        host_api::draw_text("B     Rotate CCW", left_x, BOARD_Y + 52.0, 5);

        // --- Game Over overlay ---
        if self.state == GameState::GameOver {
            // Semi-transparent overlay
            host_api::draw_rect(
                BOARD_X,
                BOARD_Y,
                BOARD_W as f32 * CELL,
                VISIBLE_H as f32 * CELL,
                COLOR_OVERLAY,
            );
            let cx = BOARD_X + (BOARD_W as f32 * CELL) / 2.0 - 30.0;
            let cy = BOARD_Y + (VISIBLE_H as f32 * CELL) / 2.0 - 12.0;
            host_api::draw_text("GAME OVER", cx, cy, 10);
            host_api::draw_text("Press START", cx + 2.0, cy + 16.0, 7);
        }
    }

    fn render_preview(&self, x: f32, y: f32) {
        use crate::ignis::game::host_api;

        let next_kind = self.randomizer.peek();
        let offsets = ALL_ROTATIONS[next_kind as usize][0]; // spawn rotation
        let color = PIECE_COLORS[next_kind as usize];
        let preview_cell = 7.0; // Slightly smaller cells for preview

        for &(dx, dy) in &offsets {
            let px = x + dx as f32 * preview_cell;
            let py = y + dy as f32 * preview_cell;
            host_api::draw_rect(px, py, preview_cell - 1.0, preview_cell - 1.0, color);
        }
    }
}

// ---------------------------------------------------------------------------
// Global State (single-threaded WASM — no data races)
// ---------------------------------------------------------------------------

#[allow(static_mut_refs)]
static mut STATE: Option<Game> = None;

// ---------------------------------------------------------------------------
// WIT Guest Implementation
// ---------------------------------------------------------------------------

impl Guest for Tetris {
    fn init() {
        unsafe {
            STATE = Some(Game::new());
        }
    }

    fn update(delta_ms: u32) {
        let game = unsafe { STATE.as_mut().expect("init not called") };

        match game.state {
            GameState::Playing => game.update_playing(delta_ms),
            GameState::GameOver => {
                // Submit score once
                if !game.score_submitted {
                    game.score_submitted = true;
                    let score_str = game.score.to_string();
                    crate::ignis::game::host_api::set_storage("last_score", &score_str);
                }
                // Wait for Start to restart
                if game.input_start {
                    game.reset();
                }
                // Clear input
                game.input_start = false;
            }
        }

        game.render();
    }

    fn handle_input(action: u32) {
        let game = unsafe { STATE.as_mut().expect("init not called") };
        match action {
            ACTION_LEFT => game.input_left = true,
            ACTION_RIGHT => game.input_right = true,
            ACTION_DOWN => game.input_down = true,
            ACTION_UP => game.input_hard_drop = true,
            ACTION_A => game.input_rotate_cw = true,
            ACTION_B => game.input_rotate_ccw = true,
            ACTION_START => game.input_start = true,
            _ => {}
        }
    }

    fn get_name() -> String {
        "Tetris".to_string()
    }

    fn get_version() -> String {
        "1.0.0".to_string()
    }

    fn get_author() -> String {
        "Ignis Team".to_string()
    }
}

export!(Tetris);
