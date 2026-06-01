pub mod poly;
pub mod converter_registry;
pub mod asset_handler;

use rbx_binary;
use rbx_dom_weak;
use std::fs::{File, write};
use std::io::BufReader;
use std::env::args;
use serde_json::json;
use colored::Colorize;
use anyhow::{Context, Result};
use ruzstd;
use ustr::{Ustr, ustr};

fn main() -> Result<()> {
    let argument = args().nth(1).with_context(|| "No file provided!")?;
    let mut input = BufReader::new(File::open(argument).with_context(|| "Unable to read file")?);
    let dom;
    dom = rbx_binary::from_reader(&mut input).unwrap_or_else(|_err| {
        rbx_xml::from_reader(&mut input, rbx_xml::DecodeOptions::default()).with_context(|| "Unable to parse file, not rbxl or rbxlx").unwrap()
    });

    let poly_instance = rbxl_to_poly_instance(dom.root(), &dom);

    let mut assets: Vec<poly::PolyInstance> = Vec::new();
    for i in asset_handler::assets.lock().unwrap().iter() {
        println!("{} Asset {:?} needs to be uploaded to polytoria.", "[WARN]:".yellow(), i.0.clone().into_value());
        assets.push(i.1.clone());
    }

    let poly_json = json!({
        "Version": "2.0.12",
        "FileType": 0,
        "Objects": [
            &poly_instance
        ],
        "NonInstanceObjects": assets
    });

    let trimmed_poly_json_string = poly_json.to_string().replace(",null", "").replace("null", ""); // why do i have to trim the nulls, just let me not add them in the first place :sob:
    let compressed = ruzstd::encoding::compress_to_vec(trimmed_poly_json_string.as_bytes(), ruzstd::encoding::CompressionLevel::Fastest);
    write("main.poly", compressed).expect("Unable to write output file");
    Ok(())
}

fn rbxl_to_poly_instance(rbxl_instance: &rbx_dom_weak::Instance, dom: &rbx_dom_weak::WeakDom) -> poly::PolyInstance {
    let mut poly_instance = poly::PolyInstance::new();
    poly_instance.name = rbxl_instance.name.clone();
    poly_instance = convert_class(rbxl_instance, poly_instance.clone());

    // Hardcode DataModel and Workspace to be renamed to poly equivalents to avoid regeneration
    if poly_instance.name == "DataModel" && poly_instance.class_name == ustr("World") { poly_instance.name = String::from("World"); }
    if poly_instance.name == "Workspace" && poly_instance.class_name == ustr("Environment") { poly_instance.name = String::from("Environment"); }

    let mut unique_names: Vec<String> = Vec::new();
    for i in rbxl_instance.children() {
        let mut child = rbxl_to_poly_instance(dom.get_by_ref(*i).unwrap(), dom);

        if unique_names.contains(&child.name) {
            let mut lowest_available = 2;
            while unique_names.contains(&(child.name.clone() + &lowest_available.to_string())) {
                lowest_available += 1;
            }
            child.name = child.name.clone() + &lowest_available.to_string();
        }

        unique_names.push(child.name.clone());

        poly_instance.children.push(child);
    }

    return poly_instance;
}

pub fn convert_class(rbxl_instance: &rbx_dom_weak::Instance, mut poly_instance: poly::PolyInstance) -> poly::PolyInstance {
  let class: Ustr = rbxl_instance.class.clone();
  poly_instance = converter_registry::get_converter_for_class(class).as_ref()(poly_instance, rbxl_instance);
  //if poly_instance.class_name == ustr("[none]") {
  //  println!("[DEBUG] Failed to parse class \"{}\", ignoring.", class);
  //}

  return poly_instance;
}