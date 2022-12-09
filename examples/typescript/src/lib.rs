mod utils;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
     export interface IFoo {
         name: string,
         data: Uint8Array,
         ver: number,
     }
 "#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IFoo")]
    pub type IFoo;
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Foo {
    name: String,
    data: [u8; 32],
    ver: u8,
}

#[wasm_bindgen]
impl Foo {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Foo {
        Foo {
            name: "hi".into(),
            ..Foo::default() //
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.to_vec()
    }

    #[wasm_bindgen(getter)]
    pub fn ver(&self) -> u8 {
        self.ver
    }
}

#[wasm_bindgen]
pub fn foo_make() -> JsValue {
    serde_wasm_bindgen::to_value(&Foo {
        name: "hi".into(),
        data: Default::default(),
        ver: 1,
    })
    .unwrap()
}
