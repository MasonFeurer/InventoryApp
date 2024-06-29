use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::time::SystemTime;

/// Always stored as ARGB, 1 byte per channel.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Picture {
    pub data: Vec<u8>,
    pub size: [u32; 2],
}

// x100 ($5.46 = Usd(546))
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Usd(pub u32);
impl std::fmt::Display for Usd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let upper = self.0 / 100;
        let lower = self.0 % 100;
        std::fmt::Display::fmt(&upper, f)?;
        f.write_char('.')?;
        std::fmt::Display::fmt(&lower, f)?;
        Ok(())
    }
}
impl std::str::FromStr for Usd {
    type Err = std::num::ParseFloatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let float: f32 = s.parse()?;
        Ok(Self((float * 100.0) as u32))
    }
}

/// Corresponds to one of the platforms in Inv::platform_names
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Platform(u8);
impl Platform {
    pub fn from_idx(idx: u8) -> Option<Self> {
        if idx >= 8 {
            None // `Listings` only supports up to 8 platforms.
        } else {
            Some(Self(idx))
        }
    }

    pub fn as_idx(&self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Listing {
    pub date: SystemTime,
    pub sold: u32,
}
impl Default for Listing {
    fn default() -> Self {
        Self {
            date: SystemTime::now(),
            sold: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Listings(pub [Option<Listing>; 8]);
impl Listings {
    pub fn count(&self) -> u8 {
        self.0.iter().map(|l| l.is_some() as u8).sum()
    }

    pub fn total_sold(&self) -> u32 {
        self.0.iter().flatten().map(|l| l.sold).sum()
    }

    pub fn contains_platform(&self, platform: Platform) -> bool {
        self.0[platform.as_idx() as usize].is_some()
    }

    pub fn add_listing(&mut self, platform: Platform) {
        self.0[platform.as_idx() as usize] = Some(Listing::default());
    }
}
impl std::ops::Index<Platform> for Listings {
    type Output = Option<Listing>;
    fn index(&self, idx: Platform) -> &Self::Output {
        &self.0[idx.0 as usize]
    }
}
impl std::ops::IndexMut<Platform> for Listings {
    fn index_mut(&mut self, idx: Platform) -> &mut Self::Output {
        &mut self.0[idx.0 as usize]
    }
}
impl IntoIterator for Listings {
    type Item = (Platform, Listing);
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = (self.0)
            .into_iter()
            .enumerate()
            .filter_map(|(p, l)| l.map(|l| (Platform(p as u8), l)));
        Box::new(iter)
    }
}
impl<'a> IntoIterator for &'a Listings {
    type Item = (Platform, &'a Listing);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;
    fn into_iter(self) -> Self::IntoIter {
        let iter = (self.0)
            .iter()
            .enumerate()
            .filter_map(|(p, l)| l.as_ref().map(|l| (Platform(p as u8), l)));
        Box::new(iter)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    // Inventory properties
    pub creation_date: SystemTime,
    pub location: String,
    pub listings: Listings,
    pub picture: Option<Picture>,

    // Item details
    pub name: String,
    pub desc: String,
    pub count: u32,

    pub est_cost: Usd,
    pub condition: String,
    pub color: String,
    pub dimensions: [f32; 3],
    pub weight: f32,
    pub shipping_weight: f32,

    pub model_no: u64,
    pub serial_no: u64,
    pub brand: String,
}
impl std::clone::Clone for Item {
    fn clone(&self) -> Self {
        Self {
            creation_date: SystemTime::now(),
            location: self.location.clone(),
            listings: self.listings.clone(),
            picture: self.picture.clone(),
            name: self.name.clone(),
            desc: self.desc.clone(),
            count: self.count,
            est_cost: self.est_cost,
            condition: self.condition.clone(),
            color: self.color.clone(),
            dimensions: self.dimensions,
            weight: self.weight,
            shipping_weight: self.shipping_weight,
            model_no: self.model_no,
            serial_no: self.serial_no,
            brand: self.brand.clone(),
        }
    }
}
impl Item {
    pub fn sold_count(&self) -> u32 {
        self.listings
            .0
            .iter()
            .flatten()
            .map(|listing| listing.sold)
            .sum()
    }
}
impl Default for Item {
    fn default() -> Self {
        Self {
            // Inventory properties
            creation_date: SystemTime::now(),
            location: String::new(),
            listings: Listings::default(),
            picture: None,

            // Item details
            name: String::new(),
            desc: String::new(),
            count: 1,

            est_cost: Usd(0),
            condition: String::new(),
            color: String::new(),
            dimensions: [0.0; 3],
            weight: 0.0,
            shipping_weight: 0.0,

            model_no: 0,
            serial_no: 0,
            brand: String::new(),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Inv {
    pub platform_names: Vec<String>,
    pub items: HashMap<Id, Item>,
}
impl Inv {
    pub fn platforms(&self) -> impl Iterator<Item = (Platform, &str)> {
        self.platform_names
            .iter()
            .enumerate()
            .map(|(p, n)| (Platform(p as u8), n.as_str()))
    }

    pub fn get_platform_name(&self, platform: Platform) -> &str {
        self.platform_names[platform.as_idx() as usize].as_str()
    }

    pub fn all_locations(&self) -> impl Iterator<Item = &str> {
        let mut items = self
            .items
            .values()
            .map(|item| item.location.as_str())
            .filter(|loc| !loc.is_empty())
            .collect::<HashSet<_>>() // removes duplicates
            .into_iter()
            .collect::<Vec<_>>();

        items.sort();
        items.into_iter()
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Id(pub u32);
impl Id {
    pub fn new() -> Self {
        Self(fastrand::u32(..))
    }
}
