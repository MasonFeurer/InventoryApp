use crate::app::App;
use crate::inv::{to_jano_pic, Id, Inv, Item, Listing, Listings, Platform, Usd};

use jano::egui::{self, Response, ScrollArea, Ui};
use jano::egui_app::Egui;
use serde::{Deserialize, Serialize};

use std::time::SystemTime;

#[derive(Clone, Default, Debug)]
pub struct TextFieldInfo {
    pub text: String,
    pub selection: [u32; 2],
}
impl TextFieldInfo {
    pub fn new(text: impl Into<String>, selection: [u32; 2]) -> Self {
        let text = text.into();
        Self { text, selection }
    }
}
pub fn text_edit(ui: &mut Ui, output: &mut UiOutput, text: &mut String) -> Response {
    let rs = ui.text_edit_singleline(text);
    if rs.has_focus() {
        output.focused_text_field = Some(TextFieldInfo::new(text.clone(), [0, 0]));
    }
    rs
}

#[derive(Default, Clone, Copy, Debug)]
pub enum ItemSort {
    #[default]
    Name,
    Price,
    Count,
    Location,
}

#[derive(PartialEq)]
pub enum ItemFilter {
    ZeroCost,
    NotSold,
    AnySold,
    SoldOut,
    NotListed,
    Listed(Platform),
    Location(String),
}
impl ItemFilter {
    pub fn all_options(inv: &Inv) -> Vec<Self> {
        let mut out = vec![
            ItemFilter::ZeroCost,
            ItemFilter::NotSold,
            ItemFilter::AnySold,
            ItemFilter::SoldOut,
            ItemFilter::NotListed,
        ];
        out.extend(
            (0..inv.platform_names.len() as u8)
                .map(|i| Self::Listed(Platform::from_idx(i).unwrap())),
        );
        out.extend(inv.all_locations().map(String::from).map(Self::Location));
        out
    }

    pub fn display(&self, inv: &Inv) -> String {
        match self {
            Self::ZeroCost => "Zero cost".into(),
            Self::NotSold => "Not sold".into(),
            Self::AnySold => "Any sold".into(),
            Self::SoldOut => "All sold".into(),
            Self::NotListed => "Not listed".into(),
            Self::Listed(platform) => inv.platform_names[platform.as_idx() as usize].clone(),
            Self::Location(l) => l.clone(),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum SortDir {
    Up,
    #[default]
    Down,
}
impl SortDir {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Up => "Up",
            Self::Down => "Dn",
        }
    }
}

#[derive(Default)]
pub struct UiOutput {
    pub text_field_has_focus: bool,
    pub copy_text: Option<String>,
    pub trigger_paste_cmd: bool,
    pub reconnect_to_server: bool,
    pub sync_server: bool,
    pub focused_text_field: Option<TextFieldInfo>,
    pub pop_page: bool,
    pub push_page: Option<Box<dyn Page>>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
pub enum UiTheme {
    #[default]
    Light,
    Dark,
    Night,
}
impl std::fmt::Display for UiTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
impl UiTheme {
    pub const ALL_OPTIONS: [Self; 3] = [Self::Light, Self::Dark, Self::Night];

    pub fn apply(&self, ctx: &egui::Context) {
        ctx.set_visuals(self.create_visuals());
    }

