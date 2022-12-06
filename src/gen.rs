use crate::ivt::ConstrainedType;
use crate::{LinkedArray, LinkedKeyVal, LinkedNode, Literal, ValidateError};
use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, to_value};
use std::collections::{BTreeMap, HashMap};
use std::include_str;
use std::{error, fmt};
use tera::Context as TeraContext;
use tera::Error as TeraError;
use tera::Result;
use tera::Tera;
use tera::Value;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let lib_tmpl = include_str!("__templates__/lib.rs.tmpl");
        let macros = include_str!("__templates__/macros.tmpl");
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![("lib.rs.tmpl", lib_tmpl), ("macros.tmpl", macros)])
            .unwrap();
        tera.register_filter("field", filter_field);
        tera.register_filter("field_attr", filter_field_attr);
        tera.register_filter("field_default", filter_field_default);
        tera.register_filter("rename", filter_rename);
        tera.register_filter("literal", filter_literal);
        tera.register_filter("structs", filter_structs);
        tera.register_filter("nodes", filter_nodes);
        tera
    };
}

macro_rules! field_arr {
    ($key:expr, $ty:literal, $len:expr) => {
        format!("pub {}: [{}; {}]", $key, $ty, $len)
    };
}

/// Our error type
#[derive(Debug)]
pub enum RenderError {
    Validate(ValidateError),
    Render(TeraError),
    Case(Value),
}

impl From<ValidateError> for RenderError {
    fn from(value: ValidateError) -> Self {
        Self::Validate(value)
    }
}

impl From<TeraError> for RenderError {
    fn from(value: TeraError) -> Self {
        Self::Render(value)
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::Validate(e) => e.fmt(f),
            RenderError::Render(e) => e.fmt(f),
            RenderError::Case(e) => write!(f, "invalid case {}", e),
        }
    }
}

impl error::Error for RenderError {}

/// All methods in this module return a RenderResult
pub type RenderResult<T> = std::result::Result<T, RenderError>;

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    C,
    Rust,
    Typescript,
}

/// Options for modifying behavior of rendered code
#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    pub language: Language,
    pub prefix: Option<String>,
}

