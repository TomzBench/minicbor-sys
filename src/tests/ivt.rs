use super::util::*;
use crate::*;
use cddl_cat::parse_cddl;

#[test]
fn parse() {
    let cddl = parse_cddl(&read_cddl("test.cddl")).unwrap();
    let node = flatten(&cddl).unwrap();
    let linked = link(&node).unwrap();

    // Flattened primative types
    assert_eq!(node["u8"], Node::ConstrainedType(ConstrainedType::U8));
    assert_eq!(node["i8"], Node::ConstrainedType(ConstrainedType::I8));
    assert_eq!(node["u16"], Node::ConstrainedType(ConstrainedType::U16));
    assert_eq!(node["i16"], Node::ConstrainedType(ConstrainedType::I16));
    assert_eq!(node["u32"], Node::ConstrainedType(ConstrainedType::U32));
    assert_eq!(node["i32"], Node::ConstrainedType(ConstrainedType::I32));
    assert_eq!(node["u64"], Node::ConstrainedType(ConstrainedType::U64));
    assert_eq!(node["i64"], Node::ConstrainedType(ConstrainedType::I64));

    // A flattened struct
    assert_eq!(
        node["colors"],
        Node::Map(Group {
            members: vec![
                KeyVal::new("car", Node::Foreign("u8".into())).into(),
                KeyVal::new("boat", Node::Foreign("u8".into())).into(),
            ]
        })
    );

    // A flattened linked struct
    assert_eq!(
        linked["colors"],
        LinkedNode::Struct(Fields {
            members: vec![
                LinkedKeyVal::new("car", ConstrainedType::U8.into()).into(),
                LinkedKeyVal::new("boat", ConstrainedType::U8.into()).into(),
            ]
        })
    );

    // A flattened group
    assert_eq!(
        node["ip"],
        Node::Group(Group {
            members: vec![
                KeyVal::new("address", ConstrainedType::Str(16).into()).into(),
                KeyVal::new("port", Node::Foreign("u16".into())).into(),
                KeyVal::new("dhcp", Node::Foreign("is-dhcp".into())).into(),
            ]
        })
    );

    // A flattened linked group
    assert_eq!(
        linked["ip"],
        LinkedNode::Fields(Fields {
            members: vec![
                LinkedKeyVal::new("address", ConstrainedType::Str(16).into()).into(),
                LinkedKeyVal::new("port", ConstrainedType::U16.into()).into(),
                LinkedKeyVal::new("dhcp", ConstrainedType::Bool.into()).into(),
            ]
        })
    );

    // A flattened struct with referened member group
    assert_eq!(
        node["local"],
        Node::Map(Group {
            members: vec![Node::Foreign("ip".into()).into()]
        })
    );

    // A flattened struct with linked member group
    assert_eq!(
        linked["local"],
        LinkedNode::Struct(Fields {
            members: vec![
                LinkedKeyVal::new("address", ConstrainedType::Str(16).into()).into(),
                LinkedKeyVal::new("port", ConstrainedType::U16.into()).into(),
                LinkedKeyVal::new("dhcp", ConstrainedType::Bool.into()).into(),
            ]
        })
    );

    // A flattened struct with referened member struct
    assert_eq!(
        node["local-nested"],
        Node::Map(Group {
            members: vec![KeyVal::new("network", Node::Foreign("local".into()).into()).into()]
        })
    );

    // A flattened struct with linked member struct
    assert_eq!(
        linked["local-nested"],
        LinkedNode::Struct(Fields {
            members: vec![LinkedKeyVal::new(
                "network",
                LinkedNode::ForeignStruct("local".into())
            )]
        })
    );

    // A flattened array with referenced member
    assert_eq!(
        node["mac"],
        Node::Array(Array {
            ty: Box::new(Node::Foreign("small".into())),
            len: 6
        })
    );

    // A flattened array with linked member
    assert_eq!(
        linked["mac"],
        LinkedNode::Array(LinkedArray {
            ty: Box::new(LinkedNode::ConstrainedType(ConstrainedType::U8)),
            len: 6
        })
    );

    // flatten literal int
    assert_eq!(node["lit"], Node::Foreign("boop".into()));

    // flattened and linked literal int
    assert_eq!(linked["lit"], LinkedNode::Literal(Literal::UInt(3)));

    // flatten literal str
    assert_eq!(node["bar"], Node::Literal(Literal::Str("bar".into())));

    // flattened and linked literal string
    assert_eq!(
        linked["bar"],
        LinkedNode::Literal(Literal::Str("bar".into()))
    );
}