    pub fn create_visuals(&self) -> egui::Visuals {
        use egui::*;
        pub fn dark_mode_visuals() -> Visuals {
            let mut vis = Visuals::dark();
            vis.widgets.inactive.fg_stroke.color = Color32::from_gray(255);
            vis.widgets.hovered.fg_stroke.color = Color32::from_gray(255);
            vis.widgets.active.fg_stroke.color = Color32::from_gray(255);
            vis.widgets.noninteractive.fg_stroke.color = Color32::from_gray(255);

            let idle = Color32::from_gray(100);
            let hovered = Color32::from_gray(150);
            let pressed = Color32::from_gray(200);

            vis.widgets.inactive.bg_stroke.color = idle;
            vis.widgets.inactive.bg_fill = idle;

            vis.widgets.hovered.bg_stroke.color = hovered;
            vis.widgets.hovered.bg_fill = hovered;

            vis.widgets.active.bg_stroke.color = pressed;
            vis.widgets.active.bg_fill = pressed;

            vis.widgets.noninteractive.bg_stroke.color = Color32::from_gray(200);
            vis.widgets.noninteractive.bg_fill = Color32::from_gray(180);
            vis
        }
        pub fn night_mode_visuals() -> Visuals {
            let mut vis = Visuals::dark();

            vis.widgets.inactive.fg_stroke.color = Color32::from_gray(100);
            vis.widgets.hovered.fg_stroke.color = Color32::from_gray(100);
            vis.widgets.active.fg_stroke.color = Color32::from_gray(100);
            vis.widgets.noninteractive.fg_stroke.color = Color32::from_gray(100);

            let idle = Color32::from_gray(30);
            let hovered = Color32::from_gray(40);
            let pressed = Color32::from_gray(200);

            vis.widgets.inactive.bg_stroke.color = idle;
            vis.widgets.inactive.bg_fill = idle;
            vis.widgets.inactive.weak_bg_fill = idle;

            vis.widgets.hovered.bg_stroke.color = hovered;
            vis.widgets.hovered.bg_fill = hovered;

            vis.widgets.active.bg_stroke.color = pressed;
            vis.widgets.active.bg_fill = pressed;

            vis.widgets.noninteractive.bg_stroke.color = Color32::from_gray(30);
            vis.widgets.noninteractive.bg_fill = Color32::from_gray(20);
            vis.panel_fill = Color32::BLACK;
            vis
        }
        let mut vis = match self {
            Self::Light => Visuals::light(),
            Self::Dark => dark_mode_visuals(),
            Self::Night => night_mode_visuals(),
        };
        let rounding = 5.0;
        vis.widgets.inactive.rounding = Rounding::same(rounding);
        vis.widgets.hovered.rounding = Rounding::same(rounding);
        vis.widgets.active.rounding = Rounding::same(rounding);
        vis.widgets.noninteractive.rounding = Rounding::same(rounding);
        vis
    }
}

pub trait Page {
    fn on_picture_taken(&mut self, _pic: jano::Picture) {}
    fn title(&self) -> String;
    #[rustfmt::skip]
    fn has_back_button(&self) -> bool { true }
    fn show(&mut self, _ui: &mut Ui, _out: &mut UiOutput, _app: &mut App) {}
    fn show2(&mut self, _ui: &mut Ui, _out: &mut UiOutput, _app: &mut App, _egui: &mut Egui) {}
}

#[derive(Default)]
pub struct HomePage {}
impl Page for HomePage {
    #[rustfmt::skip]
    fn title(&self) -> String { String::from("Home") }

    #[rustfmt::skip]
    fn has_back_button(&self) -> bool { false }
    fn show(&mut self, ui: &mut Ui, out: &mut UiOutput, _app: &mut App) {
        if ui.button("See Items").clicked {
            out.push_page = Some(Box::<ItemListPage>::default());
        }
        if ui.button("New Item").clicked {
            out.push_page = Some(Box::new(EditItemPage::new(Id::new())));
        }
        if ui.button("Settings").clicked {
            out.push_page = Some(Box::<SettingsPage>::default());
        }
        if ui.button("Stats").clicked {
            out.push_page = Some(Box::<StatsPage>::default());
        }
    }
}

#[derive(Default)]
pub struct ItemListPage {
    pub filters: Vec<ItemFilter>,
    pub sort: ItemSort,
    pub sort_dir: SortDir,
    pub search: String,
    pub scroll_offset: f32,
}
impl Page for ItemListPage {
    #[rustfmt::skip]
    fn title(&self) -> String { String::from("Items") }

