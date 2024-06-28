use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Colon2, Comma, Semi},
    GenericParam, Ident, ItemFn, Lifetime, LifetimeDef, LitInt, LitStr, PathArguments,
};

const CHIP_FN_TYPE_ERR: &str =
    "chip function must return type [ChipOutputInner;n] where n is a literal greater than 0";
const CHIP_ARG_TYPE_ERR: &str = "chip function must take arguments of &Bump,{Input<'_>|[Input<'_>; N]}* where _n_ is a literal greater than 0";

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

    assert!(ast.sig.inputs.len() > 1, "{}", CHIP_ARG_TYPE_ERR);
    let struct_inputs_name_str = format!("{}Inputs", struct_name_str);
    let struct_inputs_name = Ident::new(&struct_inputs_name_str, ast.sig.ident.span());
    let struct_inputs_name_family =
        Ident::new(&format!("{}Family", struct_inputs_name_str), ast.span());

    enum ArgType {
        Input,
        InputArray(LitInt),
    }

    let input_name_to_type = ast
        .sig
        .inputs
        .iter()
        .skip(1)
        .map(|farg| match farg {
            syn::FnArg::Receiver(_) => panic!("{}", CHIP_ARG_TYPE_ERR),
            syn::FnArg::Typed(pat) => {
                let arg_name = pat.pat.clone();
                let arg_type = match *(pat.ty.clone()) {
                    syn::Type::Array(tya) => {
                        match tya.len {
                            syn::Expr::Lit(x) => match x.lit {
                                // unwrap should be safe because we already know it's a literal
                                syn::Lit::Int(i) => ArgType::InputArray(i),
                                _ => panic!("{}", CHIP_ARG_TYPE_ERR),
                            },
                            _ => panic!("{}", CHIP_ARG_TYPE_ERR),
                        }
                    }
                    syn::Type::Reference(_) => ArgType::Input,
                    _ => panic!("{}", CHIP_ARG_TYPE_ERR),
                };
                (arg_name, arg_type)
            }
        })
        .collect::<Vec<_>>();

    let mapped_chip_inputs = input_name_to_type
        .iter()
        .map(|(arg_name, arg_type)| match arg_type {
            ArgType::Input => quote! { ChipInput::new(&alloc, inputs.#arg_name ) },
            ArgType::InputArray(_) => {
                quote! { inputs.#arg_name.map(|x| ChipInput::new(&alloc, x )) }
            }
        })
        .collect::<Punctuated<_, Comma>>();
    let inputs = input_name_to_type
        .iter()
        .map(|(arg_name, arg_type)| match arg_type {
            ArgType::Input => quote! { #arg_name: T },
            ArgType::InputArray(len) => {
                quote! { #arg_name: [T;#len] }
            }
        })
        .collect::<Punctuated<_, Comma>>();

    let arity_num = input_name_to_type
        .iter()
        .map(|(_, arg_type)| match arg_type {
            ArgType::Input => 1,
            ArgType::InputArray(litint) => litint.to_string().parse().unwrap(),
        })
        .sum::<usize>();
    let arity = LitInt::new(&arity_num.to_string(), ast.span());
    let lit_name = LitStr::new(struct_name_str, Span::call_site());
    let lit_id = LitStr::new(&format!("{}{{}}", struct_name_str), Span::call_site());

    let struct_outputs_type = match ast.sig.output {
        syn::ReturnType::Default => panic!("{}", CHIP_FN_TYPE_ERR),
        syn::ReturnType::Type(_, ref ty) => match *ty.clone() {
            syn::Type::Path(p) => p
                .path
                .segments
                .into_iter()
                .map(|mut seg| {
                    seg.arguments = PathArguments::None;
                    seg
                })
                .collect::<Punctuated<_, Colon2>>(),
            _ => panic!("{}", CHIP_ARG_TYPE_ERR),
        },
    };

    let gen = quote! {
        struct #struct_name<'a> {
            out: [&'a hdl::ChipOutput<'a>; {#struct_outputs_type::<bool/* type doesn't matter */>::get_arity()}],
            identifier: u32
        }

        #[derive(StructuredData)]
        struct #struct_inputs_name<T> {
            #inputs
        }

        struct #struct_inputs_name_family;
        impl hdl::StructuredDataFamily<#arity, {#struct_outputs_type::<bool/* type doesn't matter */>::get_arity()}> for #struct_inputs_name_family {
            type StructuredInput<T> = #struct_inputs_name<T>;
            type StructuredOutput<T> = #struct_outputs_type<T>;
        }

        #ast
        impl<'a> #struct_name<'a> {
            fn new(alloc: &'a bumpalo::Bump, inputs: #struct_inputs_name<Input<'a>>) -> &'a #struct_name<'a> {
                let inner = #ident(alloc,#mapped_chip_inputs);
                let chipout = hdl::StructuredData::to_flat(inner).map(|in_| ChipOutput::new(alloc, in_));
                static COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
                alloc.alloc(#struct_name{
                    out: chipout,
                    identifier: COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
                })
            }
        }

        impl<'a> hdl::SizedChip<'a, #struct_inputs_name_family, {#struct_outputs_type::<bool/* type doesn't matter */>::get_arity()}, #arity> for #struct_name<'a> {
            // TODO: probably don't need to allocate this in the arena
            // can instead just return the struct rather than a pointer
            fn get_out(&'a self, alloc: &'a Bump) -> #struct_outputs_type<&'a hdl::ChipOutputWrapper> {
                let flat_out = self.out.map(|out| hdl::ChipOutputWrapper::new(alloc, out, self));
                hdl::StructuredData::from_flat(flat_out)
            }
        }

        impl<'a> hdl::Chip<'a> for #struct_name<'a> {
            fn get_id(&self) -> String {
                format!(#lit_id, self.identifier)
            }

            fn get_label(&self) -> &'static str {
                #lit_name
            }
        }

    };
    gen.into()
}

