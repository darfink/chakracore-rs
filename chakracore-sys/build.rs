extern crate clang_sys;
extern crate libbindgen;
extern crate pkg_config;
extern crate regex;

use std::{env, fs};
use std::path::{Path, PathBuf};

const LIBRARY: &'static str = "ChakraCore";
const REPOSITORY: &'static str = "https://github.com/Microsoft/ChakraCore.git";
const VERSION: &'static str = "1.4";
const LIBS: [(&'static str, &'static str); 3] = [
    ("pal/src",         "Chakra.Pal"),
    ("lib/Common/Core", "Chakra.Common.Core"),
    ("lib/Jsrt",        "Chakra.Jsrt"),
];

macro_rules! get(($name:expr) => (env::var($name).unwrap()));
macro_rules! log {
    ($fmt:expr) => (println!(concat!("chakracore-sys/build.rs:{}: ", $fmt), line!()));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("chakracore-sys/build.rs:{}: ", $fmt), line!(), $($arg)*));
}

fn main() {
    if util::has_target("windows") {
        if cfg!(features = "static") {
          // This is related to ChakraCore (see #279)
          panic!("Windows build does not support static linkage");
        }

        if !util::has_target("msvc") {
            // The runtime errors are very subtle, so be explicit
            panic!("Only MSVC toolchain is compatible with ChakraCore");
        }
    } else {
        // This build relies heavily on pkg-config
        util::run_command("which", &["pkg-config"], None);
    }

    // If both these are set, they will override the default build settings
    let overrides = [env::var("CHAKRA_SOURCE"), env::var("CHAKRA_BUILD")];

    assert!(overrides.iter().filter(|var| var.is_ok()).count() != 1,
            "Only one of $CHAKRA_SOURCE/BUILD variable was set");
    let (src_dir, lib_dirs) = if overrides.iter().any(|var| var.is_err()) {
        setup_default()
    } else {
        setup_custom()
    };

    binding::generate(&src_dir);
    linking::setup(&lib_dirs);
}

fn setup_custom() -> (PathBuf, Vec<PathBuf>) {
    // Use the user supplied build directory
    let build_dir = PathBuf::from(get!("CHAKRA_BUILD"));
    let src_dir = PathBuf::from(get!("CHAKRA_SOURCE"));

    (src_dir, LIBS.iter().map(|&(dir, _)| build_dir.join(dir)).collect())
}

fn setup_default() -> (PathBuf, Vec<PathBuf>) {
    let out_dir = PathBuf::from(&get!("OUT_DIR"));
    let lib_dir = out_dir.join(format!("lib-{}", VERSION));

    // Directory where all sources are stored
    let src_dir = PathBuf::from(&get!("CARGO_MANIFEST_DIR"))
        .join(format!("target/source-{}", VERSION));

    if !lib_dir.exists() {
        log!("Creating directory '{:?}'", lib_dir);
        fs::create_dir(&lib_dir).expect("Could not create library directory");
    }

    // Clone the repository for local access
    if !Path::new(&src_dir.join(".git")).exists() {
        util::run_command("git", &[
            "clone",
            &format!("--branch=release/{}", VERSION),
            REPOSITORY,
            src_dir.to_str().unwrap(),
        ], None);
    }

    let has_required_libs = if cfg!(feature = "static") {
        // The static archives consists of three different files
        LIBS.iter().all(|&(_, name)| lib_dir.join(linking::format_lib(name)).exists())
    } else {
        lib_dir.join(linking::format_lib(LIBRARY)).exists()
    };

    if !has_required_libs {
        let build_dir = build::compile(&src_dir);
        build::copy_libs(&build_dir, &lib_dir);
    }

    // Return the source and lib directory
    (src_dir, vec![lib_dir])
}

mod build {
    use std::{env, fs};
    use std::path::{Path, PathBuf};
    use pkg_config;
    use {util, linking, LIBRARY, LIBS};

    /// Builds the ChakraCore project.
    pub fn compile(src_dir: &Path) -> PathBuf {
        let arch = util::get_arch(&get!("TARGET"));
        if util::has_target("windows") {
            // This requires `vcvars` to be sourced
            util::run_command("msbuild", &[
                "/m",
                "/p:Configuration=Test",
                &format!("/p:Platform={:?}", arch),
                r"Build\Chakra.Core.sln",
            ], Some(&src_dir));

            src_dir.join(format!("Build/VcBuild/bin/{:?}_test", arch))
        } else {
            // The ICU directory must be configued using pkg-config
            let icu_include = pkg_config::get_variable("icu-i18n", "includedir")
                .expect("No package configuration for 'icu-i18n' found");

            // These need to live long enough
            let arg_icu = format!("--icu={}", icu_include);
            let arg_jobs = format!("--jobs={}", get!("NUM_JOBS"));

            let mut arguments = vec![
                #[cfg(feature = "static")]
                "--static",
                "--test-build",
                &arg_jobs,
                &arg_icu,
            ];

            match arch {
                util::Architecture::arm => panic!("ARM is only supported on Windows"),
                util::Architecture::x86 => arguments.push("--arch=TARGETSx86"),
                util::Architecture::x64 => /* This is the default */ (),
            }

            // Use the build script bundled in ChakraCore
            util::run_command("./build.sh", &arguments, Some(&src_dir));

            // Hopefully this directory won't change
            src_dir.join("BuildLinux/Test")
        }
    }

