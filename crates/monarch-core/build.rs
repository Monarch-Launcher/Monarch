use dotenv::dotenv;

fn main() {
    dotenv().ok();

    verify_env_vars();
    build_protobufs();
}

fn verify_env_vars() {
    let monarch_url: String = std::env::var("MONARCH_URL").expect("MONARCH_URL not set");

    if monarch_url.ends_with("/") {
        eprintln!("MONARCH_URL should not contain a trailing slash. Please remove the slash.");
        std::process::exit(1);
    }

    if let Ok(var) = std::env::var("DATABASE_URL") {
        println!("cargo:rustc-env=DATABASE_URL={var}")
    }

    // This line makes the variable available to `env!` macros
    println!("cargo:rustc-env=MONARCH_URL={monarch_url}");

    if let Err(e) = std::env::var("CARGO_PKG_VERSION") {
        eprintln!("CARGO_PKG_VERSION not set | Err: {}", e);
        std::process::exit(1);
    }
}

fn build_protobufs() {
    // protox: compile .proto -> FileDescriptorSet without needing system protoc
    let file_descriptor_set =
        protox::compile(["src/proto/games.proto"], ["src/proto"]).expect("Failed to compile proto files");

    // prost-build: generate Rust structs from the descriptor
    prost_build::Config::new()
        .compile_fds(file_descriptor_set).expect("Failed to compile fds");

    // Rebuild only when the proto changes
    println!("cargo:rerun-if-changed=src/proto/games.proto");
}