use crate::inv::{InvChange, LocalInv};
use crate::ui::{HomePage, Page, TextFieldInfo, UiOutput, UiTheme};
use crate::SaveDirs;

use jano::android_activity::input::{TextInputState, TextSpan};
use jano::egui_app::Egui;
use jano::TcpStream;
use jano::{android, egui, log, set_keyboard_visibility, FrameStats};
use serde::{Deserialize, Serialize};

use inv_common::{ServerConn, ServerErr};

type Server = ServerConn<TcpStream>;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub server_address: String,
    pub server_port: u32,
    pub theme: UiTheme,
    pub name: String,
    pub scale: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            server_address: "192.168.1.239".into(),
            server_port: 25552,
            theme: UiTheme::default(),
            name: fastrand::u32(0..1000).to_string(),
            scale: 3.0,
        }
    }
}

pub struct App {
    save_dirs: SaveDirs,
    server: Option<Server>,
    pub focused_text_field: Option<TextFieldInfo>,
    pub settings: Settings,
    pub inv: LocalInv,
    pages: Option<Vec<Box<dyn Page>>>,
}
impl Default for App {
    fn default() -> Self {
        Self {
            save_dirs: SaveDirs::new(),
            server: None,
            focused_text_field: None,
            settings: Default::default(),
            inv: Default::default(),
            pages: Some(vec![Box::<HomePage>::default()]),
        }
    }
}
impl App {
    pub fn msg_popup(&mut self, msg: impl Into<String>) {
        if let Err(err) = jano::show_toast(msg, true) {
            eprintln!("Failed to show toast: {err:?}");
        }
    }

    pub fn sync_server(&mut self, download: bool) {
        if self.server.is_none() {
            return;
        }
        if let Err(err) = self.try_sync_server(download) {
            self.msg_popup(format!("Failed to sync server : {}", err));
            self.server = None;
        } else {
            self.msg_popup("Successfully synced with server");
        }
    }
    pub fn try_sync_server(&mut self, download: bool) -> Result<(), ServerErr> {
        let Some(server) = &mut self.server else {
            return Ok(());
        };

        // before obtaining inv data from server, make sure we have updated the server of any changes we have made locally
        for change in self.inv.consume_changes() {
            match change {
                InvChange::AddedItem(id) | InvChange::ModifiedItem(id) => {
                    let item = self.inv.get_item(&id).unwrap();
                    server.insert_item(id, item).map_err(ServerErr::OtherIo)?
                }
                InvChange::DeletedItem(id) => server.remove_item(id).map_err(ServerErr::OtherIo)?,
            };
        }

        if download {
            let server_inv = server.get_inv()?;
            self.inv.r#override(server_inv);
        }
        Ok(())
    }

    pub fn try_connect_to_server(&self) -> Result<Server, ServerErr> {
        let stream = TcpStream::connect_timeout(
            &std::net::SocketAddr::from((
                self.settings
                    .server_address
                    .parse::<std::net::IpAddr>()
                    .unwrap(),
                self.settings.server_port as u16,
            )),
            std::time::Duration::from_secs(5),
        )?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(3)))?;
        Server::connect(stream, &self.settings.name)
    }

    pub fn connect_to_server(&mut self) {
        match self.try_connect_to_server() {
            Ok(server) => {
                self.server = Some(server);
                self.msg_popup("Successfully connected to server");
            }
            Err(err) => {
                self.msg_popup(format!("Failed to connect to server : {}", err));
                self.server = None;
            }
        }
    }
}
impl jano::egui_app::EguiApp for App {
    fn on_picture_taken(&mut self, _egui: &Option<Egui>, pic: jano::Picture) {
        self.curr_page_mut().on_picture_taken(pic);
    }

    fn on_resume(&mut self) {
        if let Ok(bytes) = std::fs::read(&self.save_dirs.settings) {
            match bincode::deserialize::<Settings>(&bytes) {
                Ok(settings) => {
                    jano::set_scale_factor(settings.scale);
                    self.settings = settings;
                }
                Err(err) => log::error!("Failed to parse settings: {err:?}"),
            }
        }
        if let Ok(bytes) = std::fs::read(&self.save_dirs.inv) {
            match bincode::deserialize(&bytes) {
                Ok(inv) => self.inv = inv,
                Err(err) => log::error!("Failed to parse inv: {err:?}"),
            }
        }

        self.connect_to_server();
        self.sync_server(true);
    }

