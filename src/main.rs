use relm4::prelude::*;

pub mod ui;

use crate::ui::main_window;

fn main() {
    init_tracing();

    let app = RelmApp::new("relm4.example.simple");
    app.run::<main_window::App>(0);
}

fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(log_filter))
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .ok();
}
