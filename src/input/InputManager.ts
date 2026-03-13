import { invoke } from "@tauri-apps/api/core";
import type { Action, InputEvent } from "../lib/types";
import { getKeybindings } from "../lib/tauri";

/** Default keyboard → action mapping (used until backend bindings are loaded). */
const DEFAULT_KEYMAP: Record<string, Action> = {
  ArrowUp: "Up",
  KeyW: "Up",
  ArrowDown: "Down",
  KeyS: "Down",
  ArrowLeft: "Left",
  KeyA: "Left",
  ArrowRight: "Right",
  KeyD: "Right",
  KeyZ: "ActionA",
  Space: "ActionA",
  KeyX: "ActionB",
  Enter: "Start",
  ShiftLeft: "Select",
};

/**
 * Default gamepad button → action mapping.
 * Uses the "Standard Gamepad" layout (W3C):
 *   0 = A/Cross, 1 = B/Circle, 8 = Back/Select, 9 = Start,
 *   12 = D-pad Up, 13 = D-pad Down, 14 = D-pad Left, 15 = D-pad Right
 */
const DEFAULT_GAMEPAD_MAP: Record<string, Action> = {
  "button_0": "ActionA",
  "button_1": "ActionB",
  "button_8": "Select",
  "button_9": "Start",
  "button_12": "Up",
  "button_13": "Down",
  "button_14": "Left",
  "button_15": "Right",
};

/** Axis deadzone — below this magnitude the stick is considered centred. */
const AXIS_DEADZONE = 0.5;

/** All 8 canonical actions. */
const ALL_ACTIONS: Action[] = [
  "Up", "Down", "Left", "Right", "ActionA", "ActionB", "Start", "Select",
];

type InputCallback = (event: InputEvent) => void;

/**
 * Captures keyboard + gamepad events and translates them to canonical actions.
 * Forwards each event to the Rust backend via `send_input`.
 *
 * On construction, loads the active player's keybindings from the database.
 * Falls back to DEFAULT_KEYMAP / DEFAULT_GAMEPAD_MAP until the async load completes.
 *
 * Gamepad polling uses `requestAnimationFrame` to read `navigator.getGamepads()`
 * each frame, comparing previous button/axis state to emit press/release events.
 */
export class InputManager {
  private callbacks: InputCallback[] = [];
  private keydownHandler: (e: KeyboardEvent) => void;
  private keyupHandler: (e: KeyboardEvent) => void;
  private keymap: Record<string, Action>;
  private gamepadMap: Record<string, Action>;

  /** Tracks which gamepad-sourced actions are currently held. */
  private gamepadState: Set<Action> = new Set();

  /** rAF handle for gamepad polling. */
  private gamepadRafId: number | null = null;

  /** Whether a gamepad has been seen (for status display). */
  private _gamepadConnected = false;

  constructor(playerId?: number) {
    // Start with the built-in defaults
    this.keymap = { ...DEFAULT_KEYMAP };
    this.gamepadMap = { ...DEFAULT_GAMEPAD_MAP };

    this.keydownHandler = (e) => this.handleKey(e, true);
    this.keyupHandler = (e) => this.handleKey(e, false);
    window.addEventListener("keydown", this.keydownHandler);
    window.addEventListener("keyup", this.keyupHandler);

    // Start gamepad polling
    this.pollGamepad = this.pollGamepad.bind(this);
    this.gamepadRafId = requestAnimationFrame(this.pollGamepad);

    // Asynchronously load per-player keybindings
    if (playerId !== undefined) {
      this.loadBindings(playerId);
    }
  }

  /** Whether at least one gamepad is currently connected. */
  get gamepadConnected(): boolean {
    return this._gamepadConnected;
  }

  /** Register a callback for input events. */
  onInput(callback: InputCallback): void {
    this.callbacks.push(callback);
  }

  /** Remove all listeners and callbacks. */
  destroy(): void {
    window.removeEventListener("keydown", this.keydownHandler);
    window.removeEventListener("keyup", this.keyupHandler);
    if (this.gamepadRafId !== null) {
      cancelAnimationFrame(this.gamepadRafId);
      this.gamepadRafId = null;
    }
    this.callbacks = [];
  }

