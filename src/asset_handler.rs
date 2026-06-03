use std::{collections::HashMap, sync::Mutex};
use lazy_static::lazy_static;
use ustr::Ustr;
use uuid::Uuid;

use crate::poly::PolyInstance;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Content {
    Content(rbx_types::Content),
    ContentId(rbx_types::ContentId),
}

lazy_static! {
    pub static ref assets: Mutex<HashMap<Content, PolyInstance>> = Mutex::new(HashMap::new());
}

pub fn get_or_instantiate_asset(content: Content, class_name: Ustr) -> Uuid {
    let mut unwrapped_assets = assets.lock().unwrap();
    if unwrapped_assets.contains_key(&content) {
        unwrapped_assets.get(&content).unwrap().clone().id
    } else {
        let mut new_instance = PolyInstance::new();
        new_instance.class_name = class_name;
        new_instance.name = class_name.to_string();
        new_instance.id = Uuid::new_v4();

        unwrapped_assets.insert(content, new_instance.clone());
        new_instance.id
    }
}