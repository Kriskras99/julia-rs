use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let output = Command::new("julia")
        .arg("--quiet")
        .arg("--eval")
        .arg("print(VERSION)")
        .output()
        .expect("Could not find Julia!");
    let version = String::from_utf8(output.stdout).expect("VERSION is not valid UTF-8");
    assert!(version.starts_with("1.10."), "This release only supports 1.10");

    let output = Command::new("julia")
        .arg("--quiet")
        .arg("--eval")
        .arg("print(Sys.BINDIR)")
        .output()
        .expect("Could not find Julia!");

    let mut base_dir = String::from_utf8(output.stdout).expect("Sys.BINDIR is not valid UTF-8");
    assert!(
        base_dir.ends_with("/bin"),
        "Sys.BINDIR does not end in /bin"
    );
    base_dir.truncate(base_dir.len() - 4);

    let include_dir = format!("{base_dir}/include/julia");
    let lib_dir = format!("{base_dir}/lib");

    println!("cargo:rustc-link-lib=julia");
    println!("cargo:rustc-link-search=native={lib_dir}");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{include_dir}"))
        .allowlist_recursively(false)
        .allowlist_item("_*[Jj][Ll].*")
        .allowlist_type("_bigval_t")
        .allowlist_type("_mallocarray_t")
        .allowlist_type("arraylist_t")
        .allowlist_type("arraylist_t")
        .allowlist_type("bool_t")
        .allowlist_type("bufmode_t")
        .allowlist_type("bufstate_t")
        .allowlist_type("bufstate_t")
        .allowlist_type("htable_t")
        .allowlist_type("ios_t")
        .allowlist_type("pthread_t")
        .allowlist_type("sig_atomic_t")
        .allowlist_type("sigjmp_buf")
        .allowlist_type("small_arraylist_t")
        .allowlist_type("uv_file")
        .allowlist_type("uv_handle_t")
        .allowlist_type("uv_handle_s")
        .allowlist_type("uv_handle_type")
        .allowlist_type("uv_loop_s")
        .allowlist_type("uv_loop_t")
        .allowlist_type("uv_stream_t")
        .allowlist_type("uv_stream_s")
        .allowlist_type("uv_tcp_t")
        .allowlist_type("ws_queue_t")
        .allowlist_type("ws_array_t")
        .allowlist_function("arraylist_grow")
        .allowlist_function("pthread_self")
        .blocklist_function("jl_vprintf")
        .blocklist_function("jl_vexceptionf")
        .opaque_type("uv_.*")
        .opaque_type("sigjmp_buf")
        .opaque_type("sig_atomic_t")
        .opaque_type("_jl_typemap_entry_t")
        .opaque_type("_jl_binding_t")
        .opaque_type("_jl_code_instance_t")
        .opaque_type("_jl_sym_t")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
