import { invoke } from "@tauri-apps/api/core";
import type { Action, InputEvent } from "../lib/types";

/** Default keyboard → action mapping. */
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

type InputCallback = (event: InputEvent) => void;

/**
 * Captures keyboard events and translates them to canonical actions.
 * Forwards each event to the Rust backend via `send_input`.
 */
export class InputManager {
  private callbacks: InputCallback[] = [];
  private keydownHandler: (e: KeyboardEvent) => void;
  private keyupHandler: (e: KeyboardEvent) => void;

  constructor() {
    this.keydownHandler = (e) => this.handleKey(e, true);
    this.keyupHandler = (e) => this.handleKey(e, false);
    window.addEventListener("keydown", this.keydownHandler);
    window.addEventListener("keyup", this.keyupHandler);
  }

  /** Register a callback for input events. */
  onInput(callback: InputCallback): void {
    this.callbacks.push(callback);
  }

  /** Remove all listeners and callbacks. */
  destroy(): void {
    window.removeEventListener("keydown", this.keydownHandler);
    window.removeEventListener("keyup", this.keyupHandler);
    this.callbacks = [];
  }

  private handleKey(e: KeyboardEvent, pressed: boolean): void {
    // Prevent key repeat from firing multiple events
    if (pressed && e.repeat) return;

    const action = DEFAULT_KEYMAP[e.code];
    if (!action) return;

    // Prevent default for game keys (avoid scrolling etc.)
    e.preventDefault();

    const event: InputEvent = { action, pressed };

    // Notify local callbacks
    for (const cb of this.callbacks) {
      cb(event);
    }

    // Dispatch to Rust backend
    invoke("send_input", { action: event.action, pressed: event.pressed }).catch(
      () => {}
    );
  }
}
