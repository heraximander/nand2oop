use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{punctuated::Punctuated, spanned::Spanned, token::{Comma, Semi}, Ident, ItemFn, LitInt, LitStr};

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
    let struct_inputs_name_family = Ident::new(&format!("{}Family", struct_inputs_name_str), ast.span());

    enum ArgType {
        Input,
        InputArray(LitInt),
    }

    let input_name_to_type = ast.sig.inputs.iter().skip(1).map(|farg| match farg {
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
            ArgType::Input => quote! { #arg_name: TElem },
            ArgType::InputArray(len) => {
                quote! { #arg_name: [TElem;#len] }
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
    let (inputs_from_flat_mapping_vec, _) = input_name_to_type
        .iter()
        .fold((vec![],0), |(mut acc,curr_i),(arg_name, arg_type)| {            
            let start_ident = LitInt::new(&curr_i.to_string(), ast.span());

            let new_i = match arg_type {
                ArgType::Input => {
                    acc.push(quote! {#arg_name: input[#start_ident]});
                    curr_i + 1
                },
                ArgType::InputArray(len) => {
                    let new_i = curr_i + len.to_string().parse::<i32>().unwrap();
                    let end_ident = LitInt::new(&new_i.to_string(), ast.span());
                    acc.push(quote! {#arg_name: input[#start_ident..#end_ident].try_into().unwrap()});
                    new_i
                },
            };
            
            (acc, new_i)
        });
    let inputs_from_flat_mapping = inputs_from_flat_mapping_vec
        .iter()
        .collect::<Punctuated<_, Comma>>();
    let (inputs_to_flat_mapping_vec, _) = input_name_to_type
        .iter()
        .fold((vec![],0), |(mut acc,curr_i),(arg_name, arg_type)| {            
            let new_i = match arg_type {
                ArgType::Input => {
                    let start_ident = LitInt::new(&curr_i.to_string(), ast.span());
                    acc.push(quote! {input[#start_ident] = Option::Some(self.#arg_name)});
                    curr_i + 1
                },
                ArgType::InputArray(len) => {
                    let new_i = curr_i + len.to_string().parse::<i32>().unwrap();
                    for i in curr_i..new_i {
                        let curr_ident = LitInt::new(&i.to_string(), ast.span());
                        let index_ident = LitInt::new(&(i-curr_i).to_string(), ast.span());
                        acc.push(quote! {input[#curr_ident] = Option::Some(self.#arg_name[#index_ident])});
                    }
                    new_i
                },
            };
            
            (acc, new_i)
        });
    let inputs_to_flat_mapping = inputs_to_flat_mapping_vec
        .iter()
        .collect::<Punctuated<_, Semi>>();

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

        struct #struct_inputs_name<TElem> {
            #inputs
        }
        
        struct #struct_inputs_name_family;
        impl hdl::StructuredInputFamily<#arity> for #struct_inputs_name_family {
            type StructuredInput<T: Copy> = #struct_inputs_name<T>;
        }

        #ast
        impl<'a> #struct_name<'a> {
            fn new(alloc: &'a bumpalo::Bump, inputs: #struct_inputs_name<Input<'a>>) -> &'a #struct_name<'a> {
                let inner = #ident(alloc,#mapped_chip_inputs);
                let chipout = inner.map(|in_| ChipOutput::new(alloc, in_));
                static COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
                alloc.alloc(#struct_name{
                    out: chipout,
                    identifier: COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
                })
            }
        }
        
        impl<'a, TElem: Copy> hdl::StructuredInput<TElem, #arity> for #struct_inputs_name<TElem> {
            fn map_from_flat(input: [TElem; #arity]) -> Self {
                #struct_inputs_name {
                    #inputs_from_flat_mapping
                }
            }

            fn to_flat(&self) -> [TElem; #arity] {
                let mut input = [Option::<TElem>::None; #arity];
                #inputs_to_flat_mapping;
                input.map(|x| x.expect("all elements should be mapped"))
            }
        }

        impl<'a> hdl::SizedChip<'a, #struct_inputs_name_family, #return_size_literal> for #struct_name<'a> {
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
                alloc.alloc(hdl::SizedChip::<#struct_inputs_name_family,#return_size_literal>::get_out(self,alloc))
            }
        }

    };
    gen.into()
}
