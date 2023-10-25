#[cfg(feature = "dev")]
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub redis_url: String,
    pub database_url: String,
    #[serde(default = "dev_defaults::root_url")]
    pub root_url: String,
    #[serde(default = "dev_defaults::static_url")]
    pub static_url: String,
    pub user_content_url: String,
    pub s3_bucket_name: String,
    #[serde(default = "dev_defaults::s3_region")]
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    #[serde(default = "default_path_style")]
    pub s3_path_style: bool,
    pub r2_account_id: Option<String>,
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
    pub(super) fn s3_region() -> Option<String> {
        Some(String::from("us-east-1"))
    }
}

#[cfg(not(feature = "dev"))]
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub redis_url: String,
    pub database_url: String,
    pub root_url: String,
    pub static_url: String,
    pub user_content_url: String,
    pub s3_bucket_name: String,
    pub s3_region: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    #[serde(default = "default_path_style")]
    pub s3_path_style: bool,
    pub r2_account_id: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    8080
}

fn default_path_style() -> bool {
    true
}
