use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, ItemFn, LitInt, LitStr};

const CHIP_FN_TYPE_ERR: &str =
    "chip function must return type [ChipOutputInner;n] where n is a literal greater than 0";
const CHIP_ARG_TYPE_ERR: &str = "chip function must take arguments of &Bump,[Input<'_>;n] where _n_ is a literal greater than 0";

#[proc_macro_attribute]
pub fn chip(_: TokenStream, item: TokenStream) -> TokenStream {
    let ast: ItemFn = syn::parse(item).unwrap();
    let ident = &ast.sig.ident;
    let name = ident.to_string();
    let struct_name_str = &(name
        .chars()
        .take(1)
        .next()
        .unwrap()
        .to_uppercase()
        .to_string()
        + &name[1..]);
    let struct_name = Ident::new(struct_name_str, ast.sig.ident.span());

    assert_eq!(ast.sig.inputs.len(), 2, "{}", CHIP_ARG_TYPE_ERR);
    let arity_lit = match ast.sig.inputs[1].clone() {
        syn::FnArg::Receiver(_) => panic!("{}", CHIP_ARG_TYPE_ERR),
        syn::FnArg::Typed(pat) => match *(pat.ty) {
            syn::Type::Array(tya) => match tya.len {
                syn::Expr::Lit(x) => match x.lit {
                    syn::Lit::Int(i) => i,
                    _ => panic!("{}", CHIP_ARG_TYPE_ERR),
                },
                _ => panic!("{}", CHIP_ARG_TYPE_ERR),
            },
            _ => panic!("{}", CHIP_ARG_TYPE_ERR),
        },
    };

    let return_size: usize = match ast.sig.output {
        syn::ReturnType::Default => panic!("{}", CHIP_FN_TYPE_ERR),
        syn::ReturnType::Type(_, ref ty) => match *(ty.clone()) {
            syn::Type::Array(tya) => match tya.len {
                syn::Expr::Lit(x) => match x.lit {
                    syn::Lit::Int(i) => str::parse(&i.to_string()).expect(CHIP_FN_TYPE_ERR),
                    _ => panic!("{}", CHIP_FN_TYPE_ERR),
                },
                _ => panic!("{}", CHIP_FN_TYPE_ERR),
            },
            _ => panic!("{}", CHIP_FN_TYPE_ERR),
        },
    };
    let return_size_literal = LitInt::new(&return_size.to_string(), Span::call_site());
    let lit_name = LitStr::new(struct_name_str, Span::call_site());
    let lit_id = LitStr::new(&format!("{}{{}}", struct_name_str), Span::call_site());

    let gen = quote! {
        struct #struct_name<'a> {
            out: [&'a hdl::ChipOutput<'a>; #return_size_literal],
            identifier: u32
        }

        #ast
        impl<'a> #struct_name<'a> {
            fn new(alloc: &'a bumpalo::Bump, inputs: [hdl::Input<'a>;#arity_lit]) -> &'a #struct_name<'a> {
                let chipinputs = inputs.map(|in_| ChipInput::new(&alloc, in_));
                let inner = #ident(alloc,chipinputs);
                let chipout = inner.map(|in_| ChipOutput::new(alloc, in_));
                static COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
                alloc.alloc(#struct_name{
                    out: chipout,
                    identifier: COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
                })
            }

        }

        impl<'a> hdl::SizedChip<'a, #return_size_literal> for #struct_name<'a> {
            // TODO: probably don't need to allocate this in the arena
            // can instead just return the struct rather than a pointer
            fn get_out(&'a self, alloc: &'a Bump) -> [&'a hdl::ChipOutputWrapper; #return_size_literal] {
                self.out.map(|out| hdl::ChipOutputWrapper::new(alloc, out, self))
            }
        }

        impl<'a> hdl::Chip<'a> for #struct_name<'a> {
            fn get_id(&self) -> String {
                format!(#lit_id, self.identifier)
            }

            fn get_label(&self) -> &'static str {
                #lit_name
            }

            fn get_out_unsized(&'a self, alloc: &'a Bump) -> &'a[&hdl::ChipOutputWrapper] {
                alloc.alloc(hdl::SizedChip::<#return_size_literal>::get_out(self,alloc))
            }
        }
    };
    gen.into()
}
