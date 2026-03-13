wit_bindgen::generate!({
    path: "wit",
    world: "ignis-game",
});

struct HelloWorld;

/// Simple tick counter for animation.
static mut TICK: u32 = 0;
/// Persistent counter loaded from / saved to storage.
static mut COUNTER: u32 = 0;

impl Guest for HelloWorld {
    fn init() {
        // Load persisted counter from storage (or default to 0)
        let stored = ignis::game::host_api::get_storage("counter");
        let val = stored
            .as_deref()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        unsafe { COUNTER = val; }
    }

    fn update(_delta_ms: u32) {
        let tick = unsafe { TICK };
        unsafe { TICK += 1; }

        // Increment persistent counter and save every 60 frames (~1s)
        unsafe { COUNTER += 1; }
        let counter = unsafe { COUNTER };
        if tick % 60 == 0 {
            ignis::game::host_api::set_storage("counter", &counter.to_string());
        }

        // Animated background bar
        let bar_x = ((tick as f32) * 0.5) % 320.0;
        ignis::game::host_api::draw_rect(bar_x, 200.0, 60.0, 10.0, 0x333366FF);

        // Red rectangle
        ignis::game::host_api::draw_rect(10.0, 10.0, 100.0, 50.0, 0xFF0000FF);

        // Green rectangle
        ignis::game::host_api::draw_rect(120.0, 10.0, 80.0, 40.0, 0x00FF00FF);

        // Blue rectangle with 50% alpha
        ignis::game::host_api::draw_rect(210.0, 10.0, 60.0, 60.0, 0x0000FF80);

        // Yellow border-style rectangle
        ignis::game::host_api::draw_rect(10.0, 70.0, 200.0, 2.0, 0xFFFF00FF);

        // Title text
        ignis::game::host_api::draw_text("Hello Ignis!", 10.0, 80.0, 16);

        // Persistent counter display
        let counter_text = format!("Counter: {counter}");
        ignis::game::host_api::draw_text(&counter_text, 10.0, 100.0, 12);

        // Subtitle with smaller size
        ignis::game::host_api::draw_text("Phase II - Standard Wave", 10.0, 125.0, 10);

        // Footer
        ignis::game::host_api::draw_text("Press ESC to return to menu", 10.0, 220.0, 8);
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
