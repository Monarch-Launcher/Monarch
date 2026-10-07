use dotenv::dotenv;

fn main() {
    dotenv().ok();

    verify_env_vars();
}

fn verify_env_vars() {
    let login_url: String = std::env::var("EPICLOGIN_URL").expect("EPICLOGIN_URL not set");
    println!("cargo:rustc-env=EPICLOGIN_URL={login_url}");
}
