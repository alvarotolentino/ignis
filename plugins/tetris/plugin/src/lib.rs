wit_bindgen::generate!({
    path: "wit",
    world: "ignis-game",
});

struct Tetris;

// ===========================================================================
//  Constants
// ===========================================================================

const SCREEN_W: f32 = 960.0;
const SCREEN_H: f32 = 720.0;

/// Board: 10 cols × 22 rows (2 hidden + 20 visible).
const BOARD_W: usize = 10;
const BOARD_H: usize = 22;
const VISIBLE_H: usize = 20;
const HIDDEN_ROWS: usize = 2;

const CELL: f32 = 30.0;
const BOARD_PX_W: f32 = BOARD_W as f32 * CELL;
const BOARD_PX_H: f32 = VISIBLE_H as f32 * CELL;
const BOARD_X: f32 = (SCREEN_W - BOARD_PX_W) / 2.0;
const BOARD_Y: f32 = (SCREEN_H - BOARD_PX_H) / 2.0;

const LEFT_X: f32 = 30.0;
const RIGHT_X: f32 = BOARD_X + BOARD_PX_W + 30.0;

// --- Input ---
const ACTION_UP: u32 = 0;
const ACTION_DOWN: u32 = 1;
const ACTION_LEFT: u32 = 2;
const ACTION_RIGHT: u32 = 3;
const ACTION_A: u32 = 4;
const ACTION_B: u32 = 5;
const ACTION_START: u32 = 6;
const ACTION_SELECT: u32 = 7;
const RELEASE_OFFSET: u32 = 8;

const NEXT_COUNT: usize = 5;

/// Gravity: ms per cell drop, indexed by level 1..=15. Level 16+ = instant.
const GRAVITY_MS: [u32; 16] = [
    0, 1000, 793, 618, 473, 355, 262, 190, 135, 94, 64, 43, 28, 18, 11, 7,
];

const LOCK_DELAY_MS: u32 = 500;
const MAX_LOCK_RESETS: u8 = 15;
const DAS_DELAY_MS: u32 = 133;
const ARR_MS: u32 = 10;
const SOFT_DROP_FACTOR: u32 = 8;

// --- Colours (0xRRGGBBAA) ---
const COLOR_BG: u32 = 0x0D0D1AFF;
const COLOR_BOARD_BG: u32 = 0x1A1A2EFF;
const COLOR_GRID: u32 = 0xFFFFFF14;
const COLOR_GHOST: u32 = 0xFFFFFF4D;
const COLOR_HUD_BG: u32 = 0x15152AFF;
const COLOR_OVERLAY: u32 = 0x000000BF;
const COLOR_ACCENT: u32 = 0x00BCD4FF;

/// I, O, T, S, Z, J, L
const PIECE_COLORS: [u32; 7] = [
    0x00BCD4FF, 0xFFD600FF, 0x9C27B0FF, 0x4CAF50FF,
    0xF44336FF, 0x2196F3FF, 0xFF9800FF,
];

// ===========================================================================
//  SRS Tetromino Data (row 0 = top, +y = down)
// ===========================================================================

type CellOffsets = [(i32, i32); 4];

