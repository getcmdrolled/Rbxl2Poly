pub mod poly;

use rbx_binary;
use rbx_dom_weak;
use std::fs::{File, write};
use std::io::BufReader;
use std::env::args;
use serde_json::json;
use anyhow::{Context, Result};
use ruzstd;

fn main() -> Result<()> {
    let argument = args().nth(1).with_context(|| "No file provided!")?;
    let input = BufReader::new(File::open(argument).with_context(|| "Unable to read file")?);
    let dom = rbx_binary::from_reader(input).with_context(|| "No DOM in provided file")?;

    let mut poly_instance = rbxl_to_poly_instance(dom.root(), &dom);
    poly_instance.name = String::from("World"); // me no likee referring to the World as DataModel :skull:
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

    for i in rbxl_instance.properties.clone() {
        poly_instance = poly::convert_property(i, poly_instance.clone());
    }

    for i in rbxl_instance.children() {
        let child = rbxl_to_poly_instance(dom.get_by_ref(*i).unwrap(), dom);
        poly_instance.children.push(child);
    }

    return poly_instance;
}