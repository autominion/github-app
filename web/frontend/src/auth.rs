#[cfg(target_arch = "wasm32")]
pub fn set_cookies(access_token: &str, refresh_token: &str) {
    use wasm_cookies::{CookieOptions, SameSite};

    let options = CookieOptions { secure: true, same_site: SameSite::Strict, ..Default::default() };
    wasm_cookies::set("access_token", access_token, &options);
    wasm_cookies::set("refresh_token", refresh_token, &options);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_cookies(_access_token: &str, _refresh_token: &str) {}

#[cfg(target_arch = "wasm32")]
pub fn logged_in() -> bool {
    wasm_cookies::get("access_token").is_some()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn logged_in() -> bool {
    false
}

pub fn delete_cookies() {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_cookies::delete("access_token");
        wasm_cookies::delete("refresh_token");
    }
}
