pub mod app;
pub mod graphics;
pub mod input;
pub mod inv;
pub mod ui;

use crate::app::App;
use crate::graphics::Graphics;
use glam::vec2;

use core::ffi::c_void;
use objc::runtime::Object;

pub struct UiViewObject(pub *mut Object);
pub struct CaMetalLayer(pub *mut c_void);

#[repr(C)]
struct CreateAppArgs {
    view: UiViewObject,
    metal_layer: CaMetalLayer,
    maximum_frames: i32,
    swift_callback: extern "C" fn(u32),
    open_keyboard: extern "C" fn(),
    close_keyboard: extern "C" fn(),
}

pub static mut OPEN_KEYBOARD: Option<extern "C" fn()> = None;
pub static mut CLOSE_KEYBOARD: Option<extern "C" fn()> = None;
pub fn open_keyboard() {
    unsafe { OPEN_KEYBOARD.unwrap()() }
}
pub fn close_keyboard() {
    unsafe { CLOSE_KEYBOARD.unwrap()() }
}

#[no_mangle]
pub extern "C" fn create_app(args: CreateAppArgs) -> *mut libc::c_void {
    unsafe {
        OPEN_KEYBOARD = Some(args.open_keyboard);
        CLOSE_KEYBOARD = Some(args.close_keyboard);
    }
    println!("create_app, maximum frames: {}", args.maximum_frames);
    let app = App::new(Graphics::new(args.view, args.metal_layer));
    Box::into_raw(Box::new(app)) as *mut libc::c_void
}

#[no_mangle]
pub extern "C" fn draw_frame(app: *mut libc::c_void) {
    let app = unsafe { &mut *(app as *mut App) };
    app.draw_frame();
}

#[no_mangle]
pub extern "C" fn event_touch_begin(app: *mut libc::c_void, x: f32, y: f32) {
    let app = unsafe { &mut *(app as *mut App) };
    app.translater
        .phase_start(0, vec2(x, y), |e| app.input.on_event(e));
}
#[no_mangle]
pub extern "C" fn event_touch_move(app: *mut libc::c_void, x: f32, y: f32) {
    let app = unsafe { &mut *(app as *mut App) };
    app.translater
        .phase_move(0, vec2(x, y), |e| app.input.on_event(e));
}
#[no_mangle]
pub extern "C" fn event_touch_end(app: *mut libc::c_void, x: f32, y: f32) {
    let app = unsafe { &mut *(app as *mut App) };
    app.translater
        .phase_end(0, vec2(x, y), |e| app.input.on_event(e));
}
#[no_mangle]
pub extern "C" fn event_text_input(app: *mut libc::c_void, bytes: *const u8, bytes_len: u32) {
    let app = unsafe { &mut *(app as *mut App) };
    let bytes = unsafe { std::slice::from_raw_parts(bytes, bytes_len as usize) };
    let text = String::from_utf8_lossy(bytes).to_string();
    app.input.on_event(egui::Event::Text(text));
}
#[no_mangle]
pub extern "C" fn event_key_typed_backspace(app: *mut libc::c_void) {
    let app = unsafe { &mut *(app as *mut App) };
    app.input.on_event(egui::Event::Key {
        key: egui::Key::Backspace,
        physical_key: None,
        pressed: true,
        modifiers: Default::default(),
        repeat: false,
    });
    app.input.on_event(egui::Event::Key {
        key: egui::Key::Backspace,
        physical_key: None,
        pressed: false,
        modifiers: Default::default(),
        repeat: false,
    });
}
