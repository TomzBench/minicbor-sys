extern crate cbindgen;
extern crate minicbor_bindgen;
use minicbor_bindgen::{Language, Options};
use std::{env, fs, path::PathBuf, result};

fn main() -> result::Result<(), minicbor_bindgen::Error> {
    let opts = Options {
        language: Language::C,
        prefix: None,
        version: None,
    };
    let lib = minicbor_bindgen::generate("examples/data.cddl", opts)?;
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    fs::write(out, lib).expect("failed to write bindings");

    Ok(())
}
