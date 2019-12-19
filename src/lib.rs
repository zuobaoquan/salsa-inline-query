extern crate proc_macro;

use proc_macro::TokenStream;
use proc_quote::quote;
use syn::{fold::Fold, *};

/// Transform inline query definitions to free query functions
struct Rewriter {
    trait_name: Ident,
    query_functions: Vec<ItemFn>,
}

impl Fold for Rewriter {
    fn fold_trait_item_method(&mut self, mut node: TraitItemMethod) -> TraitItemMethod {
        if let Some(block) = node.default.take() {
            let mut rename = RenameSelf {
                trait_name: self.trait_name.clone(),
            };
            let func = ItemFn {
                attrs: Vec::new(),
                vis: Visibility::Inherited,
                sig: rename.fold_signature(node.sig.clone()),
                block: Box::new(rename.fold_block(block)),
            };
            self.query_functions.push(func);
        }
        node
    }
}

/// Rename `self` arg in method signature and all idents named `self` in block
struct RenameSelf {
    trait_name: Ident,
}

const DB_ARG_NAME: &str = "__salsa_db";

impl Fold for RenameSelf {
    fn fold_fn_arg(&mut self, node: FnArg) -> FnArg {
        if let FnArg::Receiver(node) = node {
            let db = Ident::new(DB_ARG_NAME, node.self_token.span);
            let ident = &self.trait_name;
            let db_arg: FnArg = parse_quote! {
                #db: &impl #ident
            };
            db_arg
        } else {
            node
        }
    }

    fn fold_ident(&mut self, node: Ident) -> Ident {
        if node == "self" {
            Ident::new(DB_ARG_NAME, node.span())
        } else {
            node
        }
    }
}

/// Supports inline query definition within salsa query group traits
/// # Example
/// By applying `salsa_inline_query` attribute to a query group trait
///  ```
///  ##[salsa_inline_query]
///  ##[salsa::query_group(SourceDatabaseStorage)]
///  trait SourceDatabase: std::fmt::Debug {
///      ##[salsa::input]
///      fn source_text(&self, file_id: u32) -> Arc<String>;
///
///      fn source_len(&self, file_id: u32) -> usize {
///          let text = self.source_text(file_id);
///          text.len()
///      }
///  }
/// ```
///
/// The above code will be transformed to:
///
/// ```
///  ##[salsa::query_group(SourceDatabaseStorage)]
///  trait SourceDatabase: std::fmt::Debug {
///      ##[salsa::input]
///      fn source_text(&self, file_id: u32) -> Arc<String>;
///
///      fn source_len(&self, file_id: u32) -> usize;
///  }
///
///  fn source_len(__salsa_db: &impl SourceDatabase, file_id: u32) -> usize {
///      let text = __salsa_db.source_text(file_id);
///      text.len()
///  }
/// ```
#[proc_macro_attribute]
pub fn salsa_inline_query(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let query_group_trait = parse_macro_input!(input as syn::ItemTrait);
    let mut rewriter = Rewriter {
        trait_name: query_group_trait.ident.clone(),
        query_functions: Vec::new(),
    };
    let query_group_trait = rewriter.fold_item_trait(query_group_trait);
    let query_functions = rewriter.query_functions.iter();
    let result = quote! {
        #query_group_trait

        #(#query_functions)*
    };
    result.into()
}
