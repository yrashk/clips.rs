#![recursion_limit="128"]

extern crate clips;
extern crate syn;
#[macro_use] extern crate darling;
#[macro_use] extern crate quote;

extern crate proc_macro;
use proc_macro::TokenStream;

use syn::Ident;
use darling::{ast, FromDeriveInput};
use quote::{ToTokens, Tokens};

use std::ops::Deref;

#[derive(FromDeriveInput)]
#[darling(attributes(clips), forward_attrs(allow, doc, cfg), supports(struct_any))]
struct FactReceiver {
    ident: Ident,
    generics: syn::Generics,
    vis: syn::Visibility,
    body: ast::Body<(), SlotReceiver>,
    #[darling(default)]
    slots_trait_name: Option<Ident>,
    #[darling(default)]
    asserted_type_name: Option<Ident>,
    template: String,
    #[darling(default)]
    consume_on_assert: bool,
    #[darling(default)]
    non_recoverable: bool,
}

impl FactReceiver {
    fn slots_trait_name(&self) -> Ident {
        self.slots_trait_name.clone().unwrap_or(Ident::from(String::from(self.ident.as_ref()) + "Slots"))
    }
    fn asserted_type_name(&self) -> Ident {
        self.asserted_type_name.clone().unwrap_or(Ident::from(String::from("Asserted") + self.ident.as_ref()))
    }
}

#[derive(FromMetaItem, Debug, Clone, Copy)]
enum ReturnType {
    Default,
    Ref,
    Copy,
    Clone,
}

impl ReturnType {
    fn choose_if_default(&self, ty: &syn::Ty) -> ReturnType {
         match self {
            &ReturnType::Default => {
                // primitive types
                if ty == &syn::parse_type("bool").unwrap() ||
                   ty == &syn::parse_type("i8").unwrap()   ||
                   ty == &syn::parse_type("i16").unwrap()  ||
                   ty == &syn::parse_type("i32").unwrap()  ||
                   ty == &syn::parse_type("i64").unwrap()  ||
                   ty == &syn::parse_type("u8").unwrap()   ||
                   ty == &syn::parse_type("u16").unwrap()  ||
                   ty == &syn::parse_type("u32").unwrap()  ||
                   ty == &syn::parse_type("u64").unwrap()  ||
                   ty == &syn::parse_type("f32").unwrap()  ||
                   ty == &syn::parse_type("f64").unwrap()  {
                    return ReturnType::Copy
                }
                // reference
                if let &syn::Ty::Rptr(_, _) = ty {
                    return ReturnType::Copy
                }
                ReturnType::Ref
            },
            v => v.clone(),
        }
    }
    fn to_ty(&self, ty: syn::Ty) -> syn::Ty {
        match self {
            &ReturnType::Default => self.choose_if_default(&ty).to_ty(ty),
            &ReturnType::Ref =>
                if ty == syn::parse_type("String").unwrap() {
                    syn::Ty::Rptr(None, Box::new(syn::MutTy {
                        ty: syn::parse_type("str").unwrap(),
                        mutability: syn::Mutability::Immutable
                    }))
                } else {
                    syn::Ty::Rptr(None, Box::new(syn::MutTy {
                        ty,
                        mutability: syn::Mutability::Immutable,
                    }))
                },
            _ => ty,
        }
    }
}

impl Default for ReturnType {
   fn default() -> Self {
       ReturnType::Default
   }
}

#[derive(Debug, FromField)]
#[darling(attributes(clips))]
struct SlotReceiver {
    ident: Option<Ident>,
    ty: syn::Ty,
    #[darling(default)]
    rename: Option<String>,
    #[darling(default)]
    return_type: ReturnType,
}

impl SlotReceiver {
    fn slot_name(&self) -> String {
        self.rename.clone().unwrap_or( String::from(self.ident.clone().unwrap().as_ref()))
    }
    fn return_ty(&self) -> syn::Ty {
        self.return_type.to_ty(self.ty.clone())
    }
    fn return_type(&self) -> ReturnType {
        self.return_type.choose_if_default(&self.ty)
    }
}

/// Slots trait definition
struct SlotsTrait<'a>(&'a FactReceiver);

impl<'a> Deref for SlotsTrait<'a> {
    type Target = FactReceiver;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> ToTokens for SlotsTrait<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let vis = &self.vis;
        let name = self.slots_trait_name();
        let fields = self.body.as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;
        let mut slots_tokens = Tokens::new();
        for field in fields {
            let field_name = field.ident.clone().expect("fields should named");
            let field_ty = field.return_ty();
            slots_tokens.append(quote! {
               fn #field_name(&self) -> #field_ty;
            });
        }
        tokens.append(quote! {
           #vis trait #name {
             #slots_tokens
           }
        });
    }
}


/// Slots trait implementation for the struct
struct StructImpl<'a>(&'a FactReceiver);