    fn show2(&mut self, ui: &mut Ui, out: &mut UiOutput, app: &mut App, egui: &mut Egui) {
        if app.inv.item_count() == 0 {
            ui.heading("No items here.");
            return;
        }
        ui.horizontal(|ui| {
            ui.menu_button(format!("{:?}", self.sort), |ui| {
                if ui.button("Name").clicked {
                    self.sort = ItemSort::Name;
                    ui.close_menu();
                }
                if ui.button("Price").clicked {
                    self.sort = ItemSort::Price;
                    ui.close_menu();
                }
                if ui.button("Count").clicked {
                    self.sort = ItemSort::Count;
                    ui.close_menu();
                }
                if ui.button("Location").clicked {
                    self.sort = ItemSort::Location;
                    ui.close_menu();
                }
            });
            if ui.button(self.sort_dir.as_str()).clicked {
                self.sort_dir = match self.sort_dir {
                    SortDir::Up => SortDir::Down,
                    SortDir::Down => SortDir::Up,
                };
            }
            for idx in (0..self.filters.len()).rev() {
                if ui.button(self.filters[idx].display(&app.inv)).clicked {
                    _ = self.filters.remove(idx);
                }
            }
            ui.menu_button("+", |ui| {
                for filter in ItemFilter::all_options(&app.inv) {
                    if self.filters.contains(&filter) {
                        continue;
                    }
                    if ui.button(filter.display(&app.inv)).clicked {
                        self.filters.push(filter);
                        ui.close_menu();
                    }
                }
            });
            text_edit(ui, out, &mut self.search);
        });
        ui.separator();

        let mut items: Vec<(&Id, &Item)> = app
            .inv
            .items
            .iter()
            .filter(|(_id, item)| 'a: {
                for filter in &self.filters {
                    if match filter {
                        ItemFilter::ZeroCost => item.est_cost.0 == 0,
                        ItemFilter::NotSold => item.sold_count() == 0,
                        ItemFilter::AnySold => item.sold_count() > 0,
                        ItemFilter::SoldOut => item.sold_count() == item.count,
                        ItemFilter::NotListed => item.listings == Listings::default(),
                        ItemFilter::Listed(p) => item.listings.contains_platform(*p),
                        ItemFilter::Location(l) => item.location.as_str() == l.as_str(),
                    } {
                        break 'a true;
                    }
                }
                self.filters.is_empty()
            })
            .filter(|(id, item)| {
                if self.search.is_empty() {
                    true
                } else {
                    item.name
                        .to_lowercase()
                        .contains(&self.search.to_lowercase())
                        || format!("{id:?}")
                            .to_lowercase()
                            .contains(&self.search.to_lowercase())
                }
            })
            .collect();
        match (self.sort, self.sort_dir) {
            (ItemSort::Name, SortDir::Up) => items.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name)),
            (ItemSort::Price, SortDir::Up) => {
                items.sort_by(|(_, a), (_, b)| a.est_cost.0.cmp(&b.est_cost.0))
            }
            (ItemSort::Count, SortDir::Up) => items.sort_by(|(_, a), (_, b)| a.count.cmp(&b.count)),
            (ItemSort::Location, SortDir::Up) => {
                items.sort_by(|(_, a), (_, b)| a.location.cmp(&b.location))
            }

            (ItemSort::Name, SortDir::Down) => {
                items.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name).reverse())
            }
            (ItemSort::Price, SortDir::Down) => {
                items.sort_by(|(_, a), (_, b)| a.est_cost.0.cmp(&b.est_cost.0).reverse())
            }
            (ItemSort::Count, SortDir::Down) => {
                items.sort_by(|(_, a), (_, b)| a.count.cmp(&b.count).reverse())
            }
            (ItemSort::Location, SortDir::Down) => {
                items.sort_by(|(_, a), (_, b)| a.location.cmp(&b.location).reverse())
            }
        }

        let rs = ScrollArea::vertical()
            .vertical_scroll_offset(self.scroll_offset)
            .show(ui, |ui| {
                for (id, item) in items {
                    let pic_size = 50.0;
                    let pic_rounding = 10.0;
                    let pic_size2 = egui::vec2(pic_size, pic_size);

                    let full_w = ui.available_width();

                    let (rect, rs) =
                        ui.allocate_exact_size(egui::vec2(full_w, pic_size), egui::Sense::click());

                    let color = ui.visuals().widgets.inactive.bg_stroke.color;
                    ui.painter()
                        .rect_stroke(rect, 10.0, egui::Stroke::new(1.0, color));

                    if let Some(pic) = &item.picture {
                        let handle = egui.obtain_tex_handle_for_pic(
                            egui::Id::new(id.0),
                            &to_jano_pic(pic.clone()),
                        );
                        let sized_image = egui::load::SizedTexture::new(handle.id(), pic_size2);
                        let image = egui::Image::from_texture(sized_image).rounding(pic_rounding);
                        image.paint_at(ui, egui::Rect::from_min_size(rect.min, pic_size2));
                    }

                    let w = full_w - pic_size;
                    let sections = [w * 0.75, w * 0.10, w * 0.15];

                    let rect0 = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + pic_size, rect.min.y),
                        egui::vec2(sections[0], rect.height()),
                    );
                    let rect1 = egui::Rect::from_min_size(
                        egui::pos2(rect0.max.x, rect.min.y),
                        egui::vec2(sections[1], rect.height()),
                    );
                    let rect2 = egui::Rect::from_min_size(
                        egui::pos2(rect1.max.x, rect.min.y),
                        egui::vec2(sections[2], rect.height()),
                    );

                    let mut ui0 = ui.child_ui(rect0, ui.layout().clone(), None);
                    ui0.set_clip_rect(rect0.intersect(ui.clip_rect()));
                    ui0.add_enabled(false, egui::Label::new(&item.name));
                    // ui0.label(&item.name);

                    let mut ui1 = ui.child_ui(rect1, ui.layout().clone(), None);
                    ui1.set_clip_rect(rect1.intersect(ui.clip_rect()));
                    ui1.label(&format!("{}", item.count));

                    let mut ui2 = ui.child_ui(rect2, ui.layout().clone(), None);
                    ui2.set_clip_rect(rect2.intersect(ui.clip_rect()));
                    ui2.label(&format!("${}", item.est_cost));

                    if rs.clicked {
                        out.push_page = Some(Box::new(ItemDetailsPage(*id)));
                    }
                    ui.add_space(1.0);
                }
            });
        self.scroll_offset = rs.state.offset.y;
    }
}

