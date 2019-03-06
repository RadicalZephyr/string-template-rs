use string_template_macro::st_test;

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
