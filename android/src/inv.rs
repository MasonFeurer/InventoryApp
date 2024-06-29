use jano::serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub use inv_common::inv::*;

pub fn to_jano_pic(pic: Picture) -> jano::Picture {
    let Picture { data, size } = pic;
    let size = jano::glam::uvec2(size[0], size[1]);
    jano::Picture { data, size }
}
pub fn to_inv_pic(pic: jano::Picture) -> Picture {
    let jano::Picture { data, size } = pic;
    let size = [size.x, size.y];
    Picture { data, size }
}

pub enum InvChange {
    AddedItem(Id),
    ModifiedItem(Id),
    DeletedItem(Id),
}

#[derive(Default, Serialize, Deserialize)]
pub struct LocalInv {
    modified_items: HashSet<Id>,
    deleted_items: HashSet<Id>,
    added_items: HashSet<Id>,

    inv: Inv,
}
impl std::ops::Deref for LocalInv {
    type Target = Inv;
    fn deref(&self) -> &Inv {
        &self.inv
    }
}
impl std::ops::DerefMut for LocalInv {
    fn deref_mut(&mut self) -> &mut Inv {
        &mut self.inv
    }
}
impl LocalInv {
    pub fn consume_changes(&mut self) -> impl Iterator<Item = InvChange> {
        let mut modified = HashSet::new();
        let mut deleted = HashSet::new();
        let mut added = HashSet::new();
        std::mem::swap(&mut modified, &mut self.modified_items);
        std::mem::swap(&mut deleted, &mut self.deleted_items);
        std::mem::swap(&mut added, &mut self.added_items);

        modified
            .into_iter()
            .map(InvChange::ModifiedItem)
            .chain(deleted.into_iter().map(InvChange::DeletedItem))
            .chain(added.into_iter().map(InvChange::AddedItem))
    }

    pub fn r#override(&mut self, inv: Inv) {
        self.inv = inv;
        self.modified_items.clear();
        self.deleted_items.clear();
        self.added_items.clear();
    }

    pub fn get_item(&self, id: &Id) -> Option<&Item> {
        self.inv.items.get(id)
    }

    pub fn items(&self) -> impl Iterator<Item = (&Id, &Item)> {
        self.inv.items.iter()
    }
    pub fn item_count(&self) -> usize {
        self.inv.items.len()
    }

    pub fn insert_item(&mut self, id: Id, item: Item) {
        if !self.inv.items.contains_key(&id) {
            self.added_items.insert(id);
        } else if !self.added_items.contains(&id) {
            self.modified_items.insert(id);
        }
        self.inv.items.insert(id, item);
    }

    pub fn remove_item(&mut self, id: &Id) {
        self.inv.items.remove(id);
        self.deleted_items.insert(*id);
    }
}
