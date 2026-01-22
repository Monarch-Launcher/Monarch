use dotenv::dotenv;

fn main() {
    dotenv().ok();

    verify_env_vars();
    tauri_build::build()
}

fn verify_env_vars() {
    let monarch_url: String = std::env::var("MONARCH_URL").expect("MONARCH_URL not set");

    if monarch_url.ends_with("/") {
        eprintln!("MONARCH_URL should not contain a trailing slash. Please remove the slash.");
        std::process::exit(1);
    }

    // This line makes the variable available to `env!` macros
    println!("cargo:rustc-env=MONARCH_URL={}", monarch_url);
}
