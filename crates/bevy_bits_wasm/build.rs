use core::fmt::Write;
use std::path::Path;
use std::{env, fs};

use cargo_metadata::MetadataCommand;
use proc_macro2::{Spacing, TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::parse_file;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if !(target_arch == "wasm32" && target_os == "unknown") {
        return;
    }

    // Get cargo metadata
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to get cargo metadata");

    // Find the serde package
    let ribbit_bits_package = metadata
        .packages
        .iter()
        .find(|p| p.name == "ribbit_bits")
        .expect("Failed to find serde package");

    // The manifest_path will give you the path to the Cargo.toml of the dependency
    // The parent of this path is the actual source directory
    let bit_name_path = ribbit_bits_package
        .manifest_path
        .parent()
        .expect("Failed to get parent directory")
        .join("src/bit_name.rs");

    // Read the contents of the bit_name.rs file
    let bit_name_content = fs::read_to_string(bit_name_path).expect("Failed to read bit_name.rs");

    // Parse the file content
    let ast = parse_file(&bit_name_content).expect("Failed to parse bit_name.rs");

    // Find the BitName enum
    let bit_name_enum = ast
        .items
        .iter()
        .find_map(|item| {
            if let syn::Item::Enum(item_enum) = item {
                if item_enum.ident == "BitName" {
                    Some(item_enum)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("Failed to find BitName enum");

    // Generate the match arms
    let match_arms = bit_name_enum.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let module_name = format_ident!(
            "{}",
            heck::AsSnakeCase(variant_ident.to_string()).to_string()
        );
        quote! {
            BitName::#variant_ident => #module_name::run(),
        }
    });

    // Generate the full implementation
    let generated_code = quote! {
        impl BitRunner {
            fn run(&self) {
                match self.bit_name {
                    #(#match_arms)*
                }
            }
        }
    };

    let output = token_stream_to_string(&generated_code);

    let out_dir = env::var_os("OUT_DIR").expect("There should be an OUT_DIR");

    // Write the generated code to a file
    let dest_path = Path::new(&out_dir).join("bit_runner_impl.rs");

    fs::write(&dest_path, output).expect("Failed to write generated code");

    // Format the output
    std::process::Command::new("rustfmt")
        .arg(&dest_path)
        .output()
        .expect("failed to execute rustfmt");

    println!("cargo:rerun-if-changed=../../Cargo.lock");
}

fn token_stream_to_string(ts: &TokenStream) -> String {
    let mut result = String::new();
    let mut iter = ts.clone().into_iter().peekable();

    while let Some(token_tree) = iter.next() {
        match token_tree {
            TokenTree::Group(group) => {
                let delimiter = match group.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => ("(", ")"),
                    proc_macro2::Delimiter::Brace => ("{", "}"),
                    proc_macro2::Delimiter::Bracket => ("[", "]"),
                    proc_macro2::Delimiter::None => ("", ""),
                };
                write!(
                    &mut result,
                    "{}{}{}",
                    delimiter.0,
                    token_stream_to_string(&group.stream()),
                    delimiter.1
                )
                .expect("Failed to write to string");
            }
            TokenTree::Ident(ident) => {
                result.push_str(&ident.to_string());
                if let Some(TokenTree::Punct(punct)) = iter.peek() {
                    if punct.as_char() == ':' || punct.as_char() == '.' {
                        // Don't add space before ':' or '.'
                    } else {
                        result.push(' ');
                    }
                } else {
                    result.push(' ');
                }
            }
            TokenTree::Punct(punct) => {
                result.push(punct.as_char());
                if punct.spacing() == Spacing::Alone {
                    result.push(' ');
                }
            }
            TokenTree::Literal(lit) => {
                result.push_str(&lit.to_string());
                result.push(' ');
            }
        }
    }

    result.trim().to_string()
}
