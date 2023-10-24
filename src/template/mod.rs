mod translate;

#[cfg(feature = "dev")]
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, fmt::Write};

use axum::{extract::FromRequestParts, http::request::Parts};
use tera::{Tera, Value};
pub use translate::{get_translations, GetTranslation};

use crate::{model::User, AppState, Error};

fn real_tera() -> Tera {
    let translations = get_translations().expect("Failed to load translations");
    let mut tera = match Tera::new("./templates/**/*") {
        Ok(v) => v,
        Err(source) => {
            if let tera::ErrorKind::Msg(msg) = &source.kind {
                error!("Failed to reload templates: {}", msg);
                std::process::exit(1);
            } else {
                error!(?source, "Failed to reload templates");
                std::process::exit(1);
            }
        }
    };
    tera.register_filter("markdown", MarkdownFilter);
    tera.register_filter("long_format_duration", HumanizeDuration);
    tera.register_filter("duration", Duration);
    tera.register_filter("video_embed", VideoEmbedder);
    tera.register_function("devmode", DevModeFunction);
    tera.register_function("gettrans", translate::GetTranslation::new(translations));
    tera.autoescape_on(vec![".html", ".htm", ".jinja", ".jinja2"]);
    tera
}

#[cfg(feature = "dev")]
pub fn tera() -> Arc<RwLock<Tera>> {
    Arc::new(RwLock::new(real_tera()))
}

#[cfg(not(feature = "dev"))]
pub fn tera() -> Tera {
    real_tera()
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct BaseRenderInfo {
    pub root_url: String,
    pub user_content_url: String,
    pub static_url: String,
    pub logged_in_user: Option<User>,
    pub language: String,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for BaseRenderInfo {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = User::from_request_parts(parts, state).await.ok();
        let language = if let Some(user) = user.as_ref() {
            if let Some(lang) = user.language.as_ref() {
                lang.clone()
            } else {
                "en".to_owned()
            }
        } else {
            "en".to_owned()
        };
        let bri = BaseRenderInfo {
            root_url: state.config.root_url.clone(),
            static_url: state.config.static_url.clone(),
            user_content_url: state.config.user_content_url.clone(),
            logged_in_user: user,
            language,
        };
        Ok(bri)
    }
}

#[derive(serde::Serialize)]
pub struct ConfirmContext {
    #[serde(flatten)]
    pub base: BaseRenderInfo,
    pub action: String,
    pub action_url: String,
    pub return_to: String,
}

#[derive(Debug, Clone, Copy)]
struct DevModeFunction;

impl tera::Function for DevModeFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        Ok(Value::Bool(crate::DEV_MODE))
    }

    fn is_safe(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
struct MarkdownFilter;

impl tera::Filter for MarkdownFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        Ok(Value::String(markdown::to_html(
            value.as_str().unwrap_or("(error)"),
        )))
    }

    fn is_safe(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
struct VideoEmbedder;

impl tera::Filter for VideoEmbedder {
    #[tracing::instrument(name = "embed_video", level = tracing::Level::TRACE, skip(_args))]
    fn filter(
        &self,
        value: &Value,
        _args: &std::collections::HashMap<String, Value>,
    ) -> tera::Result<Value> {
        let Value::String(input) = value else {
            return Ok(value.clone());
        };
        let sanitized_link = tera::escape_html(input);
        trace!(sanitized_link, input, "sanitized link");
        let data = convert_link(input).unwrap_or_else(|_| simple_a(&sanitized_link));
        trace!(data, input, "converted to clickable");
        Ok(Value::String(data))
    }

    fn is_safe(&self) -> bool {
        true
    }
}

#[tracing::instrument(name = "convert_link", level = tracing::Level::TRACE)]
fn convert_link(input: &str) -> Result<String, Error> {
    let url = url::Url::parse(input)?;
    trace!(?url, input, "parsed URL");
    let Some(domain) = url.domain().map(str::to_ascii_lowercase) else {
        return Err(Error::NoDomainInUrl);
    };
    match domain.as_str() {
        "youtube.com" | "www.youtube.com" => {
            trace!(?url, "converting YouTube.com URL");
            let mut query = url.query_pairs();
            let maybe_id = query.find_map(|v| if v.0 == "v" { Some(v.1) } else { None });
            if let Some(id) = maybe_id {
                Ok(get_yt_embed_iframe(&id))
            } else {
                Ok(simple_a(url.as_str()))
            }
        }
        "youtu.be" | "www.youtu.be" => {
            trace!(?url, "converting YouTu.be URL");
            if let Some(Some(id)) = url.path_segments().map(|mut v| v.next()) {
                Ok(get_yt_embed_iframe(id))
            } else {
                Ok(simple_a(url.as_str()))
            }
        }
        _ => Ok(simple_a(url.as_str())),
    }
}

fn simple_a(safe_link: &str) -> String {
    format!(r#"<a href="{safe_link}" target="_blank" rel="noopener noreferrer">{safe_link}</a>"#)
}

fn get_yt_embed_iframe(dangerous_video_id: &str) -> String {
    let clean_data = tera::escape_html(dangerous_video_id);

    format!(
        r#"<iframe width="560" height="315" src="https://www.youtube.com/embed/{clean_data}"
            allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
            frameborder="0" allowfullscreen></iframe>"#
    )
}

struct HumanizeDuration;

impl tera::Filter for HumanizeDuration {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        let total_time = value
            .as_u64()
            .ok_or_else(|| tera::Error::msg("Display duration was not a real number"))?;
        let (days, hours, minutes, seconds, milliseconds) = millis_to_ddhhmmssms(total_time);
        let output = millis_to_long_string(days, hours, minutes, seconds, milliseconds)
            .map_err(|v| tera::Error::msg(format!("Failed formatting string: {v:?}")))?;
        Ok(Value::String(output))
    }

    fn is_safe(&self) -> bool {
        false
    }
}

struct GetUserLinks {
    base: String,
}

impl GetUserLinks {
    pub fn new(base: String) -> Self {
        Self { base }
    }
}

impl tera::Function for GetUserLinks {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let mut json_map = serde_json::Map::with_capacity(8);
        let Some(Value::Object(obj)) = args.get("user") else {
            return Err(tera::Error::msg(
                "getuserlinks() missing `user` or `user` of wrong type",
            ));
        };
        Ok(Value::Object(json_map))
    }

    fn is_safe(&self) -> bool {
        false
    }
}

struct GetGameLinks {
    base: String,
}

impl GetGameLinks {
    pub fn new(base: String) -> Self {
        Self { base }
    }
}

impl tera::Function for GetGameLinks {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let mut json_map = serde_json::Map::with_capacity(8);
        let Some(Value::Object(obj)) = args.get("game") else {
            return Err(tera::Error::msg(
                "getgamelinks() missing `game` or `game` of wrong type",
            ));
        };
        Ok(Value::Object(json_map))
    }

    fn is_safe(&self) -> bool {
        false
    }
}

struct Duration;

impl tera::Filter for Duration {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        let total_time = value
            .as_u64()
            .ok_or_else(|| tera::Error::msg("Display duration was not a real number"))?;
        let (days, hours, minutes, seconds, milliseconds) = millis_to_ddhhmmssms(total_time);
        let output = millis_to_sr_string(days, hours, minutes, seconds, milliseconds)
            .map_err(|v| tera::Error::msg(format!("Failed formatting string: {v:?}")))?;
        Ok(Value::String(output))
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
