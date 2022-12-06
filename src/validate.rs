use crate::ivt::*;
use crate::util;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
use util::ValidateError;

/// Similar to a Group, but fully resolved with fields
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fields {
    /// The Field members of a struct
    pub members: Vec<LinkedKeyVal>,
}

/// A LinkedKeyVal accept as a struct instead of a tuple
/// Useful for serde
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkedKeyValStruct {
    key: String,
    val: LinkedNode,
}

/// When we serialize/deserialize we use struct form, and convert into tuple
impl From<LinkedKeyValStruct> for LinkedKeyVal {
    fn from(node: LinkedKeyValStruct) -> LinkedKeyVal {
        LinkedKeyVal(node.key, node.val)
    }
}

/// When we serialize/deserialize we use struct form, and convert into tuple
impl From<LinkedKeyVal> for LinkedKeyValStruct {
    fn from(node: LinkedKeyVal) -> LinkedKeyValStruct {
        LinkedKeyValStruct {
            key: node.0,
            val: node.1,
        }
    }
}

/// Similar to KeyVal, but nodes are linked
/// A linked array, similiar to ivt::Array, except with a LinkedNode
/// NOTE LinkedKeyVal and LinkedArray could share same definition with
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "LinkedKeyValStruct", into = "LinkedKeyValStruct")]
pub struct LinkedKeyVal(pub(crate) String, pub(crate) LinkedNode);
impl LinkedKeyVal {
    pub fn new<'a, K: Into<Cow<'a, str>>>(key: K, node: LinkedNode) -> LinkedKeyVal {
        LinkedKeyVal(key.into().into(), node)
    }
}

/// Helper when creating Maps from Key/Value tuples.
impl From<(String, LinkedNode)> for LinkedKeyVal {
    fn from(t: (String, LinkedNode)) -> LinkedKeyVal {
        LinkedKeyVal(t.0, t.1)
    }
}

/// A linked array, similiar to ivt::Array, except with a LinkedNode
/// NOTE LinkedKeyVal and LinkedArray could share same definition with
///      ivt::KeyVal and ivt::Array using generics however getting impls to play
///      nice with serde was over my head
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkedArray {
    pub len: usize,
    pub ty: Box<LinkedNode>,
}

/// When we have an IVT node, we lookup unresolved types and build a complete tree
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "meta")]
pub enum LinkedNode {
    /// A Literal type such as "true" or 3 or "hello"
    Literal(Literal),
    /// A primative type fully qualified
    ConstrainedType(ConstrainedType),
    /// An array is of a fixed size of a single type
    Array(LinkedArray),
    /// A group of fields missing context (might be a struct)
    Fields(Fields),
    /// A fully qualified struct with fields (Can only exist at top level)
    Struct(Fields),
    /// If a struct contains a nested struct, we store flatten instead of nest
    ForeignStruct(String),
}

/// A Enum Variant of a node, so we provide helper convert to the enum
impl From<ConstrainedType> for LinkedNode {
    fn from(ty: ConstrainedType) -> LinkedNode {
        LinkedNode::ConstrainedType(ty)
    }
}

/// Main (only) entry function to this module
pub(crate) fn link_node(node: &Node, ctx: &BTreeMap<String, Node>) -> FlattenResult<LinkedNode> {
    match node {
        Node::Literal(lit) => Ok(LinkedNode::Literal(lit.clone())),
        Node::ConstrainedType(t) => Ok(LinkedNode::ConstrainedType(t.clone())),
        Node::Foreign(t) => link_foreign(t, ctx),
        Node::Group(g) => link_group(g, ctx),
        Node::Map(g) => link_struct(g, ctx),
        Node::Array(a) => link_array(a, ctx),
        _ => unimplemented!(),
    }
}

fn link_array(arr: &Array, ctx: &BTreeMap<String, Node>) -> FlattenResult<LinkedNode> {
    // Similar to link_foreign, we only accept certain types as an array, and we don't follow
    // nesting types so we can flatten them
    link_node(&arr.ty, ctx).and_then(|node| match node {
        // We don't accept nested arrays
        LinkedNode::Array(..) => Err(ValidateError::InvalidArray),
        // We don't accept inline fields inside an array
        LinkedNode::Fields(_) => Err(ValidateError::InvalidArray),
        // We don't accept inline structs defined inside an array
        LinkedNode::Struct(_) => Err(ValidateError::InvalidArray),
        // ConstainedType or Struct defined externally are the only acceptable array types
        n => Ok(LinkedNode::Array(LinkedArray {
            ty: Box::new(n),
            len: arr.len,
        })),
    })
}

fn link_foreign(key: &str, ctx: &BTreeMap<String, Node>) -> FlattenResult<LinkedNode> {
    // When linking a "foreign" struct, we simply note it's remote name instead of
    // following the struct deeper.
    ctx.get(key)
        .ok_or_else(|| ValidateError::ForeignKey(key.into()))
        .and_then(|node| link_node(node, ctx))
        .map(|node| match node {
            LinkedNode::Struct(_s) => LinkedNode::ForeignStruct(key.into()),
            node => node,
        })
}

fn link_group(map: &Group, ctx: &BTreeMap<String, Node>) -> FlattenResult<LinkedNode> {
    link_field_key_values(map, ctx).map(|members| LinkedNode::Fields(Fields { members }))
}

fn link_struct(map: &Group, ctx: &BTreeMap<String, Node>) -> FlattenResult<LinkedNode> {
    link_field_key_values(map, ctx).map(|members| LinkedNode::Struct(Fields { members }))
}

fn link_field_key_values(
    map: &Group,
    ctx: &BTreeMap<String, Node>,
) -> FlattenResult<Vec<LinkedKeyVal>> {
    Ok(link_fields(map, ctx)?
        .into_iter()
        .map(LinkedKeyVal::from)
        .collect())
}

fn link_fields(
    map: &Group,
    ctx: &BTreeMap<String, Node>,
) -> FlattenResult<Vec<(String, LinkedNode)>> {
    Ok(map
        .members
        .iter()
        .map(|node| link_field_member(node, ctx))
        .collect::<FlattenResult<Vec<Vec<(String, LinkedNode)>>>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn link_field_member(
    node: &Node,
    ctx: &BTreeMap<String, Node>,
) -> FlattenResult<Vec<(String, LinkedNode)>> {
    match node {
        Node::KeyVal(KeyVal(k, v)) => link_node(v, ctx).map(|n| vec![(k.clone(), n)]),
        Node::Foreign(key) => match ctx.get(key) {
            Some(Node::Group(g)) => link_fields(g, ctx),
            //Some(Node::Map(g)) => link_struct(g, ctx).map(|n| vec![(key.clone(), n)]),
            _ => Err(ValidateError::InvalidType),
        },
        _ => Err(ValidateError::InvalidGroupMissingKey),
    }
}
