extern crate libbindgen;
extern crate pkg_config;
extern crate regex;

use std::env;
use std::io::{Read, Write};
use std::fs;
use std::path;
use regex::Regex;

const LIBS: [&'static str; 3] = [
    "Chakra.Pal",
    "Chakra.Common.Core",
    "Chakra.Jsrt"
];

fn main() {
    chakra_linking();
    chakra_bindings();
}

fn chakra_linking() {
    // TODO: How should 'embed-icu' be handled?
    if let Some(dir_str) = env::var_os("CHAKRA_BUILD") {
        let build = path::Path::new(&dir_str);

        // Standard paths in `ChakraCore` build
        maybe_search(build);
        maybe_search(build.join("pal/src"));
        maybe_search(build.join("lib/Jsrt"));
        maybe_search(build.join("lib/Common/Core"));
    } else {
        println!("cargo:warning=No $CHAKRA_BUILD specified");

        // There is no information available, blindly add paths
        maybe_search("/usr/lib");
        maybe_search("/usr/local/lib");
    }

    link_libraries();
}

/// Adds a library search path if it exists.
fn maybe_search<P>(dir: P) where P: AsRef<path::Path> {
    let dir = dir.as_ref();
    if fs::metadata(dir).map(|m| m.is_dir()).unwrap_or(false) {
        println!("cargo:rustc-link-search=native={}", dir.to_string_lossy());
    }
}

fn link_libraries() {
    let target = env::var("TARGET").unwrap();

    if cfg!(feature = "shared") {
        // The dynamic library is completely self-contained
        println!("cargo:rustc-link-lib=dylib=ChakraCore");
    } else {
        for lib in LIBS.iter() {
            // Statically link all ChakraCore libraries
            println!("cargo:rustc-link-lib=static={}", lib);
        }

        if target.contains("apple") {
            println!("cargo:rustc-link-lib=framework=Security");
            println!("cargo:rustc-link-lib=framework=Foundation");
        } else if target.contains("linux") {
            if pkg_config::Config::new().statik(true).probe("libunwind-ptrace").is_err() {
                link_manually(&["unwind-ptrace", "unwind-generic", "unwind"]);
            }
        }

        // TODO: Should ICU always be linked statically?
        if pkg_config::Config::new().statik(true).probe("icu-i18n").is_err() {
            // TODO: This may be embedded in ChakraCore?
            println!("cargo:warning=No libraries for icu4c (pkg_config), linking manually...");
            link_manually(&["icui18n", "icuuc", "icudata"]);
        }

        // TODO: Should this ever be linked statically?
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
}

fn link_manually(libs: &[&str]) {
    for lib in libs.iter() {
        println!("cargo:rustc-link-lib=static={}", lib);
    }
}

fn chakra_bindings() {
    let out_dir_str = env::var_os("OUT_DIR").expect("No $OUT_DIR specified");
    let out_dir_path = path::Path::new(&out_dir_str);

    let source = env::var_os("CHAKRA_SOURCE").expect("No $CHAKRA_SOURCE specified");
    let jsrt_dir_path = path::Path::new(&source).join("lib/Jsrt");

    // Convert 'ChakraCore.h' → 'ffi.rs'
    libbindgen::builder()
        // Source contains 'nullptr'
        .clang_arg("-xc++")
        .clang_arg("--std=c++11")
        // This must be after the Clang arguments
        .header(jsrt_dir_path.join("ChakraCore.h").to_str().unwrap())
        // Only include JSRT associated types (i.e not STL types)
        .whitelisted_function("^Js.+")
        .whitelisted_type("^Js.+")
        // These are not detected as dependencies
        .whitelisted_type("ChakraBytePtr")
        .whitelisted_type("TTDOpenResourceStreamCallback")
        // Some enums are used as bitfields
        .bitfield_enum(r"\w+Attributes")
        .bitfield_enum(r"\w+Modes")
        .ctypes_prefix("libc")
        .generate()
        .expect("Failed to generate binding")
        .write_to_file(out_dir_path.join("ffi.rs"))
        .expect("Failed to write binding to file");

    // Make the binding less cumbersome and platform agnostic
    sanitize_binding(&out_dir_path.join("ffi.rs"));
}

fn sanitize_binding(file: &path::Path) {
    let mut content = read_file_content(file);

    // Change calling convention → system
    regex_replace(&mut content, "extern \"C\"", "extern \"system\"");

    // Normalize all bitflags (removes the prepended enum name)
    regex_replace(&mut content, r"_\w+_(?P<name>\w+):", "$name:");

    // Ensure safety by making all void handles strongly typed, wrapping the
    // pointer in a struct. Also derive sensible defaults and add a constructor
    // to initialize the handle with a null pointer.
    regex_replace(&mut content, r"pub type (?P<name>\w+).+(?P<type>\*mut.+c_void);", &[
        "#[repr(C)]",
        "#[derive(Copy, Clone, Debug)]",
        "pub struct $name(pub $type);",
        "impl $name {",
            "pub fn new() -> Self { $name(::std::ptr::null_mut()) }",
        "}"
    ].join("\n"));

    // Remove all type aliases for underscored types:
    // (e.g 'pub type JsErrorCode = _JsErrorCode').
    regex_replace(&mut content, r"pub type (\w+) = _.+", "");

    // This is an edge case (the type definition does not match the enum name)
    regex_replace(&mut content, r"(?P<name>JsTTDMoveMode)s", "$name");

    // ... and rename all underscored types
    // (e.g '_JsErrorCode' → 'JsErrorCode')
    regex_replace(&mut content, r"\b_(?P<name>\w+)", "$name");

    // Enums are scoped in Rust, but they are not in C/C++. This leads to
    // verbose and repetitive code (e.g 'JsMemoryType::JsMemoryTypeAlloc'). To
    // prevent this, remove a specific prefix of all enum values. By default the
    // enum name (and some edge cases where the values do not match the name).
    let mut prefixes = regex_find(&content, r"enum (\w+)");

    // These prefixes do not correspond to the enum name
    prefixes.extend([
        "JsError",
        "JsArrayType",
        "JsModuleHostInfo",
        "JsMemory",
        "Js"
    ].iter().map(|s| s.to_string()));

    for prefix in prefixes.iter() {
        let ident = format!(r"{}_?(?P<name>\w+) = (?P<value>\d+)", prefix);
        regex_replace(&mut content, &ident, "$name = $value");
    }

    // Update the generated binding
    write_file_content(file, &content);
}

pub fn regex_replace(source: &mut String, ident: &str, replacement: &str) {
    let regex = Regex::new(ident).expect("Replacement regex has invalid syntax");
    *source = regex.replace_all(&source, replacement);
}

/// Returns a collection of the first capture group.
pub fn regex_find(source: &str, ident: &str) -> Vec<String> {
    Regex::new(ident)
        .expect("Find regex has invalid syntax")
        .captures_iter(source)
        .map(|cap| cap.at(1).unwrap().to_string())
        .collect()
}

pub fn read_file_content(path: &path::Path) -> String {
    let mut file = fs::File::open(path).expect("Could not open file");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).expect("Could not read file contents");
    buffer
}

pub fn write_file_content(path: &path::Path, content: &str) {
    let mut handle = fs::File::create(path).expect("Failed to create file");
    handle.write_all(content.as_bytes()).expect("Failed to write to file");
}
