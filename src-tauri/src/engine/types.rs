use serde::{Deserialize, Serialize};

/// One of the 8 canonical input actions in Ignis.
/// Plugins never see raw hardware events — only these actions.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    ActionA,
    ActionB,
    Start,
    Select,
}

impl Action {
    /// Maps an `Action` to the u32 convention used by the WASM ABI.
    pub fn to_u32(&self) -> u32 {
        match self {
            Action::Up => 0,
            Action::Down => 1,
            Action::Left => 2,
            Action::Right => 3,
            Action::ActionA => 4,
            Action::ActionB => 5,
            Action::Start => 6,
            Action::Select => 7,
        }
    }

    /// Parses an `Action` from a string sent by the frontend.
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "Up" => Some(Action::Up),
            "Down" => Some(Action::Down),
            "Left" => Some(Action::Left),
            "Right" => Some(Action::Right),
            "ActionA" => Some(Action::ActionA),
            "ActionB" => Some(Action::ActionB),
            "Start" => Some(Action::Start),
            "Select" => Some(Action::Select),
            _ => None,
        }
    }
}

/// Input event combining an action with pressed/released state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputEvent {
    pub action: Action,
    pub pressed: bool,
}

/// A single draw command emitted by a plugin each frame.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::enum_variant_names)] // Draw prefix is intentional — matches IGI host API naming
pub enum RenderCommand {
    DrawRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: u32,
    },
    DrawSprite {
        id: u32,
        x: f32,
        y: f32,
    },
    DrawText {
        text: String,
        x: f32,
        y: f32,
        size: u8,
    },
    PlaySound {
        id: u32,
    },
}

/// A complete frame of render commands sent to the frontend each tick.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RenderFrame {
    pub commands: Vec<RenderCommand>,
}

/// Metadata describing a game plugin.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)] // Constructed in Phase II for plugin metadata exchange
pub struct GameMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub resolution: (u32, u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_u32_roundtrip() {
        let actions = [
            Action::Up,
            Action::Down,
            Action::Left,
            Action::Right,
            Action::ActionA,
            Action::ActionB,
            Action::Start,
            Action::Select,
        ];
        for (i, action) in actions.iter().enumerate() {
            assert_eq!(action.to_u32(), i as u32);
        }
    }

    #[test]
    fn action_from_str() {
        assert_eq!(Action::from_str_name("Up"), Some(Action::Up));
        assert_eq!(Action::from_str_name("ActionA"), Some(Action::ActionA));
        assert_eq!(Action::from_str_name("invalid"), None);
    }

    #[test]
    fn render_frame_serialization() {
        let frame = RenderFrame {
            commands: vec![
                RenderCommand::DrawRect {
                    x: 10.0,
                    y: 20.0,
                    w: 100.0,
                    h: 50.0,
                    color: 0xFF0000FF,
                },
                RenderCommand::DrawText {
                    text: "Hello".into(),
                    x: 10.0,
                    y: 80.0,
                    size: 16,
                },
            ],
        };
        let json = serde_json::to_string(&frame).expect("serialize");
        assert!(json.contains("DrawRect"));
        assert!(json.contains("DrawText"));
    }
}
