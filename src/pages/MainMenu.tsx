import { useEffect, useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { listPlugins, type DiscoveredPlugin } from "../lib/tauri";

/** Main Menu — game selector, navigation to Players/Settings. */
export default function MainMenu() {
  const [plugins, setPlugins] = useState<DiscoveredPlugin[]>([]);
  const [selected, setSelected] = useState(0);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    listPlugins()
      .then((list) => {
        setPlugins(list);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to list plugins:", err);
        setLoading(false);
      });
  }, []);

  const launch = useCallback(
    (idx: number) => {
      const p = plugins[idx];
      if (p) navigate(`/play?plugin=${encodeURIComponent(p.id)}`);
    },
    [plugins, navigate]
  );

  // Keyboard navigation
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (plugins.length === 0) return;
      if (e.code === "ArrowUp" || e.code === "KeyW") {
        e.preventDefault();
        setSelected((s) => (s - 1 + plugins.length) % plugins.length);
      } else if (e.code === "ArrowDown" || e.code === "KeyS") {
        e.preventDefault();
        setSelected((s) => (s + 1) % plugins.length);
      } else if (e.code === "Enter") {
        e.preventDefault();
        launch(selected);
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [plugins, selected, launch]);

  return (
    <div style={{ textAlign: "center", paddingTop: "2rem" }}>
      <h1
        style={{
          fontSize: "2.5rem",
          letterSpacing: "0.3em",
          marginBottom: "2rem",
        }}
      >
        IGNIS
      </h1>

      {/* Game list */}
      <div style={{ maxWidth: "480px", margin: "0 auto", textAlign: "left" }}>
        {loading && <p style={{ color: "#888" }}>Loading games…</p>}
        {!loading && plugins.length === 0 && (
          <p style={{ color: "#888" }}>
            No games found. Add plugins to the <code>plugins/</code> directory.
          </p>
        )}
        {plugins.map((p, i) => (
          <div
            key={p.id}
            onClick={() => {
              setSelected(i);
              launch(i);
            }}
            style={{
              padding: "0.8rem 1rem",
              marginBottom: "0.5rem",
              borderRadius: "8px",
              cursor: "pointer",
              border:
                i === selected ? "2px solid #0af" : "2px solid transparent",
              background: i === selected ? "#1a1a2e" : "#111",
              transition: "background 0.15s, border-color 0.15s",
            }}
          >
            <div
              style={{ fontSize: "1.1rem", fontWeight: 600, color: "#eee" }}
            >
              {p.manifest.game.name}
            </div>
            <div style={{ fontSize: "0.8rem", color: "#888", marginTop: "2px" }}>
              {p.manifest.game.author} · v{p.manifest.game.version}
            </div>
          </div>
        ))}
      </div>

      {/* Navigation buttons */}
      <div style={{ marginTop: "2rem", display: "flex", gap: "1rem", justifyContent: "center" }}>
        <button onClick={() => navigate("/players")} style={btnStyle}>
          Players
        </button>
        <button onClick={() => navigate("/settings")} style={btnStyle}>
          Settings
        </button>
      </div>
    </div>
  );
}

const btnStyle: React.CSSProperties = {
  background: "#222",
  color: "#fff",
  border: "1px solid #444",
  padding: "0.5rem 1.5rem",
  borderRadius: "6px",
  cursor: "pointer",
  fontSize: "0.9rem",
};