pub struct ItemDetailsPage(Id);
impl Page for ItemDetailsPage {
    #[rustfmt::skip]
    fn title(&self) -> String { format!("Item Details - {}", self.0.0) }

    fn show2(&mut self, ui: &mut Ui, out: &mut UiOutput, app: &mut App, egui: &mut Egui) {
        let id = self.0;
        let pic_size = egui::vec2(50.0, 50.0);
        let pic_rounding = 10.0;
        ScrollArea::vertical().show(ui, |ui| {
            let Some(item) = app.inv.get_item(&id) else {
                out.pop_page = true;
                return;
            };

            let _pic_rs = match &item.picture {
                Some(pic) => {
                    let handle = egui
                        .obtain_tex_handle_for_pic(egui::Id::new(id.0), &to_jano_pic(pic.clone()));
                    let sized_image = egui::load::SizedTexture::new(handle.id(), pic_size);
                    let image = egui::Image::from_texture(sized_image);
                    ui.add(image.sense(egui::Sense::click()).rounding(pic_rounding))
                }
                None => {
                    let (rect, response) = ui.allocate_exact_size(pic_size, egui::Sense::click());
                    if ui.is_rect_visible(rect) {
                        ui.painter().rect_stroke(
                            rect,
                            pic_rounding,
                            egui::Stroke::new(2.0, egui::Color32::GRAY),
                        );
                    }
                    response
                }
            };

            ui.horizontal(|ui| {
                if ui.button("📋").clicked() {
                    out.copy_text = Some(item.name.clone());
                }
                ui.heading(&item.name);
            });
            ui.horizontal(|ui| {
                if ui.button("📋").clicked() {
                    out.copy_text = Some(item.desc.clone());
                }
                ui.label(&item.desc);
            });

            ui.separator();

            let mut detail = |copyable: bool, label: &str, value: &str| {
                if copyable {
                    ui.horizontal(|ui| {
                        if ui.button("📋").clicked() {
                            out.copy_text = Some(value.to_string());
                        }
                        ui.label(&format!("{label}: {value}"));
                    });
                } else {
                    ui.label(&format!("{label}: {value}"));
                }
            };

            let [w, l, h] = item.dimensions;

            detail(false, "Count", &item.count.to_string());
            detail(true, "Est. Cost", &item.est_cost.to_string());
            detail(true, "Condition", &item.condition.to_string());
            detail(true, "Dimensions", &format!("{w}x{h}x{l}in",));
            detail(true, "Weight", &format!("{}lb", item.weight));
            #[rustfmt::skip]
        	detail(true, "Shipping Weight", &format!("{}lb", item.shipping_weight));
            detail(true, "Color", &item.color);
            detail(true, "Brand", &item.brand);
            detail(true, "Location", &item.location);
            detail(true, "Model", &item.model_no.to_string());
            detail(true, "Serial", &item.serial_no.to_string());

            if item.listings.count() == 0 {
                ui.label("No listings");
            } else {
                ui.label("Listings");
                for (platform, listing) in &item.listings {
                    ui.group(|ui| {
                        ui.label(app.inv.get_platform_name(platform));
                        ui.label(format!("Listed: {}", display_date(listing.date)));
                        ui.label(format!("Sold: {}", listing.sold));
                    });
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("edit").clicked {
                    let item = app.inv.get_item(&id).unwrap().clone();
                    out.push_page = Some(Box::new(EditItemPage::new_w_item(id, item)));
                }
                if ui.button("clone").clicked {
                    out.push_page = Some(Box::new(EditItemPage::new(Id::new())));
                }
            });
        });
    }
}

