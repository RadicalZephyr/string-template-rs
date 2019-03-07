use string_template::{AsDynamicTemplate as _, GroupBody, StaticGroup};

use quote::ToTokens;
use quote::{quote, quote_spanned};

use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, token, Expr, Ident, LitBool, LitStr, Token, Visibility};

pub struct Test {
    test_name: Ident,
    render_root: Ident,
    template_group: GroupBody,
    template_group_brace: token::Brace,
    attributes: Punctuated<(LitStr, Expr), Token![,]>,
    attributes_brace: token::Brace,
    expected_value: LitStr,
    debug: bool,
}

impl Test {
    fn make_debug_print(&self, template_name: &Ident) -> proc_macro2::TokenStream {
        let template_temp = concat_ident(template_name, "_temp");
        if self.debug {
            quote! {
                let #template_temp = &#template_name;
                println!("{:?}", #template_temp);
            }
        } else {
            quote! {}
        }
    }
}

mod kw {
    syn::custom_keyword!(attributes);
    syn::custom_keyword!(debug);
    syn::custom_keyword!(expected);
    syn::custom_keyword!(render_root);
    syn::custom_keyword!(template_group);
    syn::custom_keyword!(test_name);
}

fn parse_attribute_pair(input: ParseStream) -> syn::Result<(LitStr, Expr)> {
    let key: LitStr = input.parse()?;
    input.parse::<Token![:]>()?;
    let value: Expr = input.parse()?;
    Ok((key, value))
}

impl Parse for Test {
    fn parse(input: &ParseBuffer<'_>) -> syn::Result<Self> {
        input.parse::<kw::test_name>()?;
        input.parse::<Token![:]>()?;
        let test_name = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<kw::render_root>()?;
        input.parse::<Token![:]>()?;
        let render_root = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<kw::template_group>()?;
        input.parse::<Token![:]>()?;
        let content;
        let template_group_brace = braced!(content in input);
        let template_group = content.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<kw::attributes>()?;
        input.parse::<Token![:]>()?;
        let content;
        let attributes_brace = braced!(content in input);
        let attributes = content.parse_terminated(parse_attribute_pair)?;
        input.parse::<Token![,]>()?;

        input.parse::<kw::expected>()?;
        input.parse::<Token![:]>()?;
        let expected_value = input.parse()?;
        input.parse::<Token![,]>().ok();

        let mut debug = false;
        if let Ok(_) = input.parse::<kw::debug>() {
            input.parse::<Token![:]>()?;
            debug = input.parse::<LitBool>()?.value;
            input.parse::<Token![,]>().ok();
        }

        let test = Test {
            test_name,
            render_root,
            template_group,
            template_group_brace,
            attributes,
            attributes_brace,
            expected_value,
            debug,
        };
        Ok(test)
    }
}

fn concat_ident(ident: &Ident, suffix: impl AsRef<str>) -> Ident {
    Ident::new(&format!("{}{}", ident, suffix.as_ref()), ident.span())
}

fn quote_attributes(
    render_root: &Ident,
    attributes: &Punctuated<(LitStr, Expr), Token![,]>,
) -> Vec<proc_macro2::TokenStream> {
    attributes
        .iter()
        .map(|(attr_name, attr_value)| quote! { #render_root.add_expect(#attr_name, #attr_value); })
        .collect()
}

impl ToTokens for Test {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let static_template_name = concat_ident(&self.test_name, "_stemplate");
        let dynamic_template_name = concat_ident(&self.test_name, "_dtemplate");
        let template_group = StaticGroup::with_group(
            Visibility::Inherited,
            static_template_name.clone(),
            self.template_group.clone(),
        );
        let static_test_name = concat_ident(&self.test_name, "_static");
        let dynamic_test_name = concat_ident(&self.test_name, "_dynamic");
        let render_root = &self.render_root;

        let all_attributes = quote_attributes(&render_root, &self.attributes);
        let attributes1 = quote_spanned! { self.attributes_brace.span => #( #all_attributes )* };
        let attributes2 = attributes1.clone();
        let dynamic_template = template_group.as_dynamic_template();
        let expected = &self.expected_value;

        // let static_debug = self.make_debug_print(&static_template_name);
        let dynamic_debug = self.make_debug_print(&dynamic_template_name);
        let dynamic_template_let = quote_spanned! {
            self.template_group_brace.span =>
            let #dynamic_template_name = #dynamic_template;
        };
        let expanded = quote! {
            #template_group

            #[test]
            fn #static_test_name() {
                use ::string_template_test::TemplateTestExt as _;
                let mut #render_root = #static_template_name.#render_root();
                #attributes1
                assert_eq!(#expected, #render_root.render());
            }

            #[test]
            fn #dynamic_test_name() {
                use ::string_template_test::TemplateTestExt as _;
                #dynamic_template_let
                #dynamic_debug
                let mut #render_root = ::string_template_test::get_template(&#dynamic_template_name,
                                                    stringify!( #render_root ));
                #attributes2
                assert_eq!(#expected, #render_root.render());
            }
        };
        tokens.extend(expanded);
    }
}
