use glam::{vec2, Vec2};
use std::time::{Duration, SystemTime};

pub const SCALE_FACTOR: f32 = 1.5;

pub use egui::Event;
pub use egui::PointerButton as PtrButton;

#[derive(Default)]
pub struct InputState(egui::RawInput);
impl InputState {
    pub fn on_event(&mut self, mut event: Event) {
        self.0.events.push(event);
    }

    pub fn take(&mut self) -> egui::RawInput {
        self.0.take()
    }
}

pub struct Zoom {
    start_dist: f32,
    prev_dist: f32,
    anchor: Vec2,
}

fn egui_pos(pos: Vec2) -> egui::Pos2 {
    egui::Pos2::new(pos.x / SCALE_FACTOR, pos.y / SCALE_FACTOR)
}
fn egui_vec(pos: Vec2) -> egui::Vec2 {
    egui::Vec2::new(pos.x / SCALE_FACTOR, pos.y / SCALE_FACTOR)
}
fn egui_ptr_button(idx: u32) -> egui::PointerButton {
    match idx {
        0 => egui::PointerButton::Primary,
        1 => egui::PointerButton::Secondary,
        2 => egui::PointerButton::Middle,
        3 => egui::PointerButton::Extra1,
        4 => egui::PointerButton::Extra2,
        _ => egui::PointerButton::Extra2,
    }
}

pub struct TouchTranslater {
    ignore_release: bool,
    last_press_time: SystemTime,
    last_pos: Vec2,
    press_pos: Option<Vec2>,
    holding: bool,
    pointer_count: u32,
    pointers: Vec<Option<Ptr>>,
    zoom: Option<Zoom>,
    wants_zoom: bool,
}
impl Default for TouchTranslater {
    fn default() -> Self {
        Self {
            ignore_release: false,
            last_press_time: SystemTime::UNIX_EPOCH,
            last_pos: Vec2::ZERO,
            press_pos: None,
            holding: false,
            pointer_count: 0,
            pointers: vec![],
            zoom: None,
            wants_zoom: false,
        }
    }
}
impl TouchTranslater {
    pub fn update(&mut self, mut out: impl FnMut(Event)) {
        if self.holding
            && SystemTime::now()
                .duration_since(self.last_press_time)
                .unwrap_or(Duration::ZERO)
                .as_millis()
                > 500
        {
            // out(Event::Click(self.last_pos, PtrButton::RIGHT));
            out(Event::PointerButton {
                pos: egui_pos(self.last_pos),
                button: PtrButton::Secondary,
                pressed: true,
                modifiers: Default::default(),
            });
            out(Event::PointerButton {
                pos: egui_pos(self.last_pos),
                button: PtrButton::Secondary,
                pressed: false,
                modifiers: Default::default(),
            });
            self.ignore_release = true;
            self.holding = false;
        }
    }

    pub fn phase_start(&mut self, idx: usize, pos: Vec2, mut out: impl FnMut(Event)) {
        self.pointer_count += 1;
        if idx >= self.pointers.len() {
            self.pointers.resize(idx + 1, None);
        }
        self.pointers[idx] = Some(Ptr {
            pos,
            id: idx as u32,
        });

        if self.pointer_count == 2 && self.wants_zoom {
            self.press_pos = None;
            self.ignore_release = true;
            self.holding = false;

            // out(Event::PointersLeft);
            // out(Event::Release(idx, PtrButton::LEFT));
            out(Event::PointerGone);

            let mut pointers = self.pointers.iter().cloned().flatten();
            let [a, b] = [pointers.next().unwrap(), pointers.next().unwrap()];
            let dist = a.pos.distance_squared(b.pos);
            let (min, max) = (a.pos.min(b.pos), a.pos.max(b.pos));
            let anchor = min + (max - min) * 0.5;
            self.zoom = Some(Zoom {
                start_dist: dist,
                prev_dist: dist,
                anchor,
            });
        } else {
            out(Event::PointerMoved(egui_pos(pos)));
            out(Event::PointerButton {
                pos: egui_pos(pos),
                button: PtrButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            });
            // out(Event::Hover(idx, pos));
            // out(Event::Press(idx, pos, PtrButton::LEFT));

            self.last_pos = pos;
            self.last_press_time = SystemTime::now();
            self.press_pos = Some(pos);
            self.holding = true;
            self.ignore_release = false;
        }
    }

    pub fn phase_move(&mut self, idx: usize, pos: Vec2, mut out: impl FnMut(Event)) {
        self.last_pos = pos;
        if self.pointer_count == 1 {
            out(Event::PointerMoved(egui_pos(pos)));
        }

        if let Some(press_pos) = self.press_pos {
            let press_dist = press_pos.distance_squared(pos).abs();
            if press_dist >= 50.0 / SCALE_FACTOR * 50.0 / SCALE_FACTOR {
                self.holding = false;
                self.press_pos = None;
            }
        }
        if let Some(Some(ptr)) = self.pointers.get_mut(idx) {
            ptr.pos = pos;
        }
        if let Some(zoom) = &mut self.zoom {
            let mut pointers = self.pointers.iter().cloned().flatten();
            let [a, b] = [pointers.next().unwrap(), pointers.next().unwrap()];
            let dist = a.pos.distance_squared(b.pos);
            if dist != zoom.start_dist {
                let delta = (dist - zoom.prev_dist) * 0.0003;
                out(Event::Zoom(delta));
            }
            zoom.prev_dist = dist;
        }
    }

    pub fn phase_end(&mut self, idx: usize, pos: Vec2, mut out: impl FnMut(Event)) {
        // out(Event::Release(idx, PtrButton::LEFT));
        out(Event::PointerButton {
            pos: egui_pos(pos),
            button: PtrButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });

        // If we've been holding the pointer still and have not
        // triggered a right click, we should send a left click
        if !self.ignore_release && self.holding {
            // out(Event::Click(pos, PtrButton::LEFT));
            // out(Event::PointerButton {
            //     pos: egui_pos(self.last_pos),
            //     button: PtrButton::Primary,
            //     pressed: true,
            //     modifiers: Default::default(),
            // });
            // out(Event::PointerButton {
            //     pos: egui_pos(self.last_pos),
            //     button: PtrButton::Primary,
            //     pressed: false,
            //     modifiers: Default::default(),
            // });
        }
        self.press_pos = None;
        self.holding = false;
        // out(Event::PointerLeft(idx));
        out(Event::PointerGone);

        if self.pointer_count == 2 {
            self.zoom = None;
        }

        if idx < self.pointers.len() {
            self.pointers[idx] = None;
            self.pointer_count -= 1;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ptr {
    pub pos: Vec2,
    pub id: u32,
}
