use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};


#[proc_macro_attribute]
pub fn without_gvl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let mut anon_sig = sig.clone();
    anon_sig.ident = syn::Ident::new("__anon_wrapper", sig.ident.span());

    let params = sig.inputs.iter().map(|arg| {
        if let syn::FnArg::Typed(pat) = arg {
            let arg = &pat.pat;
            let ty = &pat.ty;
            quote!(#arg, #ty)
        } else {
            panic!("expected typed argument")
        }
    });

    let return_ty = match &sig.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    };

    let block = &input.block;

    quote!(
        #(#attrs)*
        #vis #sig {
            #anon_sig {
                #block
            }
            lucchetto::call_without_gvl!(__anon_wrapper, args: (#(#params),*), return_type: #return_ty)
        }
    ).into()
}
