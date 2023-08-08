#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub redis_url: String,
    pub database_url: String,
    pub root_url: String,
    pub cdn_url: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub s3_secret_access_key: String,
    pub s3_access_key_id: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    8080
}