    /// Copies all binaries to the local 'libs' folder.
    pub fn copy_libs(build_dir: &Path, lib_dir: &Path) {
        let build_dir = build_dir.to_path_buf();

        let deps = if cfg!(feature = "static") {
            LIBS.iter().map(|&(dir, name)| (build_dir.join(dir), linking::format_lib(name))).collect()
        } else {
            vec![
                #[cfg(windows)]
                // Windows requires an import library as well
                (build_dir.clone(), format!("{}.lib", LIBRARY)),
                (build_dir, linking::format_lib(LIBRARY)),
            ]
        };

        for (dir, name) in deps {
            fs::copy(dir.join(&name), lib_dir.join(&name))
                .expect(&format!("Failed to copy '{}' to target directory", name));
        }
    }
}

mod linking {
    use std::path::{Path, PathBuf};
    use std::fs;
    use pkg_config;
    use {util, LIBS};

    /// Prints linking setup to Cargo.
    pub fn setup(search_paths: &[PathBuf]) {
        for path in search_paths {
            add_path(path);
        }

        if cfg!(feature = "static") {
            // Statically link all ChakraCore libraries
            link_manually("static", &LIBS.iter().map(|&(_, name)| name).collect::<Vec<_>>());

            if util::has_target("apple") {
                link_manually("framework", &["Security", "Foundation"]);
            } else if util::has_target("linux") {
                // TODO: Support for builds without ptrace
                link_library("libunwind-ptrace", true);
                link_library("liblzma", true);
            }

            // Use 'libstdc++' on all Unixes (like ChakraCore build)
            link_manually("dylib", &["stdc++"]);
            link_library("icu-i18n", true);
        } else {
            // The dynamic library is completely self-contained
            link_manually("dylib", &["ChakraCore"]);
        }
    }

    /// Returns a filename to OS specific format.
    pub fn format_lib(name: &str) -> String {
        if cfg!(windows) {
            format!("{}.dll", name)
        } else if cfg!(feature = "static") {
            format!("lib{}.a", name)
        } else if cfg!(target_os = "macos") {
            format!("lib{}.dylib", name)
        } else {
            format!("lib{}.so", name)
        }
    }

    /// Adds a library search path.
    fn add_path<P>(dir: P) where P: AsRef<Path> {
        let dir = dir.as_ref();
        assert!(fs::metadata(dir).map(|m| m.is_dir()).unwrap_or(false),
                format!("Library search path {:?} does not exist", dir));
        println!("cargo:rustc-link-search=native={}", dir.to_string_lossy());
    }

    fn link_library(name: &str, statik: bool) {
        pkg_config::Config::new().statik(statik).probe(name).ok()
            .expect(&format!("Could not find '{}'", name));
    }

    fn link_manually(linkage: &str, libs: &[&str]) {
        for lib in libs.iter() {
            println!("cargo:rustc-link-lib={}={}", linkage, lib);
        }
    }
}

mod binding {
    use std::env;
    use std::path::Path;
    use clang_sys::support::Clang;
    use libbindgen;
    use regex::Regex;
    use util;

    pub fn generate(src_dir: &Path) {
        let clang = Clang::find(None).expect("No clang found, is it installed?");

        // Some default includes are not found without this (e.g 'stddef.h')
        let mut builder = clang.c_search_paths.iter().fold(libbindgen::builder(), |builder, ref path| {
            // Ensure all potential system paths are searched
            builder.clang_arg("-idirafter").clang_arg(path.to_str().unwrap())
        });

        if util::has_target("windows") {
            // Clang is not aware of 'uint8_t' and its cousins by default
            builder = ["-include", "stdint.h", "-Wno-pragma-once-outside-header"]
                .iter().fold(builder, |builder, carg| builder.clang_arg(*carg));
        }

        // Convert 'ChakraCore.h' → 'ffi.rs'
        let ffi = builder
            // Source contains 'nullptr'
            .clang_arg("-xc++")
            .clang_arg("--std=c++11")
            // This must be after the arguments to Clang
            .header(src_dir.join("lib/Jsrt").join("ChakraCore.h").to_str().unwrap())
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
        let out_dir_path = Path::new(&out_dir_str);

        // Write the generated binding to file
        util::write_file_content(&out_dir_path.join("ffi.rs"), &binding);
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

    /// Replaces all occurences with a specified replacement.
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
}

mod util {
    use std::{env, fs};
    use std::process::Command;
    use std::path::Path;
    use std::io::Write;

    pub fn write_file_content(path: &Path, content: &str) {
        let mut handle = fs::File::create(path).expect("Failed to create file");
        handle.write_all(content.as_bytes()).expect("Failed to write to file");
    }

    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum Architecture {
        x86,
        x64,
        arm,
    }

    pub fn get_arch(target: &str) -> Architecture {
        if target.starts_with("x86_64") {
            Architecture::x64
        } else if target.starts_with("i686") {
            Architecture::x86
        } else if target.starts_with("arm") {
            Architecture::arm
        } else {
            panic!("Unknown target architecture");
        }
    }

    pub fn run_command(name: &str, arguments: &[&str], directory: Option<&Path>) {
        let mut command = Command::new(name);
        if let Some(path) = directory {
            command.current_dir(path);
        }

        for argument in arguments {
            command.arg(argument);
        }

        if !command.status().ok().map_or(false, |res| res.success()) {
            panic!(format!("Failed to run command {}", name));
        }
    }

    pub fn has_target(target: &str) -> bool {
        env::var("TARGET").expect("No $TARGET specified").contains(target)
    }
}
