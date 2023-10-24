#[cfg(feature = "dev")]
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub redis_url: String,
    pub database_url: String,
    #[serde(default = "dev_defaults::root_url")]
    pub root_url: String,
    #[serde(default = "dev_defaults::static_url")]
    pub static_url: String,
    #[serde(default = "dev_defaults::user_content_url")]
    pub user_content_url: String,
    #[serde(default = "dev_defaults::fakes3_endpoint")]
    pub fakes3_endpoint: String,
    #[serde(default = "String::new")]
    pub fakes3_token: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[cfg(feature = "dev")]
mod dev_defaults {
    pub(super) fn root_url() -> String {
        String::from("http://localhost:8080")
    }
    pub(super) fn static_url() -> String {
        String::from("http://localhost:8000")
    }
    pub(super) fn user_content_url() -> String {
        String::from("http://localhost:8001")
    }
    pub(super) fn fakes3_endpoint() -> String {
        String::from("http://localhost:8002")
    }
}

#[cfg(not(feature = "dev"))]
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub redis_url: String,
    pub database_url: String,
    pub root_url: String,
    pub user_content_url: String,
    pub static_url: String,
    pub fakes3_endpoint: String,
    pub fakes3_token: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    8080
}
