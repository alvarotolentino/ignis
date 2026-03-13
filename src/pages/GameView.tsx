import { useEffect, useRef, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { onRenderFrame } from "../lib/renderBridge";
import { submitScore } from "../lib/tauri";
import { getPixiApp, getPixiCanvas, destroyPixiApp } from "../renderer/PixiApp";
import { FrameRenderer } from "../renderer/FrameRenderer";
import { SpriteManager } from "../renderer/SpriteManager";
import { AudioManager } from "../renderer/AudioManager";
import { InputManager } from "../input/InputManager";

interface GameOverPayload {
  game_id: string;
  score: number;
}

/** Game View — loads a WASM plugin, renders frames via PixiJS. */
export default function GameView() {
  const [params] = useSearchParams();
  const pluginId = params.get("plugin") ?? "";
  const navigate = useNavigate();
  const canvasRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<FrameRenderer | null>(null);
  const spriteManagerRef = useRef<SpriteManager | null>(null);
  const audioManagerRef = useRef<AudioManager | null>(null);
  const inputRef = useRef<InputManager | null>(null);
  const [gameOver, setGameOver] = useState<GameOverPayload | null>(null);

  useEffect(() => {
    if (!pluginId) return;

    let unlisten: (() => void) | undefined;
    let mounted = true;

    (async () => {
      // Initialize sprite and audio managers (they listen for events before game starts)
      const spriteManager = new SpriteManager();
      await spriteManager.init();
      spriteManagerRef.current = spriteManager;

      const audioManager = new AudioManager();
      await audioManager.init();
      audioManagerRef.current = audioManager;

      // Initialize PixiJS
      const app = await getPixiApp();
      if (!mounted) return;

      const canvas = getPixiCanvas();
      if (canvas && canvasRef.current) {
        canvasRef.current.innerHTML = "";
        canvasRef.current.appendChild(canvas);
      }

      rendererRef.current = new FrameRenderer(app, spriteManager, audioManager);

      // Initialize input capture (loads per-player keybindings from DB)
      inputRef.current = new InputManager(1);

      // Start the game
      await invoke("start_game", { pluginId });

      // Listen for game_over event from the Rust backend
      const unlistenGameOver = await listen<GameOverPayload>("game_over", async (event) => {
        if (!mounted) return;
        const payload = event.payload;
        setGameOver(payload);

        // Auto-submit score (default player_id = 1 until player profiles are wired)
        try {
          await submitScore(1, payload.game_id, payload.score);
        } catch (err) {
          console.error("Failed to submit score:", err);
        }
      });

      // Subscribe to render frames
      if (!mounted) return;
      unlisten = await onRenderFrame((frame) => {
        rendererRef.current?.renderFrame(frame);
      });

      // Stash the game-over unlisten alongside the render unlisten
      const originalUnlisten = unlisten;
      unlisten = () => {
        originalUnlisten?.();
        unlistenGameOver();
      };
    })().catch((err) => console.error("start_game failed:", err));

    return () => {
      mounted = false;
      unlisten?.();
      inputRef.current?.destroy();
      inputRef.current = null;
      spriteManagerRef.current?.destroy();
      spriteManagerRef.current = null;
      audioManagerRef.current?.destroy();
      audioManagerRef.current = null;
      rendererRef.current?.destroy();
      rendererRef.current = null;
      destroyPixiApp();
      invoke("stop_game").catch(() => {});
    };
  }, [pluginId]);

  // ESC key → back to menu
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.code === "Escape") {
        e.preventDefault();
        handleBack();
      }
    };
    window.addEventListener("keydown", handleEsc);
    return () => window.removeEventListener("keydown", handleEsc);
  });

  const handleBack = () => {
    invoke("stop_game").catch(() => {});
    navigate("/");
  };

  return (
    <div style={{ paddingTop: "1rem" }}>
      <button onClick={handleBack} style={btnStyle}>
        ← Back to Menu
      </button>
      <div
        ref={canvasRef}
        style={{
          marginTop: "1rem",
          width: "960px",
          height: "720px",
          background: "#0a0a0f",
          borderRadius: "8px",
          overflow: "hidden",
          position: "relative",
        }}
      >
        {!pluginId && (
          <p style={{ color: "#555", textAlign: "center", paddingTop: "2rem" }}>
            No plugin selected — go back and pick a game
          </p>
        )}

        {/* Game Over overlay */}
        {gameOver && (
          <div style={overlayStyle}>
            <h2 style={{ margin: 0, fontSize: "2rem" }}>Game Over</h2>
            <p style={{ fontSize: "1.25rem", margin: "0.5rem 0" }}>
              Score: <strong>{gameOver.score}</strong>
            </p>
            <p style={{ fontSize: "0.85rem", color: "#aaa", margin: "0.25rem 0 1rem" }}>
              Score submitted!
            </p>
            <div style={{ display: "flex", gap: "0.75rem" }}>
              <button
                onClick={() => navigate(`/scores?game=${encodeURIComponent(gameOver.game_id)}`)}
                style={overlayBtnStyle}
              >
                🏆 High Scores
              </button>
              <button onClick={handleBack} style={overlayBtnStyle}>
                ← Menu
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

const btnStyle: React.CSSProperties = {
  background: "#222",
  color: "#fff",
  border: "1px solid #444",
  padding: "0.5rem 1rem",
  borderRadius: "6px",
  cursor: "pointer",
  fontSize: "0.9rem",
};

const overlayStyle: React.CSSProperties = {
  position: "absolute",
  inset: 0,
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  background: "rgba(0, 0, 0, 0.80)",
  color: "#fff",
  zIndex: 10,
};

const overlayBtnStyle: React.CSSProperties = {
  background: "#333",
  color: "#fff",
  border: "1px solid #555",
  padding: "0.6rem 1.2rem",
  borderRadius: "6px",
  cursor: "pointer",
  fontSize: "1rem",
};
