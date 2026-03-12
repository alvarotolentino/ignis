wit_bindgen::generate!({
    path: "wit",
    world: "ignis-game",
});

struct HelloWorld;

impl Guest for HelloWorld {
    fn init() {
        // Nothing to initialize
    }

    fn update(_delta_ms: u32) {
        // Draw a red rectangle
        ignis::game::host_api::draw_rect(10.0, 10.0, 100.0, 50.0, 0xFF0000FF);

        // Draw greeting text
        ignis::game::host_api::draw_text("Hello Ignis!", 10.0, 80.0, 16);
    }

    fn handle_input(_action: u32) {
        // No-op for hello world
    }

    fn get_name() -> String {
        "Hello World".to_string()
    }

    fn get_version() -> String {
        "0.1.0".to_string()
    }

    fn get_author() -> String {
        "Ignis Team".to_string()
    }
}

export!(HelloWorld);
