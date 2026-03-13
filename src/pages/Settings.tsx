/** Settings — keybinding remapper (keyboard + gamepad). */
import { useEffect, useState, useCallback, useRef } from "react";
import { useNavigate } from "react-router-dom";
import {
  getKeybindings,
  setKeybinding,
  resetKeybindings,
  type Keybinding,
} from "../lib/tauri";

/** Friendly display name for each action. */
const ACTION_LABELS: Record<string, string> = {
  ActionA: "Action A (confirm / rotate CW)",
  ActionB: "Action B (cancel / rotate CCW)",
  Down: "Down",
  Left: "Left",
  Right: "Right",
  Select: "Select",
  Start: "Start / Pause",
  Up: "Up",
};

/** Ordered list so actions appear in a sensible visual order. */
const ACTION_ORDER = [
  "Up",
  "Down",
  "Left",
  "Right",
  "ActionA",
  "ActionB",
  "Start",
  "Select",
];

/** Translates a `KeyboardEvent.code` to a short human-readable label. */
function prettyKey(code: string): string {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code.startsWith("Arrow")) return code.slice(5) + " Arrow";
  const map: Record<string, string> = {
    Space: "Space",
    Enter: "Enter",
    ShiftLeft: "Left Shift",
    ShiftRight: "Right Shift",
    ControlLeft: "Left Ctrl",
    ControlRight: "Right Ctrl",
    AltLeft: "Left Alt",
    AltRight: "Right Alt",
    Tab: "Tab",
    Backspace: "Backspace",
    Escape: "Escape",
    CapsLock: "Caps Lock",
  };
  return map[code] ?? code;
}

/** Translates a gamepad binding string to a human-readable label. */
function prettyButton(binding: string): string {
  if (binding.startsWith("button_")) {
    const idx = binding.slice(7);
    const map: Record<string, string> = {
      "0": "A / Cross",
      "1": "B / Circle",
      "2": "X / Square",
      "3": "Y / Triangle",
      "4": "LB / L1",
      "5": "RB / R1",
      "6": "LT / L2",
      "7": "RT / R2",
      "8": "Back / Select",
      "9": "Start",
      "10": "L3 (stick)",
      "11": "R3 (stick)",
      "12": "D-pad Up",
      "13": "D-pad Down",
      "14": "D-pad Left",
      "15": "D-pad Right",
    };
    return map[idx] ?? `Button ${idx}`;
  }
  if (binding.startsWith("axis_")) return binding.replace("_", " ");
  return binding;
}

// TODO: use active player once player selection is persistent
const PLAYER_ID = 1;

