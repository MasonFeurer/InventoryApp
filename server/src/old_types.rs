use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::time::SystemTime;

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
impl Default for Item {
    fn default() -> Self {
        Self {
            creation_date: SystemTime::now(),
            listed: Default::default(),
            sold: Default::default(),
            name: Default::default(),
            desc: Default::default(),
            count: 1,
            condition: Default::default(),
            est_cost: Default::default(),
            dimensions: Default::default(),
            weight: Default::default(),
            color: Default::default(),
            brand: Default::default(),
            category: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Inv {
    pub platform_names: Vec<String>,
    pub items: HashMap<Id, Item>,
}
impl Default for Inv {
    fn default() -> Self {
        Self {
            platform_names: vec!["Ebay".into(), "Facebook".into()],
            items: Default::default(),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(pub u32);
// impl Id {
//     pub fn new() -> Self {
//         Self(fastrand::u32(..))
//     }
// }