pub struct ItemTemplate {
    location: String,
    listings: Listings,
    picture: Option<jano::Picture>,
    name: String,
    desc: String,
    count: String,
    est_cost: String,
    condition: String,
    color: String,
    dimensions: [String; 3],
    weight: String,
    shipping_weight: String,
    model_no: String,
    serial_no: String,
    brand: String,
}
impl ItemTemplate {
    pub fn from_item(item: Item) -> Self {
        Self {
            location: item.location,
            listings: item.listings,
            picture: item.picture.clone().map(to_jano_pic),
            name: item.name,
            desc: item.desc,
            count: item.count.to_string(),
            est_cost: item.est_cost.to_string(),
            condition: item.condition,
            color: item.color,
            dimensions: item.dimensions.map(|f| f.to_string()),
            weight: item.weight.to_string(),
            shipping_weight: item.shipping_weight.to_string(),
            model_no: item.model_no.to_string(),
            serial_no: item.serial_no.to_string(),
            brand: item.brand,
        }
    }

    #[allow(clippy::field_reassign_with_default)]
    pub fn as_item(&self) -> Result<Item, &'static str> {
        let mut item = Item::default();
        item.count = self.count.parse().map_err(|_| "Count")?;
        item.est_cost = self.est_cost.parse().map_err(|_| "Cost")?;
        item.dimensions[0] = self.dimensions[0].parse().map_err(|_| "Width")?;
        item.dimensions[1] = self.dimensions[1].parse().map_err(|_| "Height")?;
        item.dimensions[2] = self.dimensions[2].parse().map_err(|_| "Length")?;
        item.weight = self.weight.parse().map_err(|_| "Weight")?;
        item.shipping_weight = self
            .shipping_weight
            .parse()
            .map_err(|_| "Shipping Weight")?;
        item.model_no = self.model_no.parse().map_err(|_| "Model")?;
        item.serial_no = self.serial_no.parse().map_err(|_| "Serial")?;

        item.location = self.location.clone();
        item.listings = self.listings.clone();
        item.picture = self.picture.clone().map(crate::inv::to_inv_pic);
        item.name = self.name.clone();
        item.desc = self.desc.clone();
        item.condition = self.condition.clone();
        item.color = self.color.clone();
        item.brand = self.brand.clone();
        Ok(item)
    }
}

pub fn display_date(mut date: SystemTime) -> String {
    let dur = SystemTime::now().duration_since(date).unwrap();
    let dur = time::Duration::try_from(dur).unwrap();

    if dur.whole_days() > 0 && dur.whole_days() < 2 {
        return format!("{} days ago", dur.whole_days());
    }
    if dur.whole_hours() > 0 && dur.whole_hours() < 24 {
        return format!("{} hours ago", dur.whole_hours());
    }
    if dur.whole_minutes() > 0 && dur.whole_minutes() < 60 {
        return format!("{} minutes ago", dur.whole_minutes());
    }
    if dur.whole_seconds() < 60 {
        return format!("{} seconds ago", dur.whole_seconds());
    }

    let utc_offset = jano::local_utc_offset().unwrap();
    if utc_offset < 0 {
        date -= std::time::Duration::from_secs((-utc_offset) as u64);
    } else {
        date -= std::time::Duration::from_secs(utc_offset as u64);
    }
    let date_time_off = time::OffsetDateTime::from(date);

    let year = date_time_off.year();
    let month = date_time_off.month() as u8;
    let day = date_time_off.day();
    let hour = date_time_off.hour();
    let min = date_time_off.minute();

    format!("{year}-{month}-{day} {hour}:{min}")
}

