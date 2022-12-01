mod gen;
mod ivt;
mod util;
mod validate;

#[cfg(test)]
mod tests;

use cddl_cat::ast;
use gen::{gen_cargo, gen_extra, gen_lib};
use ivt::flatten_rule;
use std::collections::BTreeMap;
use util::ValidateError;
use validate::link_node;

pub use gen::{Language, Options, RenderError, RenderResult};
pub use ivt::{Array, ConstrainedType, FlattenResult, Group, KeyVal, Literal, Node};
pub use validate::{Fields, LinkedArray, LinkedKeyVal, LinkedNode};

/// Take a string of CDDL text, and create a Flattened representation of
/// data types useful for further processing and generating code with.
pub fn parse(cddl: &str) -> FlattenResult<BTreeMap<String, LinkedNode>> {
    cddl_cat::parse_cddl(cddl)
        .map_err(ValidateError::from)
        .and_then(|nodes| flatten(&nodes))
        .and_then(|nodes| link(&nodes))
}

/// Take already parsed CDDL and generate a Representation that is useful
/// for codegen. This will do a first pass but will not resolve links.
///
/// NOTE prefer parse wrapper. Access to an unlinked tree not particularly useful
/// outside of this module, but is temporarily here for development.
pub fn flatten(cddl: &ast::Cddl) -> FlattenResult<BTreeMap<String, Node>> {
    cddl.rules.iter().map(flatten_rule).collect()
}

/// An already flattened and parsed CDDL will have unresolved references in
/// it's tree. This method will resolve those references while preserving flatness
///
/// NOTE prefer parse wrapper. Access to an unlinked tree not particularly useful
/// outside of this module, but is temporarily here for development.
pub fn link(nodes: &BTreeMap<String, Node>) -> FlattenResult<BTreeMap<String, LinkedNode>> {
    nodes
        .iter()
        .map(|(key, node)| link_node(node, nodes).map(|node| (key.clone(), node)))
        .collect()
}

/// We have some CDDL and we want to generate lib.rs
/// TODO create a cursor and return length from our bindings...
pub fn render_lib(s: &str, mode: &Options) -> RenderResult<String> {
    parse(s)
        .map_err(RenderError::from)
        .and_then(|nodes| gen_lib(nodes, mode))
}

pub fn render_extra(s: &str, mode: &Options) -> RenderResult<String> {
    parse(s)
        .map_err(RenderError::from)
        .and_then(|nodes| gen_extra(nodes, mode))
}

/// Generate a cargo template
pub fn render_cargo(name: &str, mode: &Options) -> RenderResult<String> {
    gen_cargo(name, mode)
}
