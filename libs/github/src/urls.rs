use once_cell::sync::Lazy;
use url::Url;

pub static OAUTH_AUTHORIZE_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://github.com/login/oauth/authorize").unwrap());

pub static OAUTH_ACCESS_TOKEN_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://github.com/login/oauth/access_token").unwrap());

pub static REST_API_URL: Lazy<Url> = Lazy::new(|| Url::parse("https://api.github.com").unwrap());