    fn on_save_state(&mut self) {
        log::info!("Saving app's state...");

        self.sync_server(false);

        let settings = bincode::serialize(&self.settings).unwrap();
        match std::fs::write(&self.save_dirs.settings, settings) {
            Ok(_) => log::info!("Saved settings to {:?}", self.save_dirs.settings),
            Err(err) => log::warn!(
                "Failed to save settings to {:?} : {err:?}",
                self.save_dirs.settings
            ),
        }

        let inv = bincode::serialize(&self.inv).unwrap();
        match std::fs::write(&self.save_dirs.inv, inv) {
            Ok(_) => log::info!("Saved inv to {:?}", self.save_dirs.inv),
            Err(err) => log::warn!("Failed to save inv to {:?} : {err:?}", self.save_dirs.inv),
        }
    }

    fn draw_frame(&mut self, egui: &mut Egui, ctx: &egui::Context, _stats: FrameStats) {
        let mut out = UiOutput::default();
        self.show(egui, ctx, &mut out);

        if out.focused_text_field.is_some() && self.focused_text_field.is_none() {
            log::info!("App started wanting text input ;' opening keyboard");
            let info = out.focused_text_field.as_ref().unwrap();
            if let Err(err) = set_keyboard_visibility(true) {
                self.msg_popup(format!("{err:?}"));
            }
            android().set_text_input_state(TextInputState {
                text: info.text.clone(),
                selection: TextSpan {
                    start: info.selection[0] as usize,
                    end: info.selection[1] as usize,
                },
                compose_region: Some(TextSpan {
                    start: info.selection[0] as usize,
                    end: info.selection[1] as usize,
                }),
            });
        }
        if out.focused_text_field.is_none() && self.focused_text_field.is_some() {
            log::info!("App stopped wanting text input ;' closing keyboard");
            if let Err(err) = set_keyboard_visibility(false) {
                self.msg_popup(format!("{err:?}"));
            }
        }
        if ctx.input(|input| {
            input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::PointerButton { pressed: true, .. }))
                && out.focused_text_field.is_some()
        }) {
            // if we've tapped the screen this frame and there is a text field focused, open keyboard.
            // This is in case the user has closed the keyboard with the back button
            if let Err(err) = set_keyboard_visibility(true) {
                self.msg_popup(format!("{err:?}"));
            }
        }
        self.focused_text_field = out.focused_text_field;

        if let Some(text) = out.copy_text {
            if let Err(err) = jano::set_clipboard_content(&text) {
                log::warn!("Error copying text: {err:?}");
            }
        }
        if out.trigger_paste_cmd {
            let text = jano::get_clipboard_content();
            log::info!("Adding Text({text:?}) event to queue");
            // self.input.0.events.push(egui::Event::Text(text));
        }
        if out.reconnect_to_server {
            self.connect_to_server();
            self.sync_server(true);
        }
        if out.sync_server {
            self.sync_server(true);
        }
    }
}
impl App {
    pub fn curr_page_mut(&mut self) -> &mut Box<dyn Page> {
        self.pages.as_mut().unwrap().last_mut().unwrap()
    }

    pub fn show(&mut self, egui: &mut Egui, ctx: &egui::Context, out: &mut UiOutput) {
        let pages = self.pages.as_mut().unwrap();
        self.settings.theme.apply(ctx);

        egui::TopBottomPanel::top("app_header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if pages.last().unwrap().has_back_button() && ui.button("<").clicked {
                    pages.pop();
                }
                ui.heading(pages.last().unwrap().title());
            });
        });
        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            if self.server.is_none() {
                ui.horizontal(|ui| {
                    ui.label("Failed to connect to server");
                    if ui.button("retry").clicked {
                        out.reconnect_to_server = true;
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    if ui.button("sync").clicked {
                        out.sync_server = true;
                    }
                });
            }
        });
        let mut pages = self.pages.take().unwrap();
        egui::CentralPanel::default().show(ctx, |ui| {
            pages.last_mut().unwrap().show(ui, out, self);
            pages.last_mut().unwrap().show2(ui, out, self, egui);
        });

        if out.pop_page {
            _ = pages.pop();
        }
        if let Some(page) = out.push_page.take() {
            pages.push(page);
        }
        self.pages = Some(pages);
    }
}