const STRUCT_DERIVE_ERROR_MSG: &str = "can only derive StructuredData on a struct";

#[proc_macro_derive(StructuredData)]
pub fn chip_output_collection_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let generics = &ast.generics;

    let mut structured_data_generics = generics.clone();
    structured_data_generics
        .params
        .extend(vec![GenericParam::Lifetime(LifetimeDef::new(
            Lifetime::new("'a", ast.span()),
        ))]);

    let fields = match ast.data {
        syn::Data::Struct(ref s) => match &s.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => panic!("{}", STRUCT_DERIVE_ERROR_MSG),
        },
        _ => panic!("{}", STRUCT_DERIVE_ERROR_MSG),
    };
    let (from_flat_mapping, _) = fields.iter().fold((vec![], 0), |(mut fieldlist, i), f| {
        let fieldname = f
            .ident
            .clone()
            .expect("field must have a name for a non-tuple struct");

        let new_i = match &f.ty {
            syn::Type::Array(ty) => {
                let arraylen: usize = match &ty.len {
                    syn::Expr::Lit(lit) => match &lit.lit {
                        syn::Lit::Int(int) => int.to_string().parse().unwrap(),
                        _ => panic!("shouldn't get here"),
                    },
                    _ => panic!("{}", STRUCT_DERIVE_ERROR_MSG),
                };

                let i_subset = (i..arraylen + i)
                    .map(|x| Ident::new(&format!("in{x}"), Span::call_site()))
                    .collect::<Punctuated<_, Comma>>();
                fieldlist.push(quote! {
                    #fieldname: [#i_subset]
                });
                i + arraylen
            }
            syn::Type::Path(_) => {
                let curr_ident = Ident::new(&format!("in{i}"), Span::call_site());
                fieldlist.push(quote! {
                    #fieldname: #curr_ident
                });
                i + 1
            }
            _ => panic!("{}", STRUCT_DERIVE_ERROR_MSG),
        };
        (fieldlist, new_i)
    });
    let inputs_from_flat_mapping = from_flat_mapping.iter().collect::<Punctuated<_, Comma>>();
    let (destructured_inputs, _) = fields.iter().fold((vec![], 0), |(mut acc, i), elem| {
        let new_i = match &elem.ty {
            syn::Type::Array(ty) => {
                let arraylen: usize = match &ty.len {
                    syn::Expr::Lit(lit) => match &lit.lit {
                        syn::Lit::Int(int) => int.to_string().parse().unwrap(),
                        _ => panic!("shouldn't get here"),
                    },
                    _ => panic!("{}", STRUCT_DERIVE_ERROR_MSG),
                };

                for new_i in i..i + arraylen {
                    acc.push(Ident::new(&format!("in{}", new_i), Span::call_site()));
                }
                i + arraylen
            }
            syn::Type::Path(_) => {
                acc.push(Ident::new(&format!("in{i}"), Span::call_site()));
                i + 1
            }
            _ => panic!("{}", STRUCT_DERIVE_ERROR_MSG),
        };
        (acc, new_i)
    });
    let destructured_inputs = destructured_inputs.iter().collect::<Punctuated<_, Comma>>();
    let (destructing_var_names, numvars) = fields.iter().fold((vec![], 0), |(mut acc, i), elem| {
        let fieldname = elem
            .ident
            .clone()
            .expect("field must have a name for a non-tuple struct");
        let new_i = match &elem.ty {
            syn::Type::Array(ty) => {
                let arraylen: usize = match &ty.len {
                    syn::Expr::Lit(lit) => match &lit.lit {
                        syn::Lit::Int(int) => int.to_string().parse().unwrap(),
                        _ => panic!("shouldn't get here"),
                    },
                    _ => panic!("{}", STRUCT_DERIVE_ERROR_MSG),
                };
                let destructured_var_names = (i..i + arraylen)
                    .map(|elem| Ident::new(&format!("o{}", elem), Span::call_site()))
                    .collect::<Punctuated<_, Comma>>();
                acc.push(quote! {
                    let [#destructured_var_names] = self.#fieldname
                });
                i + arraylen
            }
            syn::Type::Path(_) => {
                let destructured_var_name = Ident::new(&format!("o{}", i), Span::call_site());
                acc.push(quote! {
                    let #destructured_var_name = self.#fieldname
                });
                i + 1
            }
            _ => panic!("{}", STRUCT_DERIVE_ERROR_MSG),
        };
        (acc, new_i)
    });
    let destructing_var_names = destructing_var_names
        .iter()
        .collect::<Punctuated<_, Semi>>();
    let destructured_fields = (0..numvars)
        .map(|fi| Ident::new(&format!("o{}", fi), Span::call_site()))
        .collect::<Punctuated<_, Comma>>();
    let arity = LitInt::new(&numvars.to_string(), ast.span());

    quote! {
        impl #structured_data_generics hdl::StructuredData<T, #arity> for #name #generics {
            fn from_flat(input: [T; #arity]) -> Self { // FIXME: don't make this dependent on generic name
            let [#destructured_inputs] = input;
                #name {
                    #inputs_from_flat_mapping
                }
            }

            fn to_flat(self) -> [T; #arity] {
                #destructing_var_names;
                [#destructured_fields]
            }
        }

        impl #generics #name #generics {
            const fn get_arity() -> usize {
                #arity
            }
        }
    }
    .into()
}
