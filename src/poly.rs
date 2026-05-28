pub mod poly_properties;

use std::collections::HashMap;
use ustr::{Ustr, ustr};
use uuid::{Uuid};
use lazy_static::lazy_static;
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

/*{
  "Name": "Instance",
  "ClassName": "Instance",
  "ID": "blabla uuid",
  "Properties": {
    "Tags": []
  },
  "Children": [],
  "LinkedModel": null,
  "IsLinkedChild": false
}*/
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

lazy_static! {
  pub static ref DIRECT_CLASS_CONVERT: HashMap<Ustr, Ustr> = {
    let mut m = HashMap::new();
    m.insert(ustr("Part"), ustr("Part"));
    m.insert(ustr("Workspace"), ustr("Environment"));
    m.insert(ustr("DataModel"), ustr("World"));
    m.insert(ustr("Camera"), ustr("Camera"));
    m
  };

  pub static ref DIRECT_PROPERTY_CONVERT: HashMap<Ustr, Ustr> = {
    let mut m = HashMap::new();
    m.insert(ustr("Size"), ustr("Size"));
    m.insert(ustr("Anchored"), ustr("Anchored"));
    m.insert(ustr("Velocity"), ustr("Velocity"));
    m.insert(ustr("CanCollide"), ustr("CanCollide"));
    m.insert(ustr("RotVelocity"), ustr("AngularVelocity"));
    m.insert(ustr("Locked"), ustr("Locked"));
    m.insert(ustr("CastShadow"), ustr("CastShadows"));
    m.insert(ustr("FieldOfView"), ustr("FOV"));
    m
  };

  pub static ref IGNORE_CLASS_LIST: Vec<Ustr> = {
    let mut vec = Vec::new();
    vec.push(ustr("Terrain"));
    vec.push(ustr("Decal"));
    vec
  };
}

pub fn convert_class(class_name: Ustr, mut poly_instance: PolyInstance) -> PolyInstance {
  poly_instance.class_name = *DIRECT_CLASS_CONVERT.get(&class_name).unwrap_or(&ustr("[none]"));

  if poly_instance.class_name == ustr("[none]") {
    match class_name.to_string().as_str() {
      "SpawnLocation" => {
        poly_instance.class_name = ustr("Part");
        poly_instance.properties.insert(ustr("IsSpawn"), PolyProperty::Boolean(true));
      }
      _ => {
        //println!("[DEBUG] Failed to parse class \"{}\", defaulting to Instance if in ignorelist.", class_name);
        if IGNORE_CLASS_LIST.contains(&class_name) {
          poly_instance.class_name = ustr("Instance");
        } else {
          poly_instance.class_name = ustr("[none]");
        }
      }
    }
  }

  return poly_instance;
}

pub fn convert_property(property: (Ustr, rbx_dom_weak::types::Variant), mut poly_instance: PolyInstance) -> PolyInstance {
  if DIRECT_PROPERTY_CONVERT.contains_key(&property.0) {
    poly_instance.properties.insert(*DIRECT_PROPERTY_CONVERT.get(&property.0).unwrap(), direct_rbxl_variant_to_poly_property(property.1));
  } else {
    match property.0.to_string().as_str() {
      "CFrame" => {
        let rbx_dom_weak::types::Variant::CFrame(cframe) = property.1 else { return poly_instance; };
        poly_instance.properties.insert(ustr("Position"), PolyProperty::Vector3(poly_properties::Vector3 { x: cframe.position.x, y: cframe.position.y, z: cframe.position.z }));
        poly_instance.properties.insert(ustr("Rotation"), PolyProperty::Vector3(poly_properties::Vector3 { x: cframe.orientation.x.x, y: cframe.orientation.x.y, z: cframe.orientation.x.z }));
      }
      "Color" => {
        let mut target_color: poly_properties::Color;
        match property.1 {
          rbx_dom_weak::types::Variant::Color3(color) => {
            target_color = poly_properties::Color { r: color.r, g: color.g, b: color.b, a: 1.0 };
          },
          rbx_dom_weak::types::Variant::Color3uint8(color) => {
            let modified_r = color.r as f32 / 256.0;
            let modified_g = color.g as f32 / 256.0;
            let modified_b = color.b as f32 / 256.0;
            target_color = poly_properties::Color { r: modified_r, g: modified_g, b: modified_b, a: 1.0 };
          },
          _ => { return poly_instance; }
        };

        if poly_instance.properties.contains_key(&ustr("Color")) {
          let PolyProperty::Color(existing_color) = &poly_instance.properties[&ustr("Color")] else { return poly_instance; };
          target_color.a = existing_color.a;
        }

        poly_instance.properties.insert(ustr("Color"), PolyProperty::Color(target_color));
      }
      "Transparency" => {
        let rbx_dom_weak::types::Variant::Float32(transparency) = property.1 else { return poly_instance; };
        let mut target_color = poly_properties::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 - transparency };

        if poly_instance.properties.contains_key(&ustr("Color")) {
          let PolyProperty::Color(existing_color) = &poly_instance.properties[&ustr("Color")] else { return poly_instance; };
          target_color.r = existing_color.r;
          target_color.g = existing_color.g;
          target_color.b = existing_color.b;
        }

        poly_instance.properties.insert(ustr("Color"), PolyProperty::Color(target_color));
      }
      _ => {
        //println!("[DEBUG] Failed to parse property \"{}\" of type {}, not adding.", property.0, property.1.ty() as u8);
      }
    }
  }

  return poly_instance;
}

pub fn direct_rbxl_variant_to_poly_property(property: rbx_dom_weak::types::Variant) -> PolyProperty {
  let fail_property = PolyProperty::String(String::from("FAHH"));
  match property.ty() as u8 {
    2 => {
      let rbx_dom_weak::types::Variant::Bool(val) = property else { return fail_property; };
      return PolyProperty::Boolean(val);
    }
    11 => {
      let rbx_dom_weak::types::Variant::Float32(val) = property else { return fail_property; };
      return PolyProperty::Number(val);
    }
    13 => {
      let rbx_dom_weak::types::Variant::Int32(val) = property else { return fail_property; };
      return PolyProperty::Int32(val);
    }
    15 => {
      let rbx_dom_weak::types::Variant::NumberRange(val) = property else { return fail_property; };
      return PolyProperty::NumberRange(poly_properties::NumberRange { min: val.min, max: val.max });
    }
    24 => {
      let rbx_dom_weak::types::Variant::String(val) = property else { return fail_property; };
      return PolyProperty::String(val);
    }
    27 => {
      let rbx_dom_weak::types::Variant::Vector2(val) = property else { return fail_property; };
      return PolyProperty::Vector2(poly_properties::Vector2 { x: val.x, y: val.y });
    }
    29 => {
      let rbx_dom_weak::types::Variant::Vector3(val) = property else { return fail_property; };
      return PolyProperty::Vector3(poly_properties::Vector3 { x: val.x, y: val.y, z: val.z });
    }
    _ => {
      //println!("[DEBUG] Failed to parse property of type {}, not converting.", property.ty() as u8);
    }
  }
  return fail_property;
}