const I_ROTATIONS: [CellOffsets; 4] = [
    [(0, 1), (1, 1), (2, 1), (3, 1)],
    [(2, 0), (2, 1), (2, 2), (2, 3)],
    [(0, 2), (1, 2), (2, 2), (3, 2)],
    [(1, 0), (1, 1), (1, 2), (1, 3)],
];
const O_ROTATIONS: [CellOffsets; 4] = [
    [(1, 0), (2, 0), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (2, 1)],
];
const T_ROTATIONS: [CellOffsets; 4] = [
    [(1, 0), (0, 1), (1, 1), (2, 1)],
    [(1, 0), (1, 1), (2, 1), (1, 2)],
    [(0, 1), (1, 1), (2, 1), (1, 2)],
    [(1, 0), (0, 1), (1, 1), (1, 2)],
];
const S_ROTATIONS: [CellOffsets; 4] = [
    [(1, 0), (2, 0), (0, 1), (1, 1)],
    [(1, 0), (1, 1), (2, 1), (2, 2)],
    [(1, 1), (2, 1), (0, 2), (1, 2)],
    [(0, 0), (0, 1), (1, 1), (1, 2)],
];
const Z_ROTATIONS: [CellOffsets; 4] = [
    [(0, 0), (1, 0), (1, 1), (2, 1)],
    [(2, 0), (1, 1), (2, 1), (1, 2)],
    [(0, 1), (1, 1), (1, 2), (2, 2)],
    [(1, 0), (0, 1), (1, 1), (0, 2)],
];
const J_ROTATIONS: [CellOffsets; 4] = [
    [(0, 0), (0, 1), (1, 1), (2, 1)],
    [(1, 0), (2, 0), (1, 1), (1, 2)],
    [(0, 1), (1, 1), (2, 1), (2, 2)],
    [(1, 0), (1, 1), (0, 2), (1, 2)],
];
const L_ROTATIONS: [CellOffsets; 4] = [
    [(2, 0), (0, 1), (1, 1), (2, 1)],
    [(1, 0), (1, 1), (1, 2), (2, 2)],
    [(0, 1), (1, 1), (2, 1), (0, 2)],
    [(0, 0), (1, 0), (1, 1), (1, 2)],
];

const ALL_ROTATIONS: [[CellOffsets; 4]; 7] = [
    I_ROTATIONS, O_ROTATIONS, T_ROTATIONS,
    S_ROTATIONS, Z_ROTATIONS, J_ROTATIONS, L_ROTATIONS,
];

// ===========================================================================
//  SRS Wall Kicks (+y = down)
// ===========================================================================

/// JLSTZ kicks: [from_rotation][0=CW, 1=CCW][test] → (dx, dy).
const JLSTZ_KICKS: [[[(i32, i32); 5]; 2]; 4] = [
    [
        [(0, 0), (-1, 0), (-1, -1), (0,  2), (-1,  2)], // 0→R
        [(0, 0), ( 1, 0), ( 1, -1), (0,  2), ( 1,  2)], // 0→L
    ],
    [
        [(0, 0), ( 1, 0), ( 1,  1), (0, -2), ( 1, -2)], // R→2
        [(0, 0), ( 1, 0), ( 1,  1), (0, -2), ( 1, -2)], // R→0
    ],
    [
        [(0, 0), ( 1, 0), ( 1, -1), (0,  2), ( 1,  2)], // 2→L
        [(0, 0), (-1, 0), (-1, -1), (0,  2), (-1,  2)], // 2→R
    ],
    [
        [(0, 0), (-1, 0), (-1,  1), (0, -2), (-1, -2)], // L→0
        [(0, 0), (-1, 0), (-1,  1), (0, -2), (-1, -2)], // L→2
    ],
];

/// I-piece kicks.
const I_KICKS: [[[(i32, i32); 5]; 2]; 4] = [
    [
        [(0, 0), (-2, 0), ( 1, 0), (-2,  1), ( 1, -2)], // 0→R
        [(0, 0), (-1, 0), ( 2, 0), (-1, -2), ( 2,  1)], // 0→L
    ],
    [
        [(0, 0), (-1, 0), ( 2, 0), (-1, -2), ( 2,  1)], // R→2
        [(0, 0), ( 2, 0), (-1, 0), ( 2, -1), (-1,  2)], // R→0
    ],
    [
        [(0, 0), ( 2, 0), (-1, 0), ( 2, -1), (-1,  2)], // 2→L
        [(0, 0), ( 1, 0), (-2, 0), ( 1,  2), (-2, -1)], // 2→R
    ],
    [
        [(0, 0), ( 1, 0), (-2, 0), ( 1,  2), (-2, -1)], // L→0
        [(0, 0), (-2, 0), ( 1, 0), (-2,  1), ( 1, -2)], // L→2
    ],
];

