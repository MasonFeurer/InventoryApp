pub mod app;
pub mod inv;
pub mod ui;

use app::App;
use jano::{android, android_activity::AndroidApp, egui_app::EguiAppState};
use std::path::PathBuf;

#[no_mangle]
fn android_main(android: AndroidApp) {
    jano::init_android(android.clone());
    jano::android_main::<EguiAppState<App>>(android, EguiAppState::new(App::default()));
}

struct SaveDirs {
    settings: PathBuf,
    inv: PathBuf,
}
impl SaveDirs {
    fn new() -> Self {
        let dir = android().external_data_path().unwrap();
        Self {
            settings: dir.join("settings.data"),
            inv: dir.join("inv.data"),
        }
    }
}
