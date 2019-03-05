use string_template_macro::{st_group, st_test};
use string_template_test::{get_template, TemplateTestExt as _};

st_group! {
    static ref literal_group {
        a() ::= "foo"
        b() ::= r#"bar "things" { () } () baz => "#
    }
}

#[test]
fn parse_literal_group() {
    let a = get_template(&literal_group, "a");
    assert_eq!("foo", format!("{}", a.render()));

    let b = get_template(&literal_group, "b");
    assert_eq!(
        r#"bar "things" { () } () baz => "#,
        format!("{}", b.render())
    );
}

#[test]
fn use_static_methods() {
    let a = literal_group.a();
    assert_eq!("foo", format!("{}", a.render()));

    let b = literal_group.b();
    assert_eq!(
        r#"bar "things" { () } () baz => "#,
        format!("{}", b.render())
    );
}

st_group! {
    static ref attribute_group {
        c(x) ::= "<x>"
        d(x,y) ::= "<y><x>"
    }
}

#[test]
fn render_attributes() {
    let mut c = attribute_group.c();
    c.add_expect("x", "Things");
    assert_eq!("Things", c.render());
}

#[test]
fn render_multiple_attributes() {
    let mut d = attribute_group.d();
    d.add_expect("x", "Moe");
    d.add_expect("y", "Curly");
    assert_eq!("CurlyMoe", d.render());
}

st_test! {
    test_name: chained_attributes,
    render_root: d,
    template_group: {
        d(x,y) ::= "<y>:<x>"
    },
    attributes: {
        "x": "Moe",
        "y": "Curly",
    },
    expected: "Curly:Moe",
}
