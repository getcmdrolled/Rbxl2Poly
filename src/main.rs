pub mod poly;

use rbx_binary;
use rbx_dom_weak;
use std::fs::{File, write};
use std::io::BufReader;
use std::env::args;
use serde_json::json;
use anyhow::{Context, Result};
use ruzstd;
use ustr::ustr;

fn main() -> Result<()> {
    let argument = args().nth(1).with_context(|| "No file provided!")?;
    let input = BufReader::new(File::open(argument).with_context(|| "Unable to read file")?);
    let dom = rbx_binary::from_reader(input).with_context(|| "No DOM in provided file")?;

    let poly_instance = rbxl_to_poly_instance(dom.root(), &dom);
    let poly_json = json!({
        "Version": "2.0.9",
        "FileType": 0,
        "Objects": [
            &poly_instance
        ],
        "NonInstanceObjects": []
    });

    let trimmed_poly_json_string = poly_json.to_string().replace(",null", "").replace("null", ""); // why do i have to trim the nulls, just let me not add them in the first place :sob:
    let compressed = ruzstd::encoding::compress_to_vec(trimmed_poly_json_string.as_bytes(), ruzstd::encoding::CompressionLevel::Fastest);
    write("main.poly", compressed).expect("Unable to write output file");
    println!("This tool is still in very early stages, so it is designed to only generate the main.poly. Please add this to a blank project, and your rbxl content should (mostly) be there.");
    Ok(())
}

fn rbxl_to_poly_instance(rbxl_instance: &rbx_dom_weak::Instance, dom: &rbx_dom_weak::WeakDom) -> poly::PolyInstance {
    let mut poly_instance = poly::PolyInstance::new();
    poly_instance.name = rbxl_instance.name.clone();
    poly_instance = poly::convert_class(rbxl_instance.class.clone(), poly_instance.clone());

    // Hardcode DataModel and Workspace to be renamed to poly equivalents to avoid regeneration
    if poly_instance.name == "DataModel" && poly_instance.class_name == ustr("World") { poly_instance.name = String::from("World"); }
    if poly_instance.name == "Workspace" && poly_instance.class_name == ustr("Environment") { poly_instance.name = String::from("Environment"); }

    for i in rbxl_instance.properties.clone() {
        poly_instance = poly::convert_property(i, poly_instance.clone());
    }

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