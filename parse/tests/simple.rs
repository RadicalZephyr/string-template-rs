use parse::st_group;

st_group! {
    static ref simple_bracket {
        a() ::= <<foo>>
    }
}

#[test]
fn parse_literal_group() {
    let a = simple_bracket
        .get("a")
        .expect("unexpectedly failed to find templat a");
}
