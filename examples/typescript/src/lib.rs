mod utils;

pub use minicbor::{self, CborLen, Decode, Decoder, Encode, Encoder};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub trait FromBytes {
    fn from_bytes(&self) -> core::result::Result<&str, core::str::Utf8Error>;
}

impl FromBytes for [u8] {
    fn from_bytes(&self) -> core::result::Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self[0..]).map(|s| match s.find("\0") {
            Some(n) => &s[0..n],
            None => s,
        })
    }
}

struct StrToBytes<const N: usize> {}
impl<'de, const N: usize> serde::de::Visitor<'de> for StrToBytes<N> {
    type Value = [u8; N];
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> {
        let mut ret: [u8; N] = [0; N];
        let min = if s.len() < N { s.len() } else { N };
        ret[0..min].copy_from_slice(&s.as_bytes()[0..min]);
        Ok(ret)
    }
}

fn ser_bytes_as_str<S: serde::Serializer>(ty: &[u8], s: S) -> std::result::Result<S::Ok, S::Error> {
    ty.from_bytes()
        .map_err(|e| serde::ser::Error::custom(format!("{}", e)))
        .and_then(|val| s.serialize_str(val))
}

fn de_str_as_bytes<'de, D, const N: usize>(de: D) -> std::result::Result<[u8; N], D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    de.deserialize_str(StrToBytes::<N> {})
}

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

#[derive(Clone, CborLen, Debug, Serialize, Deserialize, Encode, Decode)]
#[wasm_bindgen]
pub struct Network {
    #[n(0)]
    pub dhcp: bool,
    #[n(1)]
    #[serde(serialize_with = "ser_bytes_as_str")]
    #[serde(deserialize_with = "de_str_as_bytes")]
    pub ip: [u8; 16],
    #[n(2)]
    #[serde(serialize_with = "ser_bytes_as_str")]
    #[serde(deserialize_with = "de_str_as_bytes")]
    pub sn: [u8; 16],
    #[n(3)]
    #[serde(serialize_with = "ser_bytes_as_str")]
    #[serde(deserialize_with = "de_str_as_bytes")]
    pub gw: [u8; 16],
    #[cbor(n(4))]
    pub mac: [u8; 6],
}

#[derive(Debug, Default, Encode, Decode, CborLen, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Foo {
    // TODO String not supported.
    //      Distinguish u8 from str with custom encode/decode ser/de implementations
    //      IE: serialize_with and deserialize_with encode_with and decode_with. Where these impls
    //      will serialize strings into byte arrays and deserialize byte arrays as strings as
    //      necessary. (a real [u8] will not need serialize_with and friends)
    #[n(0)]
    #[serde(serialize_with = "ser_bytes_as_str")]
    #[serde(deserialize_with = "de_str_as_bytes")]
    name: [u8; 8],
    #[n(1)]
    data: [u8; 3],
    #[n(2)]
    ver: u8,
}

#[wasm_bindgen]
impl Foo {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Foo {
        Foo::default()
    }

    #[wasm_bindgen]
    pub fn as_json(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap()
    }

    #[wasm_bindgen]
    pub fn from_json(json: &str) -> Foo {
        serde_json::from_str(json).unwrap()
    }

    #[wasm_bindgen]
    pub fn as_cbor(&self) -> Vec<u8> {
        unimplemented!()
    }

    #[wasm_bindgen]
    pub fn from_cbor(_cbor: &[u8]) -> Foo {
        unimplemented!()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        std::str::from_utf8(&self.name)
            .expect("invalid utf8 inside of name")
            .to_string()
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
        name: Default::default(),
        data: Default::default(),
        ver: 1,
    })
    .unwrap()
}


