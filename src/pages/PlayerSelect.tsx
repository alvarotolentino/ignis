import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Player, listPlayers, createPlayer, deletePlayer } from "../lib/tauri";

const AVATAR_COLORS = ["#e74c3c", "#3498db", "#2ecc71", "#f1c40f", "#9b59b6", "#e67e22"];

/** Player Select — create, list, delete player profiles. */
export default function PlayerSelect() {
  const navigate = useNavigate();
  const [players, setPlayers] = useState<Player[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [avatarId, setAvatarId] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const refresh = () => {
    listPlayers()
      .then(setPlayers)
      .catch((e) => setError(String(e)));
  };

  useEffect(refresh, []);

  const handleCreate = async () => {
    if (!name.trim()) return;
    try {
      await createPlayer(name.trim(), avatarId);
      setName("");
      setAvatarId(0);
      setShowForm(false);
      setError(null);
      refresh();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await deletePlayer(id);
      setError(null);
      refresh();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div style={{ paddingTop: "2rem" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "1rem", marginBottom: "1.5rem" }}>
        <button onClick={() => navigate("/")} style={btnStyle}>← Back</button>
        <h2 style={{ margin: 0 }}>Player Select</h2>
      </div>

      {error && <p style={{ color: "#e74c3c" }}>{error}</p>}

      {/* Player list */}
      {players.length === 0 && !showForm && (
        <p style={{ color: "#888" }}>No players yet. Create one!</p>
      )}

      <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", marginBottom: "1.5rem" }}>
        {players.map((p) => (
          <div
            key={p.id}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "0.75rem",
              padding: "0.5rem 0.75rem",
              background: "#16161e",
              borderRadius: 4,
            }}
          >
            {/* Avatar circle */}
            <div
              style={{
                width: 28,
                height: 28,
                borderRadius: "50%",
                background: AVATAR_COLORS[p.avatar_id % AVATAR_COLORS.length],
                flexShrink: 0,
              }}
            />
            <span style={{ flex: 1 }}>{p.name}</span>
            <span style={{ color: "#666", fontSize: "0.8rem" }}>
              {new Date(p.created_at + "Z").toLocaleDateString()}
            </span>
            <button onClick={() => handleDelete(p.id)} style={{ ...btnStyle, color: "#e74c3c" }}>
              ✕
            </button>
          </div>
        ))}
      </div>

      {/* New player form */}
      {showForm ? (
        <div style={{ background: "#16161e", padding: "1rem", borderRadius: 4 }}>
          <p style={{ marginTop: 0 }}>New Player</p>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Name"
            maxLength={20}
            onKeyDown={(e) => e.key === "Enter" && handleCreate()}
            style={inputStyle}
          />
          <div style={{ display: "flex", gap: "0.5rem", margin: "0.75rem 0" }}>
            {AVATAR_COLORS.map((c, i) => (
              <div
                key={i}
                onClick={() => setAvatarId(i)}
                style={{
                  width: 32,
                  height: 32,
                  borderRadius: "50%",
                  background: c,
                  cursor: "pointer",
                  border: avatarId === i ? "2px solid #fff" : "2px solid transparent",
                }}
              />
            ))}
          </div>
          <div style={{ display: "flex", gap: "0.5rem" }}>
            <button onClick={handleCreate} style={btnStyle}>Create</button>
            <button onClick={() => setShowForm(false)} style={btnStyle}>Cancel</button>
          </div>
        </div>
      ) : (
        <button onClick={() => setShowForm(true)} style={btnStyle}>+ New Player</button>
      )}
    </div>
  );
}

const btnStyle: React.CSSProperties = {
  background: "none",
  border: "1px solid #444",
  color: "#fff",
  padding: "0.4rem 0.8rem",
  borderRadius: 4,
  cursor: "pointer",
  fontFamily: "monospace",
  fontSize: "0.9rem",
};

const inputStyle: React.CSSProperties = {
  background: "#0a0a0f",
  border: "1px solid #444",
  color: "#fff",
  padding: "0.4rem 0.6rem",
  borderRadius: 4,
  fontFamily: "monospace",
  fontSize: "0.9rem",
  width: "100%",
  boxSizing: "border-box",
};
