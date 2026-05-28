use crate::poly;
use std::{collections::HashMap, sync::Arc};
use paste::paste;
use ustr::ustr;

// UTIL

type Iconverter = dyn Fn(poly::PolyInstance, &rbx_dom_weak::Instance) -> poly::PolyInstance;

pub fn direct_rbxl_variant_to_poly_property(property: rbx_dom_weak::types::Variant) -> poly::PolyProperty {
  let fail_property = poly::PolyProperty::String(String::from("FAHH"));
  match property.ty() as u8 {
    2 => {
      let rbx_dom_weak::types::Variant::Bool(val) = property else { return fail_property; };
      return poly::PolyProperty::Boolean(val);
    }
    11 => {
      let rbx_dom_weak::types::Variant::Float32(val) = property else { return fail_property; };
      return poly::PolyProperty::Number(val);
    }
    13 => {
      let rbx_dom_weak::types::Variant::Int32(val) = property else { return fail_property; };
      return poly::PolyProperty::Int32(val);
    }
    15 => {
      let rbx_dom_weak::types::Variant::NumberRange(val) = property else { return fail_property; };
      return poly::PolyProperty::NumberRange(poly::poly_properties::NumberRange { min: val.min, max: val.max });
    }
    24 => {
      let rbx_dom_weak::types::Variant::String(val) = property else { return fail_property; };
      return poly::PolyProperty::String(val);
    }
    27 => {
      let rbx_dom_weak::types::Variant::Vector2(val) = property else { return fail_property; };
      return poly::PolyProperty::Vector2(poly::poly_properties::Vector2 { x: val.x, y: val.y });
    }
    29 => {
      let rbx_dom_weak::types::Variant::Vector3(val) = property else { return fail_property; };
      return poly::PolyProperty::Vector3(poly::poly_properties::Vector3 { x: val.x, y: val.y, z: val.z });
    }
    _ => {
      //println!("[DEBUG] Failed to parse property of type {}, not converting.", property.ty() as u8);
    }
  }
  return fail_property;
}

macro_rules! gen_direct_class_converter {
    ($f_name:tt,$class_name:tt) => {
        paste! {
            fn [<convert_class_ $f_name>](mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
                poly_instance.class_name = ustr($class_name);
                [<$f_name _properties>](poly_instance, rbxl_instance)
            }
        }
    };
}

macro_rules! gen_direct_property_converter {
    ($f_name:tt,$rbxl_property_name:tt,$property_name:tt) => {
        paste! {
            fn [<convert_property_ $f_name>](mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
                poly_instance.properties.insert(ustr($property_name), direct_rbxl_variant_to_poly_property(rbxl_instance.properties.get(&ustr($rbxl_property_name)).unwrap().clone()));
                poly_instance
            }
        }
    };
}

// PROPERTY CONVERTERS

gen_direct_property_converter!(size, "Size", "Size");
gen_direct_property_converter!(anchored, "Anchored", "Anchored");
gen_direct_property_converter!(velocity, "Velocity", "Velocity");
gen_direct_property_converter!(can_collide, "CanCollide", "CanCollide");
gen_direct_property_converter!(rot_velocity, "RotVelocity", "AngularVelocity");
gen_direct_property_converter!(locked, "Locked", "Locked");
gen_direct_property_converter!(cast_shadow, "CastShadow", "CastShadows");
gen_direct_property_converter!(field_of_view, "FieldOfView", "FOV");

fn convert_property_cframe(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::CFrame(cframe) = rbxl_instance.properties.get(&ustr("CFrame")).unwrap() else { return poly_instance; };
    poly_instance.properties.insert(ustr("Position"), poly::PolyProperty::Vector3(poly::poly_properties::Vector3 { x: cframe.position.x, y: cframe.position.y, z: cframe.position.z }));
    poly_instance.properties.insert(ustr("Rotation"), poly::PolyProperty::Vector3(poly::poly_properties::Vector3 { x: cframe.orientation.x.x, y: cframe.orientation.x.y, z: cframe.orientation.x.z }));
    poly_instance
}

