extern crate bindgen;

use std::fs;
use std::env;
use std::env::var;
use std::path::PathBuf;

// Note: Actual file has `lib` and '.a` prepended/appended (e.g `libsqlite3rustbuild.a`).
static FILE_SQLITE_COMPILED: &'static str = "sqlite3rustbuild";
static FILE_BINDING: &'static str = "bindings.rs";


fn get_sqlite3_amalgamation_dir(version: &str) -> String {
    let crate_with_build_script_dir = var("CARGO_MANIFEST_DIR").unwrap();
    format!("{}/c-code/sqlite3/{}/all", crate_with_build_script_dir, version)
}

fn add_flags(cfg: &mut cc::Build, flags: &[&str]) {
    for o in flags.iter() {
        cfg.flag(format!("-D{}", o).as_str());
    }
}

/*
@see https://www.sqlite.org/compile.html
@see https://github.com/buybackoff/RustTheNewC/blob/df35bf425f69ace9dc8d996b16b3932a2cfedd71/src/rust/sqlite-sys/build.rs
@see http://hotforknowledge.com/2019/07/14/6-rust-the-new-c/
@todo/low Check default flags are optimal, correct.
*/
fn add_sqlite3_flags(cfg: &mut cc::Build) {
    let base = [
        "SQLITE_CORE",
        "SQLITE_DEFAULT_FOREIGN_KEYS=1",
        "SQLITE_ENABLE_API_ARMOR",
        "SQLITE_ENABLE_COLUMN_METADATA",
        "SQLITE_ENABLE_DBSTAT_VTAB",
        "SQLITE_ENABLE_FTS3",
        "SQLITE_ENABLE_FTS3_PARENTHESIS",
        "SQLITE_ENABLE_FTS5",
        "SQLITE_ENABLE_JSON1",
        "SQLITE_ENABLE_LOAD_EXTENSION=1",
        "SQLITE_ENABLE_MEMORY_MANAGEMENT",
        "SQLITE_ENABLE_RTREE",
        "SQLITE_ENABLE_STAT2",
        "SQLITE_ENABLE_STAT4",
        "SQLITE_HAVE_ISNAN",
        "SQLITE_SOUNDEX",
        "SQLITE_THREADSAFE=1",
        "SQLITE_USE_URI",
        "HAVE_USLEEP=1"
    ];


    // Sets the default `PRAGMA synchronous` setting.
    // SQLITE_DEFAULT_WAL_SYNCHRONOUS=1
    //  1 = Normal
    //  2 = Full, (default).


    let optional = [
        "SQLITE_ENABLE_UNLOCK_NOTIFY",
        "SQLITE_ENABLE_PREUPDATE_HOOK",
        "SQLITE_ENABLE_SESSION",
    ];


    // SQLITE_MAX_EXPR_DEPTH
    // SQLITE_MAX_VARIABLE_NUMBER

    add_flags(cfg, &base);
    add_flags(cfg, &optional);

    // @todo/low `.opt_level(2)`, `.static_crt(true)`?
}


fn compile_sqlite3(dir: &String) {
    let src = format!("{}/sqlite3.c", dir);
    println!("Compiling {}", src);


    let mut cfg = cc::Build::new();

    cfg.file(src);

    // Ignore `warning: unused parameter` from sqlite3.c code.
    cfg.warnings(false);

    add_sqlite3_flags(&mut cfg);
    cfg.compile(FILE_SQLITE_COMPILED);


    println!("Compile complete, OUT_DIR={}", env::var("OUT_DIR").unwrap());
}

fn build_already_ran(out_dir: &String) -> bool {
    let must_exist = vec![
        format!("lib{}.a", FILE_SQLITE_COMPILED.to_string()),
        FILE_BINDING.to_string()
    ];

    let paths = fs::read_dir(out_dir).unwrap();


    let mut c = 0;
    for path in paths {
        if must_exist.contains(&path.unwrap().file_name().into_string().unwrap()) {
            c += 1
        }
    }

    c == must_exist.len()
}


/*
Builds SQLite, generates Rust bindings with bindgen.

- Note: must download the SQLite artifact first.
- `cargo build` will run this code to build artifacts that Rust code will depend on.

@see https://rust-lang.github.io/rust-bindgen/tutorial-4.html
@see https://rust-embedded.github.io/book/interoperability/c-with-rust.html
@see https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script
@see https://doc.rust-lang.org/cargo/reference/build-scripts.html#case-study-building-some-native-code
@see https://github.com/rust-lang/rust-bindgen/blob/master/CONTRIBUTING.md#code-overview

Questions
- How is SQLite built for different platforms?
    - Does building the Rust project for different platforms also build SQLite for the same platforms?


@todo/low
- White list bindings generated.


*/
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Config `cargo` to statically include SQLite compilation output WHEN compiling workspace Rust code (after build.rs is run).
    println!("cargo:rustc-link-search={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", FILE_SQLITE_COMPILED);


    // Issue: When changing Rust library test code slightly, `cargo` re-compiles SQLite C code and bindings.rs which takes about 20s.
    // - `cargo` does not seem to detect that none of the input files to `build.rs` has changed (so it should not be re-run).
    // Fix: Logic to determine if `build.rs` needs to be re-run.
    if build_already_ran(&out_dir) {
        println!("Skipping build as SQLite C code already compiled, bindings.rs already generated.");
        return;
    }


    // Build SQLite.
    let sqlite_version = "3280000";
    let sqlite_amalgamation_dir = get_sqlite3_amalgamation_dir(sqlite_version);
    compile_sqlite3(&sqlite_amalgamation_dir);

    // Generate bindings.
    let bindings = bindgen::Builder::default()
        .header(format!("{}/sqlite3.h", sqlite_amalgamation_dir))
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");


    // Write bindings.
    let out_path = PathBuf::from(out_dir);
    bindings
        .write_to_file(out_path.join(FILE_BINDING))
        .expect("Couldn't write bindings!");
}
