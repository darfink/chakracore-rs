extern crate clang_sys;
extern crate libbindgen;
extern crate pkg_config;
extern crate regex;

use std::env;
use std::io::Write;
use std::fs;
use std::path;
use std::process::Command;

use clang_sys::support::Clang;
use regex::Regex;

const LIBS: [&'static str; 3] = [
    "Chakra.Pal",
    "Chakra.Common.Core",
    "Chakra.Jsrt"
];

fn main() {
    // This build relies heavily on pkg-config
    if !has_target("windows") && !has_command("pkg-config") {
        println!("cargo:warning=Cannot find pkg-config");
    }

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

    if has_target("linux") {
        // TODO: Replace this with ldconfig magic?
        maybe_search("/usr/lib/x86_64-linux-gnu");
        maybe_search("/usr/local/lib/x86_64-linux-gnu");
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
    if cfg!(feature = "shared") {
        // The dynamic library is completely self-contained
        println!("cargo:rustc-link-lib=dylib=ChakraCore");
    } else {
        // Statically link all ChakraCore libraries
        link_manually("static", &LIBS);

        if has_target("apple") {
            link_manually("framework", &["Security", "Foundation"]);
            link_manually("dylib", &["c++"])
        } else if has_target("linux") {
            // TODO: Support for builds without ptrace
            if pkg_config::Config::new().statik(true).probe("libunwind-ptrace").is_err() {
                link_manually("static", &["unwind-ptrace", "unwind-generic", "unwind"]);
            }

            // TODO: Why isn't this included in 'libunwind-ptrace'?
            if pkg_config::Config::new().statik(true).probe("liblzma").is_err() {
                link_manually("static", &["lzma"]);
            }

            // TODO: Should this ever be linked statically?
            link_manually("dylib", &["stdc++"]);
        }

        // TODO: Should ICU always be linked statically?
        if !has_target("windows") &&
                pkg_config::Config::new().statik(true).probe("icu-i18n").is_err() {
            println!("cargo:warning=No libraries for icu4c (pkg_config), linking manually...");
            link_manually("static", &["icui18n", "icuuc", "icudata"]);
        }
    }
}

fn link_manually(linkage: &str, libs: &[&str]) {
    for lib in libs.iter() {
        println!("cargo:rustc-link-lib={}={}", linkage, lib);
    }
}

fn chakra_bindings() {
    let clang = Clang::find(None).expect("No clang found, is it installed?");

    // Some default includes are not found without this (e.g 'stddef.h')
    let mut builder = clang.c_search_paths.iter().fold(libbindgen::builder(), |builder, ref path| {
        // Ensure all potential system paths are searched
        builder.clang_arg("-idirafter").clang_arg(path.to_str().unwrap())
    });

    if has_target("windows") {
        // Clang is not aware of 'uint8_t' and its cousins by default
        builder = ["-include", "stdint.h", "-Wno-pragma-once-outside-header"]
            .iter().fold(builder, |builder, carg| builder.clang_arg(*carg));
    }

    let source_dir_str = env::var_os("CHAKRA_SOURCE").expect("No $CHAKRA_SOURCE specified");
    let jsrt_dir_path = path::Path::new(&source_dir_str).join("lib/Jsrt");

    // Convert 'ChakraCore.h' → 'ffi.rs'
    let ffi = builder
        // Source contains 'nullptr'
        .clang_arg("-xc++")
        .clang_arg("--std=c++11")
        // This must be after the arguments to Clang
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
        .to_string();

    // Make the binding Rust friendly and platform agnostic
    let binding = sanitize_binding(ffi);

    let out_dir_str = env::var_os("OUT_DIR").expect("No $OUT_DIR specified");
    let out_dir_path = path::Path::new(&out_dir_str);

    // Write the generated binding to file
    write_file_content(&out_dir_path.join("ffi.rs"), &binding);
}

fn sanitize_binding(mut content: String) -> String {
    // Change calling convention from C → system
    regex_replace(&mut content, "extern \"C\"", "extern \"system\"");

    // Normalize all bitflags (by removing the prepended enum name)
    regex_replace(&mut content, r"_\w+_(?P<name>\w+):", "$name:");

    // Ensure safety by making all void handles strongly typed, wrapping the
    // pointer in a struct. Also derive sensible defaults and add a constructor
    // that initializes the handle with a null pointer.
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

    // ... and rename all underscored types. This is done so users can access
    // enums without using weird, prefixed syntax, such as '_JsErrorCode'.
    // (e.g '_JsErrorCode' → 'JsErrorCode')
    regex_replace(&mut content, r"\b_(?P<name>\w+)", "$name");

    // Enums are scoped in Rust, but they are not in C/C++. This leads to
    // verbose and cumbersome code (e.g 'JsMemoryType::JsMemoryTypeAlloc'). To
    // prevent this, remove a specific prefix of all enum values. By default the
    // enum name (and some edge cases where the values do not match the name).
    let mut prefixes_to_remove = regex_find(&content, r"enum (\w+)");

    // These prefixes do not correspond to the enum name
    prefixes_to_remove.extend([
        "JsError",
        "JsArrayType",
        "JsModuleHostInfo",
        "JsMemory",
        "Js"
    ].iter().map(|s| s.to_string()));

    for prefix in prefixes_to_remove.iter() {
        let ident = format!(r"{}_?(?P<name>\w+) = (?P<value>\d+)", prefix);
        regex_replace(&mut content, &ident, "$name = $value");
    }

    content
}

fn regex_replace(source: &mut String, ident: &str, replacement: &str) {
    let regex = Regex::new(ident).expect("Replacement regex has invalid syntax");
    *source = regex.replace_all(&source, replacement);
}

/// Returns a collection of the first capture group.
fn regex_find(source: &str, ident: &str) -> Vec<String> {
    Regex::new(ident)
        .expect("Find regex has invalid syntax")
        .captures_iter(source)
        .map(|cap| cap.at(1).unwrap().to_string())
        .collect()
}

fn write_file_content(path: &path::Path, content: &str) {
    let mut handle = fs::File::create(path).expect("Failed to create file");
    handle.write_all(content.as_bytes()).expect("Failed to write to file");
}

fn has_command(command: &str) -> bool {
    Command::new("which")
            .arg(command)
            .status()
            .ok()
            .map_or(false, |res| res.success())
}

fn has_target(target: &str) -> bool {
    env::var("TARGET").expect("No $TARGET specified").contains(target)
}
