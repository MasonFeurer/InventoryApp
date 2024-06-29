use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::time::SystemTime;

use crate::inv as new;

// x100 ($5.46 = Usd(546))
#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub struct Usd(pub u32);
impl std::fmt::Display for Usd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let upper = self.0 / 100;
        let lower = self.0 % 100;
        f.write_char('$')?;
        std::fmt::Display::fmt(&upper, f)?;
        f.write_char('.')?;
        std::fmt::Display::fmt(&lower, f)?;
        Ok(())
    }
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Condition {
    #[default]
    NewInBox,
    FairInBox,
    WornInBox,

    NewNoBox,
    FairNoBox,
    WornNoBox,

    NewDamagedBox,
    FairDamagedBox,
    WornDamagedBox,
}

// Each bit enabled for the corresponding platform in Inv::platform_names
#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub struct Platforms(pub u8);

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub creation_date: SystemTime,
    pub listed: Platforms,
    pub sold: u32,
    pub name: String,
    pub desc: String,
    pub count: u32,
    pub condition: Condition,
    pub est_cost: Usd,
    pub dimensions: [f32; 3],
    pub weight: f32,
    pub color: String,
    pub brand: String,
    pub category: String,
}
impl Item {
    pub fn update(self) -> new::Item {
        new::Item {
            creation_date: self.creation_date,
            location: String::new(),
            listings: new::Listings::default(),
            picture: None,
            name: self.name,
            desc: self.desc,
            count: self.count,
            est_cost: new::Usd(self.est_cost.0),
            condition: format!("{:?}", self.condition),
            color: self.color,
            dimensions: self.dimensions,
            weight: self.weight,
            shipping_weight: 0.0,
            model_no: 0,
            serial_no: 0,
            brand: self.brand,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Inv {
    pub platform_names: Vec<String>,
    pub items: HashMap<Id, Item>,
}
impl Inv {
    pub fn update(self) -> new::Inv {
        let items = self
            .items
            .into_iter()
            .map(|(id, item)| (new::Id(id.0), item.update()))
            .collect();
        new::Inv {
            platform_names: self.platform_names,
            items,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(pub u32);