// ===========================================================================
//  Types
// ===========================================================================

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

// ---------------------------------------------------------------------------
//  xorshift32 RNG
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
//  7-bag queue with look-ahead
// ---------------------------------------------------------------------------

const QUEUE_CAP: usize = 14;

struct PieceQueue {
    items: [u8; QUEUE_CAP],
    head: usize,
    len: usize,
    rng: Rng,
}

impl PieceQueue {
    fn new(seed: u32) -> Self {
        let mut q = Self {
            items: [0; QUEUE_CAP],
            head: 0,
            len: 0,
            rng: Rng::new(seed),
        };
        q.fill_bag();
        q.fill_bag();
        q
    }

    fn fill_bag(&mut self) {
        let mut bag: [u8; 7] = [0, 1, 2, 3, 4, 5, 6];
        for i in (1..7usize).rev() {
            let j = self.rng.range(i as u32 + 1) as usize;
            bag.swap(i, j);
        }
        for &piece in &bag {
            let idx = (self.head + self.len) % QUEUE_CAP;
            self.items[idx] = piece;
            self.len += 1;
        }
    }

    fn next(&mut self) -> u8 {
        let piece = self.items[self.head];
        self.head = (self.head + 1) % QUEUE_CAP;
        self.len -= 1;
        if self.len < 7 {
            self.fill_bag();
        }
        piece
    }

    fn peek(&self, offset: usize) -> u8 {
        self.items[(self.head + offset) % QUEUE_CAP]
    }
}

// ---------------------------------------------------------------------------
//  Active piece
// ---------------------------------------------------------------------------

struct ActivePiece {
    kind: u8,
    rotation: u8,
    x: i32,
    y: i32,
}

impl ActivePiece {
    fn cells(&self) -> CellOffsets {
        ALL_ROTATIONS[self.kind as usize][self.rotation as usize]
    }
    fn world_cells(&self) -> [(i32, i32); 4] {
        let o = self.cells();
        core::array::from_fn(|i| (self.x + o[i].0, self.y + o[i].1))
    }
}

type Board = [[u8; BOARD_W]; BOARD_H];

// ===========================================================================
//  Game
// ===========================================================================

struct Game {
    state: GameState,
    board: Board,
    piece: ActivePiece,
    queue: PieceQueue,

    hold_piece: Option<u8>,
    hold_used: bool,

    score: u32,
    level: u32,
    lines_cleared: u32,

    combo: i32,
    b2b_active: bool,

    gravity_timer_ms: u32,
    lock_timer_ms: u32,
    is_locking: bool,
    lock_resets: u8,

    das_dir: i32,
    das_timer: u32,
    das_charged: bool,

    // Held-state (set on press, cleared on release)
    held_left: bool,
    held_right: bool,
    held_down: bool,

    // One-shot (consumed each frame)
    input_rotate_cw: bool,
    input_rotate_ccw: bool,
    input_hard_drop: bool,
    input_hold: bool,
    input_start: bool,

    last_was_rotation: bool,
    last_clear_text: &'static str,
    clear_text_timer: u32,
    score_submitted: bool,
}

