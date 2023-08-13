use axum::{extract::FromRequestParts, http::request::Parts};

use std::fmt::Write;

use crate::{model::User, AppState};

#[derive(serde::Serialize)]
pub struct BaseRenderInfo {
    pub root_url: String,
    pub cdn_url: String,
    pub logged_in_user: Option<User>,
}

impl BaseRenderInfo {
    pub fn new(root_url: String, cdn_url: String) -> Self {
        Self {
            root_url,
            cdn_url,
            logged_in_user: None,
        }
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for BaseRenderInfo {
    type Rejection = crate::Error;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = User::from_request_parts(parts, state).await.ok();
        let bri = BaseRenderInfo {
            root_url: state.config.root_url.clone(),
            cdn_url: state.config.cdn_url.clone(),
            logged_in_user: user,
        };
        Ok(bri)
    }
}

pub struct MarkdownFilter;

impl tera::Filter for MarkdownFilter {
    fn filter(
        &self,
        value: &tera::Value,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        Ok(tera::Value::String(markdown::to_html(
            value.as_str().unwrap_or(""),
        )))
    }
    fn is_safe(&self) -> bool {
        true
    }
}

pub struct HumanizeDuration;

impl tera::Filter for HumanizeDuration {
    fn filter(
        &self,
        value: &tera::Value,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let total_time = value
            .as_u64()
            .ok_or_else(|| tera::Error::msg("Display duration was not a real number"))?;
        let output = millis_to_string(total_time)
            .map_err(|v| tera::Error::msg(format!("Failed formatting string: {v:?}")))?;
        Ok(tera::Value::String(output))
    }
    fn is_safe(&self) -> bool {
        false
    }
}

fn millis_to_string(total_time_ms: u64) -> Result<String, std::fmt::Error> {
    const MILLIS_PER_DAY: u64 = 86_400_000;
    const MILLIS_PER_HOUR: u64 = 3_600_000;
    const MILLIS_PER_MINUTE: u64 = 60_000;
    const MILLIS_PER_SECOND: u64 = 1000;

    let mut output = String::with_capacity(1024);

    let days = total_time_ms / MILLIS_PER_DAY;
    let remainder = total_time_ms % MILLIS_PER_DAY;

    let hours = remainder / MILLIS_PER_HOUR;
    let remainder = days % MILLIS_PER_HOUR;

    let minutes = remainder / MILLIS_PER_MINUTE;
    let remainder = hours % MILLIS_PER_MINUTE;

    let seconds = remainder / MILLIS_PER_SECOND;
    let milliseconds = seconds % MILLIS_PER_SECOND;

    let mut started = false;

    if days > 0 {
        write!(output, "{days} day{} ", pluralize(days))?;
        started = true;
    }
    if hours > 0 || started {
        write!(output, "{hours} hour{} ", pluralize(hours))?;
    }
    if minutes > 0 || started {
        write!(output, "{minutes} minute{} ", pluralize(minutes))?;
    }
    if seconds > 0 || started {
        write!(output, "{seconds} second{} ", pluralize(seconds))?;
    }
    write!(
        output,
        "{milliseconds} millisecond{}",
        pluralize(milliseconds)
    )?;
    Ok(output)
}

fn pluralize(num: u64) -> &'static str {
    if num == 1 {
        ""
    } else {
        "s"
    }
}
