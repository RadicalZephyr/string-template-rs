use serde_derive::Serialize;

use string_template_macro::st_test;
use string_template_test::parse_template;

st_test! {
    test_name: null_attribute,
    render_root: t,
    template_group: {
        t(x,y) ::= "hi <name>!"
    },
    attributes: {},
    expected: "hi !"
}

st_test! {
    test_name: literal_template,
    render_root: t,
    template_group: {
        t() ::= r#"bar "things" { () } () baz => "#
    },
    attributes: {},
    expected: r#"bar "things" { () } () baz => "#
}

st_test! {
    test_name: simple_attribute,
    render_root: t,
    template_group: {
        t(name) ::= "hi <name>!"
    },
    attributes: {
        "name": "Ter"
    },
    expected: "hi Ter!"
}

st_test! {
    test_name: chained_attributes,
    render_root: t,
    template_group: {
        t(x, name) ::= "<x>:<name>!"
    },
    attributes: {
        "x": 1,
        "name": "Ter",
    },
    expected: "1:Ter!"
}

st_test! {
    test_name: multi_attribute,
    render_root: t,
    template_group: {
        t(names) ::= "hi <names>!"
    },
    attributes: {
        "names": "Ter",
        "names": "Tom",
    },
    expected: "hi TerTom!"
}

st_test! {
    test_name: list_attribute,
    render_root: t,
    template_group: {
        t(names) ::= "hi <names>!"
    },
    attributes: {
        "names": { vec!["Ter", "Tom"] },
        "names": "Sumana",
    },
    expected: "hi TerTomSumana!"
}

#[derive(Serialize)]
struct User {
    id: u8,
    name: &'static str,
}

st_test! {
    test_name: attribute_properties,
    render_root: t,
    template_group: {
        t(user) ::= "<user.id>: <user.name>"
    },
    attributes: {
        "user": { User { id: 1, name: "John" } },
    },
    expected: "1: John",
}

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
                m
        }
    };
);

st_test! {
    test_name: property_with_no_attribute,
    render_root: t,
    template_group: {
        t(foo, ick) ::= "<foo.a>: <ick>"
    },
    attributes: {
        "foo": { map! {"a" => "b"} },
    },
    expected: "b: ",
}

st_test! {
    test_name: template_prop,
    render_root: t,
    template_group: {
        t(t) ::= "<t.x>"
    },
    attributes: {
        "t": {
            let mut t = parse_template("<x>");
            t.add_expect("x", "Ter");
            t
        },
    },
    expected: "Ter",
}
