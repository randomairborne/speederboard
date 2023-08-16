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
        let (days, hours, minutes, seconds, milliseconds) = millis_to_ddhhmmssms(total_time);
        let output = millis_to_long_string(days, hours, minutes, seconds, milliseconds)
            .map_err(|v| tera::Error::msg(format!("Failed formatting string: {v:?}")))?;
        Ok(tera::Value::String(output))
    }
    fn is_safe(&self) -> bool {
        false
    }
}

pub struct Duration;

impl tera::Filter for Duration {
    fn filter(
        &self,
        value: &tera::Value,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let total_time = value
            .as_u64()
            .ok_or_else(|| tera::Error::msg("Display duration was not a real number"))?;
        let (days, hours, minutes, seconds, milliseconds) = millis_to_ddhhmmssms(total_time);
        let output = millis_to_sr_string(days, hours, minutes, seconds, milliseconds)
            .map_err(|v| tera::Error::msg(format!("Failed formatting string: {v:?}")))?;
        Ok(tera::Value::String(output))
    }
    fn is_safe(&self) -> bool {
        false
    }
}

fn millis_to_long_string(
    days: u64,
    hours: u64,
    minutes: u64,
    seconds: u64,
    millis: u64,
) -> Result<String, std::fmt::Error> {
    fn pluralize(num: u64) -> &'static str {
        if num == 1 {
            ""
        } else {
            "s"
        }
    }

    let mut output = String::with_capacity(1024);
    let mut started = false;

    if days > 0 {
        write!(output, "{days} day{} ", pluralize(days))?;
        started = true;
    }
    if hours > 0 || started {
        write!(output, "{hours} hour{} ", pluralize(hours))?;
        started = true;
    }
    if minutes > 0 || started {
        write!(output, "{minutes} minute{} ", pluralize(minutes))?;
        started = true;
    }
    if seconds > 0 || started {
        write!(output, "{seconds} second{} ", pluralize(seconds))?;
    }
    write!(output, "{millis} millisecond{}", pluralize(millis))?;
    Ok(output)
}

fn millis_to_sr_string(
    days: u64,
    hours: u64,
    minutes: u64,
    seconds: u64,
    millis: u64,
) -> Result<String, std::fmt::Error> {
    let mut output = String::with_capacity(1024);
    let mut started = true;

    if days > 0 {
        write!(output, "{days:0<2}:")?;
        started = true;
    }
    if hours > 0 || started {
        write!(output, "{hours:0<2}:")?;
        started = true;
    }
    if minutes > 0 || started {
        write!(output, "{minutes:0<2}:")?;
        started = true;
    }
    if seconds > 0 || started {
        write!(output, "{seconds:0<2}.")?;
    }
    write!(output, "{millis:0<3}")?;
    Ok(output)
}

fn millis_to_ddhhmmssms(total_time_ms: u64) -> (u64, u64, u64, u64, u64) {
    const MILLIS_PER_DAY: u64 = 86_400_000;
    const MILLIS_PER_HOUR: u64 = 3_600_000;
    const MILLIS_PER_MINUTE: u64 = 60_000;
    const MILLIS_PER_SECOND: u64 = 1000;

    let days = total_time_ms / MILLIS_PER_DAY;
    let mut remainder = total_time_ms % MILLIS_PER_DAY;

    let hours = remainder / MILLIS_PER_HOUR;
    remainder %= MILLIS_PER_HOUR;

    let minutes = remainder / MILLIS_PER_MINUTE;
    remainder %= MILLIS_PER_MINUTE;

    let seconds = remainder / MILLIS_PER_SECOND;
    let milliseconds = remainder % MILLIS_PER_SECOND;

    (days, hours, minutes, seconds, milliseconds)
}