impl<'a> Deref for StructImpl<'a> {
    type Target = FactReceiver;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> ToTokens for StructImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let (imp, ty, wher) = self.generics.split_for_impl();
        let name = self.slots_trait_name();
        let ident = &self.ident;
        let fields = self.body.as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;
        let mut slots_tokens = Tokens::new();
        for field in fields {
            let field_name = field.ident.clone().expect("fields should named");
            let field_ty = field.return_ty();
            let body = match field.return_type() {
                ReturnType::Ref | ReturnType::Default => quote!(&self.#field_name),
                ReturnType::Clone => quote!(self.#field_name.clone()),
                ReturnType::Copy => quote!(self.#field_name),
            };
            slots_tokens.append(quote! {
               fn #field_name(&self) -> #field_ty {
                  #body
               }
            });
        }
        tokens.append(quote! {
            impl #imp #name for #ident #ty #wher {
               #slots_tokens
            }
        });
    }
}

/// Slots trait implementation for the asserted fact
struct AssertedImpl<'a>(&'a FactReceiver);

impl<'a> Deref for AssertedImpl<'a> {
    type Target = FactReceiver;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> ToTokens for AssertedImpl<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let slots = self.slots_trait_name();
        let name = self.asserted_type_name();
        let ident = &self.ident;
        let fields = self.body.as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;
        let vis = &self.vis;
        let clips_crate = Ident::new(format!("__clips_for_{}", ident));
        let mut slots_tokens = Tokens::new();
        for field in fields.clone() {
            let field_name = field.ident.clone().expect("fields should named");
            let slot_name = field.slot_name();
            let field_ty = field.return_ty();
            slots_tokens.append(quote! {
               fn #field_name(&self) -> #field_ty {
                  (#clips_crate::ValueAccess::value(&self.slot(#slot_name)) as Option<#field_ty>).unwrap()
               }
            });
        }
        tokens.append(quote! {
                extern crate clips as #clips_crate;
                #vis struct #name<'a>(#clips_crate::Fact<'a>);
                impl<'a> ::std::ops::Deref for #name<'a> {
                   type Target = #clips_crate::Fact<'a>;
                   fn deref(&self) -> &Self::Target {
                      &self.0
                   }
                }
        });
        tokens.append(quote! {
            impl<'a> #slots for #name<'a> {
               #slots_tokens
            }
        });
        // implement Recoverable
        if !self.non_recoverable {
            let mut slot_tokens = Tokens::new();
            slot_tokens.append_separated(fields.iter().map(|field| {
                let field_name = field.ident.clone().expect("fields should named");
                let slot_name = field.slot_name();
                quote!(#field_name: #clips_crate::ValueAccess::value(&self.slot(#slot_name)).unwrap())
            }), ",");
            tokens.append(quote! {
             impl<'a> #clips_crate::fact::Recoverable for #name<'a> {
                type T = #ident;

                fn recover(self) -> Self::T {
                   #ident {
                      #slot_tokens
                   }
                }

             }
            });
        }
    }
}

/// Implementation of Assertable for the struct
struct Assertable<'a>(&'a FactReceiver);

impl<'a> Deref for Assertable<'a> {
    type Target = FactReceiver;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> ToTokens for Assertable<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        let mut generics = self.generics.clone();
        if !self.consume_on_assert {
            generics.lifetimes.insert(0, syn::LifetimeDef::new("'__clips_assertable"));
        }
        generics.lifetimes.insert(0, syn::LifetimeDef::new("'__clips_env"));
        let (imp, _, _) = generics.split_for_impl();
        let (_, ty, wher) = self.generics.split_for_impl();
        let name = self.asserted_type_name();
        let ident = &self.ident;
        let target_ident = if self.consume_on_assert {
            quote!()
        } else {
            quote!(&'__clips_assertable)
        };
        let fields = self.body.as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;
        let mut slots_tokens = Tokens::new();
        for field in fields {
            let field_name = field.ident.clone().expect("fields should named");
            let slot_name = field.slot_name();
            slots_tokens.append(quote! {
                fb.put(#slot_name, self.#field_name()).or_else(|_| Err(()))?;
            });
        }
        let dummy_const = Ident::new(format!("_IMPL_ASSERTABLE_FOR_{}", ident));
        let template = self.template.as_str();
        tokens.append(quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
             const #dummy_const: () = {
                extern crate clips as _clips;
                impl #imp _clips::fact::Assertable<'__clips_env> for #target_ident #ident #ty #wher {
                   type T = #name<'__clips_env>;
                   type Error = ();
                   fn assert(self, env: &'__clips_env _clips::Environment) -> Result<Self::T, Self::Error> {
                      let fb = env.new_fact_builder(#template);
                      #slots_tokens
                      fb.assert().and_then(|f| Ok(#name(f))).or(Err(()))
                   }
                }
             };
        });
    }
}


#[proc_macro_derive(clips_fact, attributes(clips))]
pub fn derive_instruments(input: TokenStream) -> TokenStream {
    let input = syn::parse_derive_input(&input.to_string()).unwrap();
    let rcvr = FactReceiver::from_derive_input(&input).unwrap();

    let slot_trait = SlotsTrait(&rcvr);
    let struct_impl = StructImpl(&rcvr);
    let asserted_impl = AssertedImpl(&rcvr);
    let assertable = Assertable(&rcvr);

    let tokens = quote!( #slot_trait #struct_impl #asserted_impl #assertable);

    tokens.parse().unwrap()
}


