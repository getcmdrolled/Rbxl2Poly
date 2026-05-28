pub mod poly_properties;

use std::collections::HashMap;
use ustr::{Ustr, ustr};
use uuid::{Uuid};
use serde::{self, Serialize, ser::SerializeStruct};

#[derive(Clone, Debug, serde::Serialize)]
#[serde(untagged)]
pub enum PolyProperty {
  Boolean(bool),
  Color(poly_properties::Color),
  Enum(u32),
  Int32(i32),
  NumberRange(poly_properties::NumberRange),
  String(String),
  Vector2(poly_properties::Vector2),
  Vector3(poly_properties::Vector3),
  Number(f32),
}

#[derive(Clone, Debug)]
pub struct PolyInstance {
  pub name: String,
  pub class_name: Ustr,
  pub id: Uuid,
  pub properties: HashMap<Ustr, PolyProperty>,
  pub children: Vec<PolyInstance>
}

impl Serialize for PolyInstance {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: serde::Serializer
  {
    if self.class_name == ustr("[none]") {
      serializer.serialize_none() // i fucking hate this, just let me not serialize anything???
    } else {
      let mut state = serializer.serialize_struct("PolyInstance", 5)?;
      state.serialize_field("Name", &self.name)?;
      state.serialize_field("ClassName", &self.class_name)?;
      state.serialize_field("ID", &self.id)?;
      state.serialize_field("Properties", &self.properties)?;
      state.serialize_field("Children", &self.children)?;
      state.end()
    }
  }
}

impl PolyInstance {
  pub fn new() -> PolyInstance {
    PolyInstance {
      name: String::new(),
      class_name: ustr(""),
      id: Uuid::new_v4(),
      properties: HashMap::new(),
      children: Vec::new()
    }
  }
}