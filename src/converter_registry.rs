use crate::{asset_handler, poly};
use std::{collections::HashMap, sync::Arc};
use paste::paste;
use ustr::ustr;
use glam;

// UTIL

pub fn matrix_to_euler_angles(orientation: rbx_types::Matrix3) -> (f32, f32, f32) {
    let matrix = glam::Mat3 { x_axis: glam::Vec3 {
        x: orientation.x.x,
        y: orientation.y.x,
        z: orientation.z.x
    }, y_axis: glam::Vec3 {
        x: orientation.x.y,
        y: orientation.y.y,
        z: orientation.z.y
    }, z_axis: glam::Vec3 {
        x: orientation.x.z,
        y: orientation.y.z,
        z: orientation.z.z
    }};
    let euler = matrix.to_euler(glam::EulerRot::YXZ);

    return (-euler.1.to_degrees(), euler.0.to_degrees(), euler.2.to_degrees())
}

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

macro_rules! gen_direct_class_property_converter {
    ($f_name:tt,$($item:expr),*) => {
        paste! {
            fn [<$f_name _properties>](mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
            $(
                poly_instance = [<convert_property_ $item>](poly_instance, rbxl_instance);
            )*
                poly_instance
            }
        }
    }
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

macro_rules! gen_direct_propertyless_class_converter {
    ($f_name:tt,$class_name:tt) => {
        paste! {
            fn [<convert_class_ $f_name>](mut poly_instance: poly::PolyInstance, _: &rbx_dom_weak::Instance) -> poly::PolyInstance {
                poly_instance.class_name = ustr($class_name);
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
gen_direct_property_converter!(gui_enabled, "Enabled", "Visible");
gen_direct_property_converter!(gui_display_order, "DisplayOrder", "ZIndex");

fn convert_property_part_material(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::Enum(material) = rbxl_instance.properties.get(&ustr("Material")).unwrap() else { return poly_instance; };
    let new_material: u32 = match material.to_u32() {
        848 => 1,
        816 => 2,
        1344 => 3,
        3 => 4,
        1280 => 5,
        1536 => 6,
        784 => 7,
        1088 => 8,
        // 0 => 9, <-- no metal grid equivalent
        1056 => 10,
        288 => 11,
        528 => 12,
        256 => 13,
        // 0 => 14, <-- no plywood equivalent
        1040 => 15,
        1296 => 16,
        912 => 17,
        272 => 0,
        1328 => 18,
        820 => 19, // i don't have a close equivalent to stone, just put limestone ig
        512 => 20,
        _ => 17
    };

    poly_instance.properties.insert(ustr("Material"), poly::PolyProperty::Enum(new_material));
    poly_instance
}

fn convert_property_model_scale(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::Float32(scale) = rbxl_instance.properties.get(&ustr("Scale")).unwrap() else { return poly_instance; };
    poly_instance.properties.insert(ustr("Size"), poly::PolyProperty::Vector3(poly::poly_properties::Vector3 { x: *scale, y: *scale, z: *scale }));
    poly_instance
}

fn convert_property_part_shape(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::Enum(part_shape) = rbxl_instance.properties.get(&ustr("Shape")).unwrap() else { return poly_instance; };
    let new_shape: u32 = match part_shape.to_u32() {
        0 => 1,
        1 => 0,
        2 => 2,
        3 => 4,
        4 => 5,
        _ => 0
    };

    poly_instance.properties.insert(ustr("Shape"), poly::PolyProperty::Enum(new_shape));
    poly_instance
}

fn convert_property_cframe(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::CFrame(cframe) = rbxl_instance.properties.get(&ustr("CFrame")).unwrap() else { return poly_instance; };
    poly_instance.properties.insert(ustr("Position"), poly::PolyProperty::Vector3(poly::poly_properties::Vector3 { x: -cframe.position.x, y: cframe.position.y, z: -cframe.position.z }));
    let rot = matrix_to_euler_angles(cframe.orientation);

    poly_instance.properties.insert(ustr("Rotation"), poly::PolyProperty::Vector3(poly::poly_properties::Vector3 { x: rot.0, y: rot.1, z: rot.2 }));
    poly_instance
}

fn convert_property_world_pivot(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::CFrame(cframe) = rbxl_instance.properties.get(&ustr("WorldPivotData")).unwrap() else { return poly_instance; };
    poly_instance.properties.insert(ustr("Position"), poly::PolyProperty::Vector3(poly::poly_properties::Vector3 { x: -cframe.position.x, y: cframe.position.y, z: -cframe.position.z }));
    let rot = matrix_to_euler_angles(cframe.orientation);
    poly_instance.properties.insert(ustr("Rotation"), poly::PolyProperty::Vector3(poly::poly_properties::Vector3 { x: rot.0, y: rot.1, z: rot.2 }));
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

fn convert_property_team_color(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    let rbx_dom_weak::types::Variant::BrickColor(brick_color) = rbxl_instance.properties.get(&ustr("TeamColor")).unwrap() else { return poly_instance; };
    let color = brick_color.to_color3uint8();
    let target_color = poly::poly_properties::Color { r: color.r as f32 / 256.0, g: color.g as f32 / 256.0, b: color.b as f32 / 256.0, a: 1.0 };

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

gen_direct_class_property_converter!(part, cframe, size, velocity, rot_velocity, anchored, can_collide, cast_shadow, locked, part_color, part_transparency, part_material, part_shape);
gen_direct_class_property_converter!(base_part, cframe, size, velocity, rot_velocity, anchored, can_collide, cast_shadow, locked, part_color, part_transparency, part_material);
gen_direct_class_property_converter!(model, world_pivot, model_scale);
gen_direct_class_property_converter!(workspace, gravity);
gen_direct_class_property_converter!(camera, field_of_view);
gen_direct_class_property_converter!(team, team_color);

gen_direct_class_property_converter!(screen_gui, gui_enabled, gui_display_order);

// CLASS CONVERTERS

gen_direct_class_converter!(part, "Part");
gen_direct_class_converter!(model, "Model");
gen_direct_class_converter!(workspace, "Environment");
gen_direct_class_converter!(camera, "Camera");
gen_direct_class_converter!(team, "Team");

gen_direct_class_converter!(screen_gui, "GUI");

gen_direct_propertyless_class_converter!(none, "[none]");
gen_direct_propertyless_class_converter!(data_model, "World");
gen_direct_propertyless_class_converter!(script_service, "ScriptService");
gen_direct_propertyless_class_converter!(replicated, "Hidden");
gen_direct_propertyless_class_converter!(server_storage, "ServerHidden");
gen_direct_propertyless_class_converter!(players, "Players");
gen_direct_propertyless_class_converter!(teams, "Teams");
gen_direct_propertyless_class_converter!(starter_gui, "PlayerGUI");

fn convert_class_spawn_location(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance.class_name = ustr("Part");
    poly_instance.properties.insert(ustr("IsSpawn"), poly::PolyProperty::Boolean(true));
    poly_instance = part_properties(poly_instance, rbxl_instance); // inherits part :D
    poly_instance
}

fn convert_class_truss_part(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance.class_name = ustr("Part");
    poly_instance = base_part_properties(poly_instance, rbxl_instance);
    poly_instance.properties.insert(ustr("Shape"), poly::PolyProperty::Enum(8));
    poly_instance
}

fn convert_class_mesh_part(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance.class_name = ustr("Mesh");
    poly_instance = base_part_properties(poly_instance, rbxl_instance);
    poly_instance.properties.insert(ustr("UsePartColor"), poly::PolyProperty::Boolean(true));

    let rbx_dom_weak::types::Variant::Content(content) = rbxl_instance.properties.get(&ustr("MeshContent")).unwrap() else { return poly_instance; };
    poly_instance.properties.insert(ustr("Asset"), poly::PolyProperty::Ref(asset_handler::get_or_instantiate_asset(content.clone(), ustr("PTMeshAsset"))));

    // halve size it's formatted differently :sob:
    let poly::PolyProperty::Vector3(mut size) = poly_instance.properties.get(&ustr("Size")).unwrap().clone() else { return poly_instance; };
    size = poly::poly_properties::Vector3 { x: size.x / 2.0, y: size.y / 2.0, z: size.z / 2.0 };
    poly_instance.properties.insert(ustr("Size"), poly::PolyProperty::Vector3(size));
    
    poly_instance
}

fn convert_class_wedge_part(mut poly_instance: poly::PolyInstance, rbxl_instance: &rbx_dom_weak::Instance) -> poly::PolyInstance {
    poly_instance.class_name = ustr("Part");
    poly_instance = base_part_properties(poly_instance, rbxl_instance);
    poly_instance.properties.insert(ustr("Shape"), poly::PolyProperty::Enum(4));

    // rotate 90 degrees on y axis because it's formatted differently :sob:
    let poly::PolyProperty::Vector3(mut rotation) = poly_instance.properties.get(&ustr("Rotation")).unwrap().clone() else { return poly_instance; };
    rotation.y -= 90.0;
    poly_instance.properties.insert(ustr("Rotation"), poly::PolyProperty::Vector3(rotation));

    poly_instance
}

// YIPPEE

pub fn get_converter_for_class(class_name: ustr::Ustr) -> Arc<Iconverter> {
    let class_converter: HashMap<ustr::Ustr, Arc<Iconverter>> = HashMap::from([
        (ustr("Part"), Arc::new(convert_class_part) as Arc<Iconverter>),
        (ustr("TrussPart"), Arc::new(convert_class_truss_part) as Arc<Iconverter>),
        (ustr("WedgePart"), Arc::new(convert_class_wedge_part) as Arc<Iconverter>),
        (ustr("MeshPart"), Arc::new(convert_class_mesh_part) as Arc<Iconverter>),
        (ustr("Model"), Arc::new(convert_class_model) as Arc<Iconverter>),
        (ustr("Camera"), Arc::new(convert_class_camera) as Arc<Iconverter>),
        (ustr("SpawnLocation"), Arc::new(convert_class_spawn_location) as Arc<Iconverter>),
        (ustr("Team"), Arc::new(convert_class_team) as Arc<Iconverter>),

        (ustr("DataModel"), Arc::new(convert_class_data_model) as Arc<Iconverter>),
        (ustr("Workspace"), Arc::new(convert_class_workspace) as Arc<Iconverter>),
        (ustr("ServerScriptService"), Arc::new(convert_class_script_service) as Arc<Iconverter>),
        (ustr("ReplicatedStorage"), Arc::new(convert_class_replicated) as Arc<Iconverter>),
        (ustr("ReplicatedFirst"), Arc::new(convert_class_replicated) as Arc<Iconverter>),
        (ustr("ServerStorage"), Arc::new(convert_class_server_storage) as Arc<Iconverter>),
        (ustr("Players"), Arc::new(convert_class_players) as Arc<Iconverter>),
        (ustr("Teams"), Arc::new(convert_class_teams) as Arc<Iconverter>),

        (ustr("StarterGui"), Arc::new(convert_class_starter_gui) as Arc<Iconverter>),
        (ustr("ScreenGui"), Arc::new(convert_class_screen_gui) as Arc<Iconverter>),
    ]);
    
    let invalid_class: Arc<Iconverter> = Arc::new(convert_class_none) as Arc<Iconverter>;
    return class_converter.get(&class_name).unwrap_or(&invalid_class).clone();
}