impl Game {
    fn new() -> Self {
        let mut queue = PieceQueue::new(42);
        let first = queue.next();
        Self {
            state: GameState::Playing,
            board: [[0; BOARD_W]; BOARD_H],
            piece: Self::spawn_piece(first),
            queue,
            hold_piece: None,
            hold_used: false,
            score: 0,
            level: 1,
            lines_cleared: 0,
            combo: -1,
            b2b_active: false,
            gravity_timer_ms: 0,
            lock_timer_ms: 0,
            is_locking: false,
            lock_resets: 0,
            das_dir: 0,
            das_timer: 0,
            das_charged: false,
            held_left: false,
            held_right: false,
            held_down: false,
            input_rotate_cw: false,
            input_rotate_ccw: false,
            input_hard_drop: false,
            input_hold: false,
            input_start: false,
            last_was_rotation: false,
            last_clear_text: "",
            clear_text_timer: 0,
            score_submitted: false,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn spawn_piece(kind: u8) -> ActivePiece {
        ActivePiece { kind, rotation: 0, x: 3, y: 0 }
    }

    // -----------------------------------------------------------------------
    //  Helpers
    // -----------------------------------------------------------------------

    fn is_blocked(&self, x: i32, y: i32) -> bool {
        x < 0 || x >= BOARD_W as i32 || y < 0 || y >= BOARD_H as i32
            || self.board[y as usize][x as usize] != 0
    }

    fn try_reset_lock(&mut self) {
        if self.is_locking && self.lock_resets < MAX_LOCK_RESETS {
            self.lock_timer_ms = 0;
            self.lock_resets += 1;
        }
    }

    fn fits(&self, kind: u8, rot: u8, x: i32, y: i32) -> bool {
        ALL_ROTATIONS[kind as usize][rot as usize]
            .iter()
            .all(|&(dx, dy)| !self.is_blocked(x + dx, y + dy))
    }

    fn ghost_y(&self) -> i32 {
        let mut gy = self.piece.y;
        while self.fits(self.piece.kind, self.piece.rotation, self.piece.x, gy + 1) {
            gy += 1;
        }
        gy
    }

    // -----------------------------------------------------------------------
    //  SRS rotation with wall kicks
    // -----------------------------------------------------------------------

    fn try_rotate(&mut self, clockwise: bool) {
        let from = self.piece.rotation;
        let to = if clockwise { (from + 1) % 4 } else { (from + 3) % 4 };
        let table = if self.piece.kind == 0 { &I_KICKS } else { &JLSTZ_KICKS };
        let kicks = &table[from as usize][if clockwise { 0 } else { 1 }];

        for &(dx, dy) in kicks {
            if self.fits(self.piece.kind, to, self.piece.x + dx, self.piece.y + dy) {
                self.piece.rotation = to;
                self.piece.x += dx;
                self.piece.y += dy;
                self.last_was_rotation = true;
                self.try_reset_lock();
                return;
            }
        }
    }

    // -----------------------------------------------------------------------
    //  Movement
    // -----------------------------------------------------------------------

    fn move_horizontal(&mut self, dx: i32) -> bool {
        if self.fits(self.piece.kind, self.piece.rotation, self.piece.x + dx, self.piece.y) {
            self.piece.x += dx;
            self.last_was_rotation = false;
            self.try_reset_lock();
            true
        } else {
            false
        }
    }

    fn move_down(&mut self) -> bool {
        if self.fits(self.piece.kind, self.piece.rotation, self.piece.x, self.piece.y + 1) {
            self.piece.y += 1;
            self.last_was_rotation = false;
            true
        } else {
            false
        }
    }

    fn hard_drop(&mut self) {
        let target = self.ghost_y();
        self.score += (target - self.piece.y).max(0) as u32 * 2;
        self.piece.y = target;
        self.lock_piece();
    }

    // -----------------------------------------------------------------------
    //  Hold
    // -----------------------------------------------------------------------

    fn do_hold(&mut self) {
        if self.hold_used {
            return;
        }
        self.hold_used = true;
        let held = self.piece.kind;
        self.piece = if let Some(prev) = self.hold_piece {
            Self::spawn_piece(prev)
        } else {
            Self::spawn_piece(self.queue.next())
        };
        self.hold_piece = Some(held);
        self.is_locking = false;
        self.lock_timer_ms = 0;
        self.lock_resets = 0;
        self.gravity_timer_ms = 0;
        self.last_was_rotation = false;
    }

    // -----------------------------------------------------------------------
    //  T-Spin detection (3-corner rule)
    // -----------------------------------------------------------------------

    fn detect_tspin(&self) -> bool {
        if self.piece.kind != 2 || !self.last_was_rotation {
            return false;
        }
        let (cx, cy) = (self.piece.x + 1, self.piece.y + 1);
        [(cx - 1, cy - 1), (cx + 1, cy - 1), (cx - 1, cy + 1), (cx + 1, cy + 1)]
            .iter()
            .filter(|&&(x, y)| self.is_blocked(x, y))
            .count()
            >= 3
    }

    // -----------------------------------------------------------------------
    //  Locking & line clear
    // -----------------------------------------------------------------------

    fn lock_piece(&mut self) {
        let is_tspin = self.detect_tspin();
        let marker = self.piece.kind + 1;
        for &(cx, cy) in &self.piece.world_cells() {
            if cx >= 0 && cx < BOARD_W as i32 && cy >= 0 && cy < BOARD_H as i32 {
                self.board[cy as usize][cx as usize] = marker;
            }
        }
        let lines = self.clear_lines();
        self.award_score(lines, is_tspin);
        self.spawn_next();
    }

    fn clear_lines(&mut self) -> u32 {
        let mut lines = 0u32;
        let mut write = BOARD_H;
        for read in (0..BOARD_H).rev() {
            if self.board[read].iter().all(|&c| c != 0) {
                lines += 1;
            } else {
                write -= 1;
                if write != read {
                    self.board[write] = self.board[read];
                }
            }
        }
        for row in 0..write {
            self.board[row] = [0; BOARD_W];
        }
        lines
    }

    // -----------------------------------------------------------------------
    //  Scoring (Guideline)
    // -----------------------------------------------------------------------

    fn award_score(&mut self, lines: u32, is_tspin: bool) {
        self.last_clear_text = match (lines, is_tspin) {
            (0, false) => "",
            (1, false) => "SINGLE",
            (2, false) => "DOUBLE",
            (3, false) => "TRIPLE",
            (4, false) => "TETRIS!",
            (0, true) => "T-SPIN",
            (1, true) => "T-SPIN SINGLE",
            (2, true) => "T-SPIN DOUBLE",
            (3, true) => "T-SPIN TRIPLE",
            _ => "",
        };
        if !self.last_clear_text.is_empty() {
            self.clear_text_timer = 0;
        }

        if lines == 0 && !is_tspin {
            self.combo = -1;
            return;
        }

        self.combo += 1;

        let base = if is_tspin {
            match lines { 1 => 800, 2 => 1200, 3 => 1600, _ => 100 }
        } else {
            match lines { 1 => 100, 2 => 300, 3 => 500, 4 => 800, _ => 0 }
        };
        let mut points = base * self.level;

        // B2B bonus (1.5×)
        let eligible = lines == 4 || is_tspin;
        if eligible && self.b2b_active {
            points = points * 3 / 2;
        }
        if eligible {
            self.b2b_active = true;
        } else if lines > 0 {
            self.b2b_active = false;
        }

        // Combo bonus
        if self.combo > 0 {
            points += 50 * self.combo as u32 * self.level;
        }

        // All Clear bonus
        if self.board.iter().all(|row| row.iter().all(|&c| c == 0)) {
            points += 2800 * self.level;
        }

        self.score += points;
        self.lines_cleared += lines;

        // Level up: 10 lines per level, max 15
        let target = (self.lines_cleared / 10 + 1).min(15);
        if target > self.level {
            self.level = target;
        }
    }

    fn spawn_next(&mut self) {
        let kind = self.queue.next();
        self.piece = Self::spawn_piece(kind);
        self.gravity_timer_ms = 0;
        self.is_locking = false;
        self.lock_timer_ms = 0;
        self.lock_resets = 0;
        self.hold_used = false;
        self.last_was_rotation = false;

        if !self.fits(self.piece.kind, self.piece.rotation, self.piece.x, self.piece.y) {
            self.state = GameState::GameOver;
        }
    }

    // -----------------------------------------------------------------------
    //  Game loop
    // -----------------------------------------------------------------------

    fn current_gravity_ms(&self) -> u32 {
        if self.level >= 16 { 0 } else { GRAVITY_MS[self.level as usize] }
    }

    fn update_playing(&mut self, delta_ms: u32) {
        // One-shot inputs
        if self.input_rotate_cw {
            self.try_rotate(true);
            self.input_rotate_cw = false;
        }
        if self.input_rotate_ccw {
            self.try_rotate(false);
            self.input_rotate_ccw = false;
        }
        if self.input_hold {
            self.do_hold();
            self.input_hold = false;
        }
        if self.input_hard_drop {
            self.hard_drop();
            self.input_hard_drop = false;
            return;
        }

        // DAS
        self.process_das(delta_ms);

        // Gravity
        let grav = self.current_gravity_ms();
        if grav == 0 {
            while self.move_down() {}
        } else {
            let effective = if self.held_down { (grav / SOFT_DROP_FACTOR).max(1) } else { grav };
            self.gravity_timer_ms += delta_ms;
            while self.gravity_timer_ms >= effective {
                self.gravity_timer_ms -= effective;
                if !self.move_down() {
                    break;
                }
                if self.held_down {
                    self.score += 1;
                }
            }
        }

        // Lock delay
        let on_ground = !self.fits(
            self.piece.kind, self.piece.rotation, self.piece.x, self.piece.y + 1,
        );
        if on_ground {
            if !self.is_locking {
                self.is_locking = true;
                self.lock_timer_ms = 0;
            }
            self.lock_timer_ms += delta_ms;
            if self.lock_timer_ms >= LOCK_DELAY_MS || self.lock_resets >= MAX_LOCK_RESETS {
                self.lock_piece();
            }
        } else {
            self.is_locking = false;
            self.lock_timer_ms = 0;
        }

        // Clear text timer
        if !self.last_clear_text.is_empty() {
            self.clear_text_timer += delta_ms;
            if self.clear_text_timer >= 2000 {
                self.last_clear_text = "";
            }
        }
    }

    fn process_das(&mut self, delta_ms: u32) {
        let dir = match (self.held_left, self.held_right) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        };

        if dir == 0 {
            self.das_dir = 0;
            self.das_timer = 0;
            self.das_charged = false;
            return;
        }

        if dir != self.das_dir {
            self.das_dir = dir;
            self.das_timer = 0;
            self.das_charged = false;
            self.move_horizontal(dir);
        } else if !self.das_charged {
            self.das_timer += delta_ms;
            if self.das_timer >= DAS_DELAY_MS {
                self.das_charged = true;
                self.das_timer -= DAS_DELAY_MS;
                self.move_horizontal(dir);
            }
        } else {
            self.das_timer += delta_ms;
            while self.das_timer >= ARR_MS {
                self.das_timer -= ARR_MS;
                if !self.move_horizontal(dir) {
                    break;
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    //  Rendering (960 × 720, 3-column layout)
    // -----------------------------------------------------------------------

    fn draw_board_cell(col: i32, row: i32, color: u32) {
        if row >= HIDDEN_ROWS as i32 && row < BOARD_H as i32 && col >= 0 && col < BOARD_W as i32 {
            use crate::ignis::game::host_api;
            let px = BOARD_X + col as f32 * CELL + 1.0;
            let py = BOARD_Y + (row - HIDDEN_ROWS as i32) as f32 * CELL + 1.0;
            host_api::draw_rect(px, py, CELL - 2.0, CELL - 2.0, color);
        }
    }

    fn draw_piece_preview(kind: u8, x: f32, y: f32, cell_size: f32) {
        use crate::ignis::game::host_api;
        let offsets = ALL_ROTATIONS[kind as usize][0];
        let color = PIECE_COLORS[kind as usize];

        let (min_c, max_c) = offsets.iter().fold((4, 0), |(lo, hi), &(c, _)| (lo.min(c), hi.max(c)));
        let (min_r, max_r) = offsets.iter().fold((4, 0), |(lo, hi), &(_, r)| (lo.min(r), hi.max(r)));
        let ox = (4.0 - (max_c - min_c + 1) as f32) / 2.0 * cell_size;
        let oy = (3.0 - (max_r - min_r + 1) as f32) / 2.0 * cell_size;

        for &(dc, dr) in &offsets {
            let px = x + ox + (dc - min_c) as f32 * cell_size;
            let py = y + oy + (dr - min_r) as f32 * cell_size;
            host_api::draw_rect(px, py, cell_size - 2.0, cell_size - 2.0, color);
        }
    }

    fn render(&self) {
        use crate::ignis::game::host_api;

        host_api::draw_rect(0.0, 0.0, SCREEN_W, SCREEN_H, COLOR_BG);

        // Board border + background
        host_api::draw_rect(BOARD_X - 2.0, BOARD_Y - 2.0, BOARD_PX_W + 4.0, BOARD_PX_H + 4.0, COLOR_ACCENT);
        host_api::draw_rect(BOARD_X, BOARD_Y, BOARD_PX_W, BOARD_PX_H, COLOR_BOARD_BG);

        // Grid lines
        for col in 0..=BOARD_W {
            host_api::draw_rect(BOARD_X + col as f32 * CELL, BOARD_Y, 1.0, BOARD_PX_H, COLOR_GRID);
        }
        for row in 0..=VISIBLE_H {
            host_api::draw_rect(BOARD_X, BOARD_Y + row as f32 * CELL, BOARD_PX_W, 1.0, COLOR_GRID);
        }

        // Locked cells
        for row in HIDDEN_ROWS..BOARD_H {
            for col in 0..BOARD_W {
                let cell = self.board[row][col];
                if cell != 0 {
                    Self::draw_board_cell(col as i32, row as i32, PIECE_COLORS[(cell - 1) as usize]);
                }
            }
        }

        if self.state == GameState::Playing {
            // Ghost piece
            let gy = self.ghost_y();
            if gy != self.piece.y {
                for &(dx, dy) in &self.piece.cells() {
                    Self::draw_board_cell(self.piece.x + dx, gy + dy, COLOR_GHOST);
                }
            }
            // Active piece
            let color = PIECE_COLORS[self.piece.kind as usize];
            for &(cx, cy) in &self.piece.world_cells() {
                Self::draw_board_cell(cx, cy, color);
            }
        }

        // ---- LEFT PANEL ----
        let lx = LEFT_X;

        host_api::draw_text("HOLD", lx, BOARD_Y, 16);
        host_api::draw_rect(lx, BOARD_Y + 24.0, 120.0, 70.0, COLOR_HUD_BG);
        if let Some(hk) = self.hold_piece {
            Self::draw_piece_preview(hk, lx + 10.0, BOARD_Y + 34.0, 14.0);
        }

        host_api::draw_text("SCORE", lx, BOARD_Y + 120.0, 14);
        host_api::draw_text(&self.score.to_string(), lx, BOARD_Y + 140.0, 18);

        host_api::draw_text("LEVEL", lx, BOARD_Y + 180.0, 14);
        host_api::draw_text(&self.level.to_string(), lx, BOARD_Y + 200.0, 18);

        host_api::draw_text("LINES", lx, BOARD_Y + 240.0, 14);
        host_api::draw_text(&self.lines_cleared.to_string(), lx, BOARD_Y + 260.0, 18);

        host_api::draw_text("CONTROLS", lx, BOARD_Y + 340.0, 12);
        host_api::draw_text("Arrow  Move", lx, BOARD_Y + 358.0, 10);
        host_api::draw_text("Up     Hard drop", lx, BOARD_Y + 372.0, 10);
        host_api::draw_text("Down   Soft drop", lx, BOARD_Y + 386.0, 10);
        host_api::draw_text("A/Z    Rotate CW", lx, BOARD_Y + 400.0, 10);
        host_api::draw_text("B/X    Rotate CCW", lx, BOARD_Y + 414.0, 10);
        host_api::draw_text("Select Hold", lx, BOARD_Y + 428.0, 10);

        // ---- RIGHT PANEL ----
        let rx = RIGHT_X;

        host_api::draw_text("NEXT", rx, BOARD_Y, 16);
        for i in 0..NEXT_COUNT {
            let kind = self.queue.peek(i);
            let y_off = BOARD_Y + 24.0 + i as f32 * 70.0;
            host_api::draw_rect(rx, y_off, 120.0, 60.0, COLOR_HUD_BG);
            Self::draw_piece_preview(kind, rx + 10.0, y_off + 10.0, 12.0);
        }

        if self.b2b_active {
            host_api::draw_text("B2B", rx, BOARD_Y + 380.0, 14);
        }
        if self.combo > 0 {
            host_api::draw_text(&format!("COMBO x{}", self.combo), rx, BOARD_Y + 400.0, 14);
        }

        // Clear text overlay
        if !self.last_clear_text.is_empty() {
            host_api::draw_text(
                self.last_clear_text,
                BOARD_X + BOARD_PX_W / 2.0 - 60.0,
                BOARD_Y + BOARD_PX_H / 2.0 - 10.0,
                18,
            );
        }

        // Game Over overlay
        if self.state == GameState::GameOver {
            host_api::draw_rect(BOARD_X, BOARD_Y, BOARD_PX_W, BOARD_PX_H, COLOR_OVERLAY);
            let cx = BOARD_X + BOARD_PX_W / 2.0 - 60.0;
            let cy = BOARD_Y + BOARD_PX_H / 2.0 - 20.0;
            host_api::draw_text("GAME OVER", cx, cy, 20);
            host_api::draw_text(&format!("Score: {}", self.score), cx, cy + 28.0, 14);
            host_api::draw_text("Press START", cx + 4.0, cy + 52.0, 12);
        }
    }
}

// ===========================================================================
//  Global state (single-threaded WASM)
// ===========================================================================

static mut STATE: Option<Game> = None;

// ===========================================================================
//  WIT guest implementation
// ===========================================================================

impl Guest for Tetris {
    fn init() {
        unsafe { STATE = Some(Game::new()); }
    }

    fn update(delta_ms: u32) {
        #[allow(static_mut_refs)]
        let game = unsafe { STATE.as_mut().expect("init not called") };

        match game.state {
            GameState::Playing => game.update_playing(delta_ms),
            GameState::GameOver => {
                if !game.score_submitted {
                    game.score_submitted = true;
                    crate::ignis::game::host_api::set_storage(
                        "last_score",
                        &game.score.to_string(),
                    );
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
                ACTION_DOWN => game.held_down = true,
                ACTION_UP => game.input_hard_drop = true,
                ACTION_A => game.input_rotate_cw = true,
                ACTION_B => game.input_rotate_ccw = true,
                ACTION_START => game.input_start = true,
                ACTION_SELECT => game.input_hold = true,
                _ => {}
            }
        } else {
            match action - RELEASE_OFFSET {
                ACTION_LEFT => game.held_left = false,
                ACTION_RIGHT => game.held_right = false,
                ACTION_DOWN => game.held_down = false,
                _ => {}
            }
        }
    }

    fn get_name() -> String { "Tetris".to_string() }
    fn get_version() -> String { "2.0.0".to_string() }
    fn get_author() -> String { "Ignis Team".to_string() }
}

export!(Tetris);