pub struct EditItemPage {
    pub id: Id,
    pub template: ItemTemplate,
    pub pic_options: bool,
}
impl EditItemPage {
    pub fn new_w_item(id: Id, item: Item) -> Self {
        Self {
            id,
            template: ItemTemplate::from_item(item),
            pic_options: false,
        }
    }
    pub fn new(id: Id) -> Self {
        Self {
            id,
            template: ItemTemplate::from_item(Item::default()),
            pic_options: false,
        }
    }
}
impl Page for EditItemPage {
    fn on_picture_taken(&mut self, pic: jano::Picture) {
        self.template.picture = Some(pic)
    }

    #[rustfmt::skip]
    fn title(&self) -> String { format!("Edit Item - {}", self.id.0) }

    #[rustfmt::skip]
    fn has_back_button(&self) -> bool { false }
    fn show2(&mut self, ui: &mut Ui, out: &mut UiOutput, app: &mut App, egui: &mut Egui) {
        #[rustfmt::skip]
        let Self { id, template: item, .. } = self;
        let id = *id;

        fn add_field(ui: &mut Ui, out: &mut UiOutput, label: &str, value: &mut String, w: f32) {
            ui.style_mut().spacing.text_edit_width = w;
            ui.horizontal(|ui| {
                let text_edit = text_edit(ui, out, value);
                text_edit.context_menu(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("paste").clicked {
                            text_edit.request_focus();
                            out.trigger_paste_cmd = true;
                        }
                    });
                });
                ui.label(label);
            });
        }
        fn add_field2(ui: &mut Ui, out: &mut UiOutput, label: &str, value: &mut String, w: f32) {
            ui.style_mut().spacing.text_edit_width = w;
            ui.horizontal(|ui| {
                let text_edit = {
                    let rs = ui.add(egui::TextEdit::singleline(value).hint_text(label));
                    if rs.has_focus() {
                        out.focused_text_field = Some(TextFieldInfo::new(label, [0, 0]));
                    }
                    rs
                };
                text_edit.context_menu(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("paste").clicked {
                            text_edit.request_focus();
                            out.trigger_paste_cmd = true;
                        }
                    });
                });
            });
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                let pic_size = egui::vec2(85.0, 85.0);
                let pic_rounding = 20.0;
                let pic_rs = match &item.picture {
                    Some(pic) => {
                        let handle = egui.obtain_tex_handle_for_pic(egui::Id::new(id.0), pic);
                        let sized_image = egui::load::SizedTexture::new(handle.id(), pic_size);
                        let image = egui::Image::from_texture(sized_image);
                        ui.add(image.sense(egui::Sense::click()).rounding(pic_rounding))
                    }
                    None => {
                        let (rect, response) =
                            ui.allocate_exact_size(pic_size, egui::Sense::click());
                        if ui.is_rect_visible(rect) {
                            ui.painter().rect_stroke(
                                rect,
                                pic_rounding,
                                egui::Stroke::new(2.0, egui::Color32::GRAY),
                            );
                        }
                        response
                    }
                };
                self.pic_options ^= pic_rs.clicked;
            });

            if self.pic_options {
                ui.horizontal(|ui| {
                    ui.label("|");
                    if ui.button("retake").clicked {
                        if let Err(err) = jano::take_picture() {
                            app.msg_popup(format!("{err:?}"));
                        }
                        self.pic_options = false;
                    }
                    if ui.button("remove").clicked {
                        item.picture = None;
                    }
                });
            }

            add_field2(ui, out, "name", &mut item.name, 500.0);
            add_field2(ui, out, "description", &mut item.desc, 500.0);
            add_field(ui, out, "count", &mut item.count, 80.0);
            add_field(ui, out, "cost", &mut item.est_cost, 80.0);
            add_field(ui, out, "width", &mut item.dimensions[0], 80.0);
            add_field(ui, out, "length", &mut item.dimensions[1], 80.0);
            add_field(ui, out, "height", &mut item.dimensions[2], 80.0);
            add_field(ui, out, "weight", &mut item.weight, 80.0);
            add_field(ui, out, "shipping weight", &mut item.shipping_weight, 80.0);
            add_field(ui, out, "color", &mut item.color, 80.0);
            add_field(ui, out, "brand", &mut item.brand, 80.0);
            add_field(ui, out, "model", &mut item.model_no, 80.0);
            add_field(ui, out, "serial", &mut item.serial_no, 80.0);
            add_field(ui, out, "condition", &mut item.condition, 80.0);
            add_field(ui, out, "location", &mut item.location, 80.0);

            for (idx, platform) in app.inv.platforms() {
                let mut rem = false;
                if let Some(listing) = &mut item.listings[idx] {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(platform);
                            if ui.button("-").clicked {
                                rem = true;
                            }
                        });
                        ui.label(format!("Listed: {}", display_date(listing.date)));
                        ui.horizontal(|ui| {
                            ui.label(format!("sold: {}", listing.sold));
                            let add = ui.button("+");
                            if add.secondary_clicked() {
                                listing.sold += 10;
                            } else if add.clicked() {
                                listing.sold += 1;
                            }

                            let sub = ui.button("-");
                            if sub.secondary_clicked() {
                                listing.sold = listing.sold.saturating_sub(10);
                            } else if sub.clicked() {
                                listing.sold = listing.sold.saturating_sub(1);
                            }
                        });
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label(platform);
                        if ui.button("+").clicked {
                            item.listings[idx] = Some(Listing::default());
                        }
                    });
                }
                if rem {
                    item.listings[idx] = None;
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.menu_button("Delete", |ui| {
                    ui.label("Are you sure? ");
                    if ui.button("Yes").clicked() {
                        ui.close_menu();
                        app.inv.remove_item(&id);
                        out.pop_page = true;
                    }
                    if ui.button("Cancel").clicked() {
                        ui.close_menu();
                    }
                });
                if ui.button("Save").clicked {
                    match item.as_item() {
                        Ok(item) => {
                            app.inv.insert_item(self.id, item);
                            out.pop_page = true;
                        }
                        Err(err) => {
                            app.msg_popup(format!("Invalid {err}"));
                        }
                    }
                }
                if ui.button("Cancel").clicked {
                    out.pop_page = true;
                }
            });
            ui.add_space(400.0);
        });
    }
}

