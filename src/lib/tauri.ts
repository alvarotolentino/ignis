import { invoke } from "@tauri-apps/api/core";

// ── Player types ─────────────────────────────────────────────

export interface Player {
  id: number;
  name: string;
  avatar_id: number;
  created_at: string;
}

export const listPlayers = () => invoke<Player[]>("list_players");

export const createPlayer = (name: string, avatarId: number) =>
  invoke<Player>("create_player", { name, avatarId });

export const deletePlayer = (id: number) =>
  invoke<void>("delete_player", { id });

// ── Plugin / Game types ──────────────────────────────────────

export interface PluginManifest {
  game: {
    name: string;
    version: string;
    author: string;
    igi_version: string;
  };
  display?: { resolution?: { width: number; height: number } };
  rendering?: { tier?: string };
}

export interface DiscoveredPlugin {
  id: string;
  manifest: PluginManifest;
}

export const listPlugins = () =>
  invoke<DiscoveredPlugin[]>("list_discovered_plugins");

// ── Score types ──────────────────────────────────────────────

export interface ScoreEntry {
  player_name: string;
  score: number;
  achieved_at: string;
}

export const submitScore = (playerId: number, gameId: string, score: number) =>
  invoke<void>("submit_score", { playerId, gameId, score });

export const getHighScores = (gameId: string, limit: number) =>
  invoke<ScoreEntry[]>("get_high_scores", { gameId, limit });

// ── Keybinding types ─────────────────────────────────────────

export interface Keybinding {
  action: string;
  device_type: string;
  binding: string;
}

export const getKeybindings = (playerId: number) =>
  invoke<Keybinding[]>("get_keybindings", { playerId });

export const setKeybinding = (
  playerId: number,
  action: string,
  deviceType: string,
  binding: string,
) => invoke<void>("set_keybinding", { playerId, action, deviceType, binding });

export const resetKeybindings = (playerId: number) =>
  invoke<Keybinding[]>("reset_keybindings", { playerId });
