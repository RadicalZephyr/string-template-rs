use parse::st_group;

st_group! {
    static ref simple_bracket {
        a() ::= <<foo>>
    }
}