#[derive(Default)]
pub struct SettingsPage {}
impl Page for SettingsPage {
    #[rustfmt::skip]
    fn title(&self) -> String { String::from("Settings") }

    fn show(&mut self, ui: &mut Ui, out: &mut UiOutput, app: &mut App) {
        ui.horizontal(|ui| {
            ui.label("Theme: ");
            ui.menu_button(app.settings.theme.to_string(), |ui| {
                for theme in UiTheme::ALL_OPTIONS {
                    if ui.button(theme.to_string()).clicked {
                        ui.close_menu();
                        app.settings.theme = theme;
                    }
                }
            });
        });
        ui.horizontal(|ui| {
            ui.label("Scale: ");
            ui.label(format!("{}", app.settings.scale));
            if ui.button("-").clicked && app.settings.scale > 2.0 {
                app.settings.scale -= 0.5;
            }
            if ui.button("+").clicked && app.settings.scale < 5.0 {
                app.settings.scale += 0.5;
            }
            jano::input::set_scale_factor(app.settings.scale);
        });
        ui.horizontal(|ui| {
            ui.label("Server address: ");
            text_edit(ui, out, &mut app.settings.server_address);
        });
        ui.horizontal(|ui| {
            ui.label("Server port: ");
            ui.add(egui::Slider::new(&mut app.settings.server_port, 0..=26000));
        });
        ui.horizontal(|ui| {
            ui.label("Username: ");
            text_edit(ui, out, &mut app.settings.name);
        });
    }
}

#[derive(Default)]
pub struct StatsPage {}
impl Page for StatsPage {
    #[rustfmt::skip]
    fn title(&self) -> String { String::from("Stats") }

    fn show(&mut self, ui: &mut Ui, _out: &mut UiOutput, app: &mut App) {
        let mut unsold_count = 0;
        let mut sold_count = 0;
        let mut unsold_cost = Usd(0);
        let mut sold_cost = Usd(0);

        for (_, item) in app.inv.items() {
            unsold_count += item.count - item.sold_count();
            sold_count += item.sold_count();
            unsold_cost.0 += item.est_cost.0 * item.count - item.est_cost.0 * item.sold_count();
            sold_cost.0 += item.est_cost.0 * item.sold_count();
        }

        ui.label(format!("Total listings: {}", app.inv.item_count()));

        ui.label(format!(
            "Total items: {} ({})",
            unsold_count + sold_count,
            Usd(unsold_cost.0 + sold_cost.0)
        ));
        ui.label(format!("Unsold items: {unsold_count} ({unsold_cost})"));
        ui.label(format!("Sold items: {sold_count} ({sold_cost})"));
    }
}
