#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub redis_url: String,
    pub database_url: String,
    pub root_url: String,
    pub cdn_url: String,
    pub fakes3_endpoint: String,
    pub fakes3_token: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    8080
}
