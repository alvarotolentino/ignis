use crate::engine::types::{Action, InputEvent};

/// Receives an input event from the frontend and pushes it into the engine's
/// input queue for processing on the next game loop tick.
#[tauri::command]
pub fn send_input(
    action: String,
    pressed: bool,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let action = Action::from_str_name(&action)
        .ok_or_else(|| format!("Unknown action: {action}"))?;

    state
        .engine
        .input_queue
        .lock()
        .unwrap()
        .push_back(InputEvent { action, pressed });

    Ok(())
}
