use dotenv::dotenv;

fn main() {
    dotenv().ok();

    verify_env_vars();
}

fn verify_env_vars() {
    // The workspace .env lives at the repo root, outside this package, so
    // Cargo's default "rerun on any package change" won't catch edits to it.
    println!("cargo:rerun-if-changed=../../.env");
    println!("cargo:rerun-if-changed=build.rs");

    let login_url: String = std::env::var("EPICLOGIN_URL").expect("EPICLOGIN_URL not set");
    assert!(
        !login_url.trim().is_empty(),
        "EPICLOGIN_URL is set but empty"
    );
    println!("cargo:rustc-env=EPICLOGIN_URL={login_url}");
}