fn convert_property_part_color(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::Color3uint8(color) = rbxl_instance.properties.get(&ustr("Color")).unwrap() else { return poly_instance; };

    let mut target_color = poly::poly_properties::Color { r: color.r as f32 / 256.0, g: color.g as f32 / 256.0, b: color.b as f32 / 256.0, a: 1.0 };

    if poly_instance.properties.contains_key(&ustr("Color")) {
        let poly::PolyProperty::Color(existing_color) = &poly_instance.properties[&ustr("Color")] else { return poly_instance; };
        target_color.a = existing_color.a;
    }

    poly_instance.properties.insert(ustr("Color"), poly::PolyProperty::Color(target_color));
    poly_instance
}

fn convert_property_part_transparency(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::Float32(transparency) = rbxl_instance.properties.get(&ustr("Transparency")).unwrap() else { return poly_instance; };

    let mut target_color = poly::poly_properties::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 - transparency };

    if poly_instance.properties.contains_key(&ustr("Color")) {
        let poly::PolyProperty::Color(existing_color) = &poly_instance.properties[&ustr("Color")] else { return poly_instance; };
        target_color.r = existing_color.r;
        target_color.g = existing_color.g;
        target_color.b = existing_color.b;
    }

    poly_instance.properties.insert(ustr("Color"), poly::PolyProperty::Color(target_color));
    poly_instance
}

fn convert_property_gravity(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::Float32(existing_gravity) = rbxl_instance.properties.get(&ustr("Gravity")).unwrap() else { return poly_instance; };
    let new_property: poly::PolyProperty = poly::PolyProperty::Vector3(poly::poly_properties::Vector3 {x: 0.0, y: *existing_gravity / -2.308235294, z: 0.0});
    poly_instance.properties.insert(ustr("Gravity"), new_property);
    poly_instance
}

// CLASS-PROPERTY CONVERTERS

fn part_properties(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance = convert_property_cframe(poly_instance, rbxl_instance);
    poly_instance = convert_property_size(poly_instance, rbxl_instance);
    poly_instance = convert_property_velocity(poly_instance, rbxl_instance);
    poly_instance = convert_property_rot_velocity(poly_instance, rbxl_instance);
    poly_instance = convert_property_anchored(poly_instance, rbxl_instance);
    poly_instance = convert_property_can_collide(poly_instance, rbxl_instance);
    poly_instance = convert_property_cast_shadow(poly_instance, rbxl_instance);
    poly_instance = convert_property_locked(poly_instance, rbxl_instance);
    poly_instance = convert_property_part_color(poly_instance, rbxl_instance);
    poly_instance = convert_property_part_transparency(poly_instance, rbxl_instance);
    poly_instance
}

fn workspace_properties(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance = convert_property_gravity(poly_instance, rbxl_instance);
    poly_instance
}

fn camera_properties(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance = convert_property_field_of_view(poly_instance, rbxl_instance);
    poly_instance
}

// CLASS CONVERTERS

gen_direct_class_converter!{part, "Part"}
gen_direct_class_converter!{workspace, "Environment"}
gen_direct_class_converter!{camera, "Camera"}

fn convert_class_none(mut poly_instance: poly::PolyInstance, _: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance.class_name = ustr("[none]");
    poly_instance
}

fn convert_class_data_model(mut poly_instance: poly::PolyInstance, _: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance.class_name = ustr("World");
    poly_instance
}

fn convert_class_spawn_location(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance.class_name = ustr("Part");
    poly_instance.properties.insert(ustr("IsSpawn"), poly::PolyProperty::Boolean(true));
    poly_instance = part_properties(poly_instance, rbxl_instance); // inherits part :D
    poly_instance
}

// YIPPEE

pub fn get_converter_for_class(class_name: ustr::Ustr) -> Arc<Iconverter> {
    let class_converter: HashMap<ustr::Ustr, Arc<Iconverter>> = HashMap::from([
        (ustr("Part"), Arc::new(convert_class_part) as Arc<Iconverter>),
        (ustr("Workspace"), Arc::new(convert_class_workspace) as Arc<Iconverter>),
        (ustr("DataModel"), Arc::new(convert_class_data_model) as Arc<Iconverter>),
        (ustr("Camera"), Arc::new(convert_class_camera) as Arc<Iconverter>),
        (ustr("SpawnLocation"), Arc::new(convert_class_spawn_location) as Arc<Iconverter>)
    ]);
    
    let invalid_class: Arc<Iconverter> = Arc::new(convert_class_none) as Arc<Iconverter>;
    return class_converter.get(&class_name).unwrap_or(&invalid_class).clone();
}