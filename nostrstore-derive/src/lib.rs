use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(AppendOnlyStream)]
pub fn nostrstore_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        
        impl nostrstore::Operation for #name
        where
            #name: Clone + serde::Serialize + serde::de::DeserializeOwned
        {
            type Value = Vec<#name>;

            fn default() -> Self::Value {
                Vec::new()
            }

            fn deserialize(value: String) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(serde_json::from_str(&value)?)
            }

            fn serialize(&self) -> Result<String, Box<dyn std::error::Error>> {
                Ok(serde_json::to_string(&self)?)
            }

            fn apply(&self, mut value: Self::Value) -> Self::Value {
                value.push(self.clone());
                value
            }
        }
    };

    TokenStream::from(expanded)
}