  /**
   * Reload keybindings from the backend for the given player.
   * Rebuilds both keyboard and gamepad keymaps.
   */
  async reloadBindings(playerId: number): Promise<void> {
    await this.loadBindings(playerId);
  }

  private async loadBindings(playerId: number): Promise<void> {
    try {
      const rows = await getKeybindings(playerId);

      // Build fresh keymaps from the DB rows
      const freshKeyboard: Record<string, Action> = {};
      const freshGamepad: Record<string, Action> = {};

      for (const row of rows) {
        if (!ALL_ACTIONS.includes(row.action as Action)) continue;
        if (row.device_type === "keyboard") {
          freshKeyboard[row.binding] = row.action as Action;
        } else if (row.device_type === "gamepad") {
          freshGamepad[row.binding] = row.action as Action;
        }
      }

      if (Object.keys(freshKeyboard).length > 0) {
        this.keymap = freshKeyboard;
      }
      if (Object.keys(freshGamepad).length > 0) {
        this.gamepadMap = freshGamepad;
      }
    } catch {
      console.warn("InputManager: failed to load keybindings, using defaults");
    }
  }

  // ── Keyboard handling ────────────────────────────────────────

  private handleKey(e: KeyboardEvent, pressed: boolean): void {
    if (pressed && e.repeat) return;

    const action = this.keymap[e.code];
    if (!action) return;

    e.preventDefault();
    this.dispatch(action, pressed);
  }

  // ── Gamepad polling ──────────────────────────────────────────

  private pollGamepad(): void {
    const gamepads = navigator.getGamepads ? navigator.getGamepads() : [];
    let anyConnected = false;

    /** Collect the set of actions that should be held this frame. */
    const currentlyHeld = new Set<Action>();

    for (const gp of gamepads) {
      if (!gp) continue;
      anyConnected = true;

      // Buttons
      for (let i = 0; i < gp.buttons.length; i++) {
        const btn = gp.buttons[i];
        if (!btn) continue;
        const key = `button_${i}`;
        const action = this.gamepadMap[key];
        if (action && (btn.pressed || btn.value > 0.5)) {
          currentlyHeld.add(action);
        }
      }

      // Left stick axes (axis 0 = X, axis 1 = Y)
      if (gp.axes.length >= 2) {
        const x = gp.axes[0];
        const y = gp.axes[1];
        if (x < -AXIS_DEADZONE) {
          const a = this.gamepadMap["axis_left"] ?? ("Left" as Action);
          currentlyHeld.add(a);
        } else if (x > AXIS_DEADZONE) {
          const a = this.gamepadMap["axis_right"] ?? ("Right" as Action);
          currentlyHeld.add(a);
        }
        if (y < -AXIS_DEADZONE) {
          const a = this.gamepadMap["axis_up"] ?? ("Up" as Action);
          currentlyHeld.add(a);
        } else if (y > AXIS_DEADZONE) {
          const a = this.gamepadMap["axis_down"] ?? ("Down" as Action);
          currentlyHeld.add(a);
        }
      }
    }

    this._gamepadConnected = anyConnected;

    // Diff against previous state to generate press / release events.
    // New presses
    for (const action of currentlyHeld) {
      if (!this.gamepadState.has(action)) {
        this.dispatch(action, true);
      }
    }
    // Releases
    for (const action of this.gamepadState) {
      if (!currentlyHeld.has(action)) {
        this.dispatch(action, false);
      }
    }

    this.gamepadState = currentlyHeld;

    // Schedule next poll
    this.gamepadRafId = requestAnimationFrame(this.pollGamepad);
  }

  // ── Shared dispatch ──────────────────────────────────────────

  private dispatch(action: Action, pressed: boolean): void {
    const event: InputEvent = { action, pressed };

    for (const cb of this.callbacks) {
      cb(event);
    }

    invoke("send_input", { action: event.action, pressed: event.pressed }).catch(
      () => {}
    );
  }
}