export default function Settings() {
  const navigate = useNavigate();
  const [bindings, setBindings] = useState<Keybinding[]>([]);
  const [loading, setLoading] = useState(true);

  /** The action currently being remapped (keyboard), or null when idle. */
  const [listeningFor, setListeningFor] = useState<string | null>(null);

  /** The action currently being remapped (gamepad), or null when idle. */
  const [gpListeningFor, setGpListeningFor] = useState<string | null>(null);

  /** Whether a gamepad is connected right now. */
  const [gamepadConnected, setGamepadConnected] = useState(false);

  /** rAF for gamepad polling in remap mode. */
  const gpRafRef = useRef<number | null>(null);

  /** Load keybindings on mount. */
  useEffect(() => {
    getKeybindings(PLAYER_ID)
      .then(setBindings)
      .catch((err: unknown) => console.error("Failed to load keybindings:", err))
      .finally(() => setLoading(false));
  }, []);

  // ── Gamepad connection detection ──────────────────────────────

  useEffect(() => {
    const onConnect = () => setGamepadConnected(true);
    const onDisconnect = () => {
      const pads = navigator.getGamepads ? navigator.getGamepads() : [];
      setGamepadConnected(pads.some((g) => g !== null));
    };
    window.addEventListener("gamepadconnected", onConnect);
    window.addEventListener("gamepaddisconnected", onDisconnect);

    // Initial check
    const pads = navigator.getGamepads ? navigator.getGamepads() : [];
    setGamepadConnected(pads.some((g) => g !== null));

    return () => {
      window.removeEventListener("gamepadconnected", onConnect);
      window.removeEventListener("gamepaddisconnected", onDisconnect);
    };
  }, []);

  // ── Keyboard remap capture ────────────────────────────────────

  /** Capture keydown when in keyboard listening mode. */
  const captureKey = useCallback(
    (e: KeyboardEvent) => {
      if (!listeningFor) return;

      // Ignore modifier-only presses
      if (["Shift", "Control", "Alt", "Meta"].includes(e.key)) return;

      e.preventDefault();
      e.stopPropagation();

      const newCode = e.code;

      setKeybinding(PLAYER_ID, listeningFor, "keyboard", newCode)
        .then(() => {
          setBindings((prev) =>
            prev.map((b) =>
              b.action === listeningFor && b.device_type === "keyboard"
                ? { ...b, binding: newCode }
                : b,
            ),
          );
        })
        .catch((err: unknown) => console.error("Failed to save keybinding:", err))
        .finally(() => setListeningFor(null));
    },
    [listeningFor],
  );

  useEffect(() => {
    if (listeningFor) {
      window.addEventListener("keydown", captureKey, { capture: true });
      return () =>
        window.removeEventListener("keydown", captureKey, { capture: true });
    }
  }, [listeningFor, captureKey]);

  // ── Gamepad remap capture (polls for first button press) ──────

  useEffect(() => {
    if (!gpListeningFor) {
      if (gpRafRef.current !== null) {
        cancelAnimationFrame(gpRafRef.current);
        gpRafRef.current = null;
      }
      return;
    }

    /** Baseline: which buttons were already held when remap started. */
    let baseline: Set<number> | null = null;

    const poll = () => {
      const gamepads = navigator.getGamepads ? navigator.getGamepads() : [];
      for (const gp of gamepads) {
        if (!gp) continue;

        // Build the set of currently pressed button indices
        const pressed = new Set<number>();
        for (let i = 0; i < gp.buttons.length; i++) {
          if (gp.buttons[i].pressed || gp.buttons[i].value > 0.5) {
            pressed.add(i);
          }
        }

        // On the first frame, record what's already held so we can ignore it
        if (baseline === null) {
          baseline = new Set(pressed);
          gpRafRef.current = requestAnimationFrame(poll);
          return;
        }

        // Look for a *newly* pressed button (not in baseline)
        for (const idx of pressed) {
          if (baseline.has(idx)) continue;

          const binding = `button_${idx}`;
          setKeybinding(PLAYER_ID, gpListeningFor, "gamepad", binding)
            .then(() => {
              setBindings((prev) =>
                prev.map((b) =>
                  b.action === gpListeningFor && b.device_type === "gamepad"
                    ? { ...b, binding }
                    : b,
                ),
              );
            })
            .catch((err: unknown) =>
              console.error("Failed to save gamepad keybinding:", err),
            )
            .finally(() => setGpListeningFor(null));
          return; // done — don't schedule another frame
        }
      }

      gpRafRef.current = requestAnimationFrame(poll);
    };

    gpRafRef.current = requestAnimationFrame(poll);

    return () => {
      if (gpRafRef.current !== null) {
        cancelAnimationFrame(gpRafRef.current);
        gpRafRef.current = null;
      }
    };
  }, [gpListeningFor]);

  /** Reset all bindings to defaults. */
  const handleReset = async () => {
    try {
      const fresh = await resetKeybindings(PLAYER_ID);
      setBindings(fresh);
    } catch (err) {
      console.error("Failed to reset keybindings:", err);
    }
  };

  /** Build lookups: action -> binding code (per device type). */
  const kbMap: Record<string, string> = {};
  const gpMap: Record<string, string> = {};
  for (const b of bindings) {
    if (b.device_type === "keyboard") {
      kbMap[b.action] = b.binding;
    } else if (b.device_type === "gamepad") {
      gpMap[b.action] = b.binding;
    }
  }

  return (
    <div style={{ paddingTop: "2rem", maxWidth: 600, margin: "0 auto" }}>
      <h2 style={{ textAlign: "center", marginBottom: "1.5rem" }}>
        Input Settings
      </h2>

      {loading ? (
        <p style={{ color: "#888", textAlign: "center" }}>Loading…</p>
      ) : (
        <>
          {/* ── Keyboard section ─────────────────────────────── */}
          <h3 style={sectionHeadingStyle}>Keyboard</h3>
          <table style={tableStyle}>
            <thead>
              <tr>
                <th style={thStyle}>Action</th>
                <th style={thStyle}>Key</th>
                <th style={thStyle}></th>
              </tr>
            </thead>
            <tbody>
              {ACTION_ORDER.map((action) => {
                const isListening = listeningFor === action;
                const code = kbMap[action] ?? "—";
                return (
                  <tr key={action}>
                    <td style={tdStyle}>
                      {ACTION_LABELS[action] ?? action}
                    </td>
                    <td
                      style={{
                        ...tdStyle,
                        color: isListening ? "#FFD700" : "#ffffff",
                        fontFamily: "monospace",
                      }}
                    >
                      {isListening ? "Press a key…" : prettyKey(code)}
                    </td>
                    <td style={tdStyle}>
                      <button
                        style={buttonStyle}
                        onClick={() =>
                          setListeningFor(isListening ? null : action)
                        }
                      >
                        {isListening ? "Cancel" : "Remap"}
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>

          {/* ── Gamepad section ──────────────────────────────── */}
          <h3 style={{ ...sectionHeadingStyle, marginTop: "2rem" }}>
            Gamepad{" "}
            <span
              style={{
                fontSize: "0.75rem",
                color: gamepadConnected ? "#4f4" : "#888",
                fontWeight: "normal",
              }}
            >
              {gamepadConnected ? "● Connected" : "○ Not detected"}
            </span>
          </h3>

          {!gamepadConnected ? (
            <p style={{ color: "#666", fontSize: "0.9rem" }}>
              Connect a controller and press any button to enable gamepad
              remapping. The left stick and D-pad map to directions
              automatically.
            </p>
          ) : (
            <table style={tableStyle}>
              <thead>
                <tr>
                  <th style={thStyle}>Action</th>
                  <th style={thStyle}>Button</th>
                  <th style={thStyle}></th>
                </tr>
              </thead>
              <tbody>
                {ACTION_ORDER.map((action) => {
                  const isListening = gpListeningFor === action;
                  const code = gpMap[action] ?? "—";
                  return (
                    <tr key={action}>
                      <td style={tdStyle}>
                        {ACTION_LABELS[action] ?? action}
                      </td>
                      <td
                        style={{
                          ...tdStyle,
                          color: isListening ? "#FFD700" : "#ffffff",
                          fontFamily: "monospace",
                        }}
                      >
                        {isListening ? "Press a button…" : prettyButton(code)}
                      </td>
                      <td style={tdStyle}>
                        <button
                          style={buttonStyle}
                          onClick={() =>
                            setGpListeningFor(isListening ? null : action)
                          }
                        >
                          {isListening ? "Cancel" : "Remap"}
                        </button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}

          {/* ── Bottom actions ───────────────────────────────── */}
          <div
            style={{
              display: "flex",
              justifyContent: "center",
              gap: "1rem",
              marginTop: "1.5rem",
            }}
          >
            <button style={resetBtnStyle} onClick={handleReset}>
              Reset All to Defaults
            </button>
            <button style={backBtnStyle} onClick={() => navigate("/")}>
              Back to Menu
            </button>
          </div>
        </>
      )}
    </div>
  );
}

// ── Inline styles ────────────────────────────────────────────

const sectionHeadingStyle: React.CSSProperties = {
  color: "#ccc",
  fontSize: "1rem",
  marginBottom: "0.75rem",
  borderBottom: "1px solid #333",
  paddingBottom: "0.25rem",
};

const tableStyle: React.CSSProperties = {
  width: "100%",
  borderCollapse: "collapse",
};

const thStyle: React.CSSProperties = {
  padding: "0.5rem 1rem",
  borderBottom: "1px solid #444",
  color: "#aaa",
  textAlign: "left",
  fontSize: "0.85rem",
  textTransform: "uppercase",
};

const tdStyle: React.CSSProperties = {
  padding: "0.6rem 1rem",
  borderBottom: "1px solid #333",
};

const buttonStyle: React.CSSProperties = {
  padding: "0.25rem 0.75rem",
  border: "1px solid #555",
  borderRadius: 4,
  background: "#2a2a2a",
  color: "#ddd",
  cursor: "pointer",
  fontSize: "0.85rem",
};

const resetBtnStyle: React.CSSProperties = {
  ...buttonStyle,
  borderColor: "#a44",
  color: "#faa",
};

const backBtnStyle: React.CSSProperties = {
  ...buttonStyle,
  borderColor: "#4a4",
  color: "#afa",
};
