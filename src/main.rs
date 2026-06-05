//! Rancer - Digital Art Application
//!
//! Main entry point for the application.

use rancer::logger;
use rancer::preferences;
use rancer::window;

fn main() {
    let prefs = preferences::load();
    logger::info(&format!(
        "Config file: {:?}",
        preferences::get_config_path()
    ));

    window::run_app(prefs);
}