/// Main entry to this module
pub(crate) fn gen_lib(cddl: BTreeMap<String, LinkedNode>, opts: &Options) -> RenderResult<String> {
    let mut ctx = TeraContext::new();
    ctx.insert("cddl", &cddl);
    ctx.insert("options", opts);
    println!("{:?}", ctx.get("options"));
    TEMPLATES
        .render("lib.rs.tmpl", &ctx)
        .map_err(RenderError::from)
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Case {
    Fn,
    Struct,
}

/// Take an input string and change it's case
/// Can request name for a fn, struct
/// Functions are always snake_case and can be prefixed
/// Structs are snake_case or CamelCase depending if bindings or not
fn filter_rename(val: &Value, map: &HashMap<String, Value>) -> Result<Value> {
    // Read input as a string to perform case change
    let input = val
        .as_str()
        .ok_or_else(|| TeraError::msg(format!("unexpected input to name filter {:?}", val)))?;
    map.get("case")
        .and_then(|val| val.as_str())
        .ok_or_else(|| TeraError::msg(format!("unexpected value for name filter")))
        .and_then(|case| caseify(input, case, map))
        .map(Value::String)
}

/// Take a Literal and declare a const type
fn filter_literal(val: &Value, map: &HashMap<String, Value>) -> Result<Value> {
    // get the name of the literal and format to "SHOUTY_SNAKE_CASE"
    let name = map
        .get("name")
        .and_then(|val| val.as_str())
        .ok_or_else(|| TeraError::msg(format!("unexpected value for name filter")))
        .and_then(|name| caseify(name, "const", map))?;
    // get the enum of our type
    let lit = from_value::<Literal>(val.clone())?;
    match lit {
        Literal::Bool(b) => Ok(format!("pub const {}: bool = {};", name, b)),
        Literal::Int(i) => Ok(format!("pub const {}: i32 = {};", name, i)),
        Literal::UInt(u) => Ok(format!("pub const {}: u32 = {};", name, u)),
        Literal::Str(s) => Ok(format!("pub const {}: 'static str = \"{}\";", name, s)),
        Literal::Char(c) => Ok(format!("pub const {}: char = '{}';", name, c)),
        Literal::Bytes(_b) => Err(TeraError::msg(format!("unsupported literal"))),
    }
    .map(Value::String)
}

/// Take a Literal and declare a const type
fn filter_structs(val: &Value, _map: &HashMap<String, Value>) -> Result<Value> {
    let cddl = from_value::<BTreeMap<String, LinkedNode>>(val.clone())?;
    let filtered = cddl
        .into_iter()
        .filter(|(_key, node)| match node {
            LinkedNode::Struct(_) => true,
            _ => false,
        })
        .collect::<BTreeMap<String, LinkedNode>>();
    to_value(filtered).map_err(|_| TeraError::msg(format!("infallible conversion failure")))
}

/// Our HashMap of CDDL linked nodes can be filtered based on kind of node it is (aka struct or
/// literal, etc)
fn filter_nodes(val: &Value, map: &HashMap<String, Value>) -> Result<Value> {
    let value = map
        .get("value")
        .and_then(|val| val.as_str())
        .ok_or_else(|| TeraError::msg(format!("unexpected value for name filter")))?;
    let cddl = from_value::<BTreeMap<String, LinkedNode>>(val.clone())?;
    let filtered = cddl
        .into_iter()
        .filter(|(_key, node)| match node {
            LinkedNode::Struct(_) if value == "struct" => true,
            LinkedNode::Literal(_) if value == "literal" => true,
            _ => false,
        })
        .collect::<BTreeMap<String, LinkedNode>>();
    to_value(filtered).map_err(|_| TeraError::msg(format!("infallible conversion failure")))
}

fn to_fn_case(name: &str, verb: &str, lang: Language, prefix: Option<String>) -> String {
    match (lang, prefix) {
        (_, Some(prefix)) => format!("{}{}_{}", prefix, verb, name).to_snake_case(),
        (_, _) => format!("{}_{}", verb, name).to_snake_case(),
    }
}

fn to_snake_case(val: &str, language: Language, prefix: Option<String>) -> String {
    match (language, prefix) {
        (Language::C, None) => val.to_snake_case(),
        (Language::C, Some(prefix)) => format!("{}{}", prefix, val).to_snake_case(),
        (_, _) => val.to_snake_case(),
    }
}

fn to_shouty_snake_case(val: &str, lang: Language, prefix: Option<String>) -> String {
    match (lang, prefix) {
        (Language::C, None) => val.to_shouty_snake_case(),
        (Language::C, Some(prefix)) => format!("{}{}", prefix, val).to_shouty_snake_case(),
        (_, _) => val.to_shouty_snake_case(),
    }
}

fn to_upper_camel_case(val: &str, lang: Language, prefix: Option<String>) -> String {
    match (lang, prefix) {
        (Language::C, None) => val.to_upper_camel_case(),
        (Language::C, Some(prefix)) => format!("{}{}", prefix, val).to_upper_camel_case(),
        (_, _) => val.to_upper_camel_case(),
    }
}

fn caseify(s: &str, case: &str, opts: &HashMap<String, Value>) -> Result<String> {
    let (lang, pre) = opts
        .get("options")
        .and_then(|v| serde_json::from_value::<Options>(v.clone()).ok())
        .map(|opts| (opts.language, opts.prefix))
        .unwrap_or((Language::default(), None));

    if case == "struct" {
        match lang {
            Language::C => Ok(to_snake_case(s, lang, pre)),
            Language::Rust | Language::Typescript => Ok(to_upper_camel_case(s, lang, pre)),
        }
    } else if case == "fn" {
        // TODO for the function case, unwrap a "verb" arg
        let verb = opts
            .get("verb")
            .and_then(|val| val.as_str())
            .ok_or_else(|| TeraError::msg(format!("")))?;
        Ok(to_fn_case(s, verb, lang, pre))
    } else if case == "enum" || case == "const" || case == "define" {
        Ok(to_shouty_snake_case(s, lang, pre))
    } else {
        Err(TeraError::msg(format!("unsupported rename {}", case)))
    }
}

fn filter_field_default(val: &Value, _map: &HashMap<String, Value>) -> Result<Value> {
    let LinkedKeyVal(key, val) = from_value::<LinkedKeyVal>(val.clone())
        .map(|LinkedKeyVal(key, val)| LinkedKeyVal(key.to_snake_case(), val))?;
    match val {
        LinkedNode::Array(LinkedArray { ty, len }) => match *ty {
            LinkedNode::ConstrainedType(ConstrainedType::U8) => {
                Ok(format!("{}: [0; {}]", key, len))
            }
            _ => unimplemented!(),
        },
        _ => Ok(format!("{}: Default::default()", key)),
    }
    .map(Value::String)
}

fn filter_field_attr(val: &Value, map: &HashMap<String, Value>) -> Result<Value> {
    let lang = map
        .get("language")
        .and_then(|val| from_value::<String>(val.clone()).ok())
        .unwrap_or("c".to_string());
    match lang.as_ref() {
        "c" => filter_field_attr_c(val, map),
        _ => filter_field_attr_rust(val, map),
    }
}

fn filter_field_attr_rust(val: &Value, map: &HashMap<String, Value>) -> Result<Value> {
    let LinkedKeyVal(_key, val) = from_value::<LinkedKeyVal>(val.clone())?;
    map.get("index")
        .and_then(|i| i.as_i64())
        .ok_or_else(|| TeraError::msg(format!("expected number")))
        .map(|n| match val {
            LinkedNode::Array(LinkedArray { ty, len }) => match *ty {
                LinkedNode::ConstrainedType(ConstrainedType::U8) if len <= 32 => {
                    Value::String(format!(r#"#[cbor(n({}), with = "minicbor::bytes")] "#, n))
                }
                LinkedNode::ConstrainedType(ConstrainedType::U8) if len > 32 => {
                    Value::String(format!(
                        r#"#[cbor(n({}), with = "minicbor::bytes")]
                           #[serde(with="BigArray")]"#,
                        n
                    ))
                }
                _ => Value::String(format!("#[n({})]", n)),
            },
            LinkedNode::ConstrainedType(ConstrainedType::Str(_)) => Value::String(format!(
                r#"#[cbor(n({}), with = "minicbor::bytes")] 
                   #[serde(serialize_with = "ser_bytes_as_str")] 
                   #[serde(deserialize_with = "de_str_as_bytes")]"#,
                n
            )),
            _ => Value::String(format!("#[n({})]", n)),
        })
}

fn filter_field_attr_c(val: &Value, map: &HashMap<String, Value>) -> Result<Value> {
    let LinkedKeyVal(_key, val) = from_value::<LinkedKeyVal>(val.clone())?;
    map.get("index")
        .and_then(|i| i.as_i64())
        .ok_or_else(|| TeraError::msg(format!("expected number")))
        .map(|n| match val {
            LinkedNode::Array(LinkedArray { ty, .. }) => match *ty {
                LinkedNode::ConstrainedType(ConstrainedType::U8) => {
                    Value::String(format!(r#"#[cbor(n({}), with = "minicbor::bytes")] "#, n))
                }
                _ => Value::String(format!("#[n({})]", n)),
            },
            LinkedNode::ConstrainedType(ConstrainedType::Str(_)) => {
                Value::String(format!(r#"#[cbor(n({}), with = "minicbor::bytes")]"#, n))
            }
            _ => Value::String(format!("#[n({})]", n)),
        })
}

/// Take a field node and convert to a field member according to lang type
fn filter_field(val: &Value, map: &HashMap<String, Value>) -> Result<Value> {
    match map.get("lang") {
        Some(Value::String(s)) if s.to_lowercase() == "rs" => filter_field_rs(val, map),
        _ => filter_field_c(val, map),
    }
}

/// Take a field node and convert to a field member for rust struct
fn filter_field_rs(val: &Value, map: &HashMap<String, Value>) -> Result<Value> {
    // let opts = map
    //     .get("options")
    //     .and_then(|v| serde_json::from_value::<Options>(v.clone()).ok())
    //     .map(|opts| (opts.bindings, opts.prefix));
    let LinkedKeyVal(key, val) = from_value::<LinkedKeyVal>(val.clone())
        .map(|LinkedKeyVal(key, val)| LinkedKeyVal(key.to_snake_case(), val))?;
    match val {
        LinkedNode::ConstrainedType(ConstrainedType::U8) => Ok(format!("pub {}: u8", key)),
        LinkedNode::ConstrainedType(ConstrainedType::U16) => Ok(format!("pub {}: u16", key)),
        LinkedNode::ConstrainedType(ConstrainedType::U32) => Ok(format!("pub {}: u32", key)),
        LinkedNode::ConstrainedType(ConstrainedType::U64) => Ok(format!("pub {}: u64", key)),
        LinkedNode::ConstrainedType(ConstrainedType::I8) => Ok(format!("pub {}: i8", key)),
        LinkedNode::ConstrainedType(ConstrainedType::I16) => Ok(format!("pub {}: i16", key)),
        LinkedNode::ConstrainedType(ConstrainedType::I32) => Ok(format!("pub {}: i32", key)),
        LinkedNode::ConstrainedType(ConstrainedType::I64) => Ok(format!("pub {}: i64", key)),
        LinkedNode::ConstrainedType(ConstrainedType::Bool) => Ok(format!("pub {}: bool", key)),
        LinkedNode::ConstrainedType(ConstrainedType::Str(n)) => Ok(field_arr!(key, "u8", n)),
        LinkedNode::ForeignStruct(s) => Ok(format!("pub {}: {}", key, caseify(&s, "struct", map)?)),
        LinkedNode::Array(LinkedArray { ty, len }) => match *ty {
            LinkedNode::ConstrainedType(ConstrainedType::U8) => Ok(field_arr!(key, "u8", len)),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
    .map(Value::String)
}

/// Take a field node and convert to a field member for a c struct
fn filter_field_c(val: &Value, _map: &HashMap<String, Value>) -> Result<Value> {
    let LinkedKeyVal(key, val) = from_value::<LinkedKeyVal>(val.clone())
        .map(|LinkedKeyVal(key, val)| LinkedKeyVal(key.to_snake_case(), val))?;
    match val {
        LinkedNode::ConstrainedType(ConstrainedType::U8) => Ok(format!("uint8_t {}", key)),
        LinkedNode::ConstrainedType(ConstrainedType::U16) => Ok(format!("uint16_t {}", key)),
        LinkedNode::ConstrainedType(ConstrainedType::U32) => Ok(format!("uint32_t {}", key)),
        LinkedNode::ConstrainedType(ConstrainedType::U64) => Ok(format!("uint64_t {}", key)),
        LinkedNode::ConstrainedType(ConstrainedType::I8) => Ok(format!("int8_t {}", key)),
        LinkedNode::ConstrainedType(ConstrainedType::I16) => Ok(format!("int16_t {}", key)),
        LinkedNode::ConstrainedType(ConstrainedType::I32) => Ok(format!("int32_t {}", key)),
        LinkedNode::ConstrainedType(ConstrainedType::I64) => Ok(format!("int64_t {}", key)),
        LinkedNode::ConstrainedType(ConstrainedType::Bool) => Ok(format!("bool {}", key)),
        LinkedNode::ConstrainedType(ConstrainedType::Str(n)) => Ok(format!("char {}[{}]", key, n)),
        _ => unimplemented!(),
    }
    .map(Value::String)
}
