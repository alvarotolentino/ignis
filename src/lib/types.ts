/** One of the 8 canonical input actions — matches Rust `Action` enum. */
export type Action =
  | "Up"
  | "Down"
  | "Left"
  | "Right"
  | "ActionA"
  | "ActionB"
  | "Start"
  | "Select";

/** Input event: an action was pressed or released. */
export interface InputEvent {
  action: Action;
  pressed: boolean;
}
