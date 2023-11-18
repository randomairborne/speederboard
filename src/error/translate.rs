use std::{fmt::Error as FmtError, io::Error as IoError, num::TryFromIntError};

use axum::extract::rejection::FormRejection;
use axum_extra::extract::multipart::MultipartError;
use deadpool_redis::PoolError;
use garde::{Error as GardeError, Report as GardeReport};
use image::ImageError;
use redis::{ErrorKind as RedisErrorKind, RedisError};
use reqwest::Error as ReqwestError;
use s3::error::S3Error;
use serde_json::{error::Category as SerdeJsonErrorCategory, Error as SerdeJsonError};
use sqlx::Error as SqlxError;
use tera::ErrorKind as TeraErrorKind;
use tokio::task::JoinError;
use url::ParseError;

use super::{ArgonError, Error, ImageTooBig};

impl Error {
    pub(super) fn translation_key(&self) -> &'static str {
        match self {
            Self::Sqlx(err) => translate_sqlx(err),
            Self::DeadpoolRedis(err) => translate_deadpool_redis(err),
            Self::Redis(err) => translate_redis(err),
            Self::Tera(err) => translate_tera(err),
            Self::Argon2(err) => translate_argon(err),
            Self::SerdeJson(err) => translate_serde_json(err),
            Self::Reqwest(err) => translate_reqwest(err),
            Self::S3(err) => translate_s3(err),
            Self::Multipart(err) => translate_multipart(err),
            Self::TaskJoin(err) => translate_taskjoin(err),
            Self::TryFromInt(err) => translate_tryfromint(err),
            Self::Format(err) => translate_format(err),
            Self::Io(err) => translate_io(err),
            Self::FormValidation(err) => translate_form_validation(err),
            Self::MultiFormValidation(err) => translate_multi_form_validation(err),
            Self::FormRejection(err) => translate_form_rejection(err),
            Self::CustomFormValidation(err) => translate_custom_form_validation(err),
            Self::UrlParse(err) => translate_url_parse(err),
            Self::Image(err) => translate_image(err),
            Self::MissingQueryPair(err) => translate_missing_query_pair(err),
            Self::NeedsLogin(err) => translate_needs_login(err),
            Self::S3Status(err) => translate_s3_status(*err),
            Self::TooManyRows(expected, actual) => translate_too_many_rows(*expected, *actual),
            Self::ImageTooTall(err) => translate_img_too_tall(err),
            Self::ImageTooWide(err) => translate_img_too_wide(err),
            Self::OneshotRecv(_) => "error.oneshot_recv",
            Self::Impossible(_) => "error.impossible",
            Self::InvalidPassword => "error.invalid_password",
            Self::InvalidCookie => "error.invalid_cookie",
            Self::TokenHasIdButIdIsUnkown => "error.token_has_id_but_unknown",
            Self::NotFound => "error.not_found",
            Self::InsufficientPermissions => "error.insufficient_permissions",
            Self::InvalidGameCategoryPair => "error.invalid_game_category_pair",
            Self::NoTitleForRootPost => "error.no_title_for_root_post",
            Self::CannotDeleteDefaultCategory => "error.cannot_delete_default_category",
            Self::NoDomainInUrl => "error.no_domain_in_url",
            Self::NoFileStem => "error.no_file_stem",
            Self::InvalidOsString => "error.invalid_os_string",
            Self::PathHasNoParent => "error.path_has_no_parent",
            Self::RowDoesNotMatchInputGame => "error.row_does_not_match_input_game",
        }
    }
}

fn translate_sqlx(err: &SqlxError) -> &'static str {
    // todo: maybe support further nesting?
    match err {
        SqlxError::Configuration(_) => "error.sqlx.configuration",
        SqlxError::Database(_) => "error.sqlx.database",
        SqlxError::Io(_) => "error.sqlx.io",
        SqlxError::Tls(_) => "error.sqlx.tls",
        SqlxError::Protocol(_) => "error.sqlx.protocol",
        SqlxError::RowNotFound => "error.sqlx.row_not_found",
        SqlxError::TypeNotFound { .. } => "error.sqlx.type_not_found",
        SqlxError::ColumnIndexOutOfBounds { .. } => "error.sqlx.column_index_out_of_bounds",
        SqlxError::ColumnNotFound(_) => "error.sqlx.column_not_found",
        SqlxError::ColumnDecode { .. } => "error.sqlx.column_decode",
        SqlxError::Decode(_) => "error.sqlx.decode",
        SqlxError::AnyDriverError(_) => "error.sqlx.any_driver",
        SqlxError::PoolTimedOut => "error.sqlx.pool_timeout",
        SqlxError::PoolClosed => "error.sqlx.pool_close",
        SqlxError::WorkerCrashed => "error.sqlx.worker_crash",
        SqlxError::Migrate(_) => "error.sqlx.migrate",
        _ => "error.sqlx.unknown",
    }
}

fn translate_deadpool_redis(err: &PoolError) -> &'static str {
    match err {
        PoolError::Timeout(_) => "error.deadpool_redis.timeout",
        PoolError::Backend(redis_err) => translate_redis(redis_err),
        PoolError::Closed => "error.deadpool_redis.closed",
        PoolError::NoRuntimeSpecified => "error.deadpool_redis.no_runtime_specified",
        PoolError::PostCreateHook(_) => "error.deadpool_redis.post_create_hook",
    }
}

fn translate_redis(err: &RedisError) -> &'static str {
    match err.kind() {
        RedisErrorKind::ResponseError => "error.redis.response",
        RedisErrorKind::AuthenticationFailed => "error.redis.authentication",
        RedisErrorKind::TypeError => "error.redis.type",
        RedisErrorKind::ExecAbortError => "error.redis.exec_abort",
        RedisErrorKind::BusyLoadingError => "error.redis.busy_loading",
        RedisErrorKind::NoScriptError => "error.redis.no_script",
        RedisErrorKind::InvalidClientConfig => "error.redis.invalid_client_config",
        RedisErrorKind::Moved => "error.redis.moved",
        RedisErrorKind::Ask => "error.redis.ask",
        RedisErrorKind::TryAgain => "error.redis.try_again",
        RedisErrorKind::ClusterDown => "error.redis.cluster_down",
        RedisErrorKind::CrossSlot => "error.redis.cross_slot",
        RedisErrorKind::MasterDown => "error.redis.master_down",
        RedisErrorKind::IoError => "error.redis.io",
        RedisErrorKind::ClientError => "error.redis.client",
        RedisErrorKind::ExtensionError => "error.redis.extension",
        RedisErrorKind::ReadOnly => "error.redis.read_only",
        RedisErrorKind::MasterNameNotFoundBySentinel => "error.redis.master_name_not_found",
        RedisErrorKind::NoValidReplicasFoundBySentinel => "error.redis.no_valid_replicas_found",
        RedisErrorKind::EmptySentinelList => "error.redis.empty_sentinel_list",
        RedisErrorKind::NotBusy => "error.redis.not_busy",
        _ => "error.redis.unknown",
    }
}

fn translate_tera(err: &tera::Error) -> &'static str {
    match &err.kind {
        TeraErrorKind::Msg(msg) => translate_tera_custom(msg),
        TeraErrorKind::CircularExtend { .. } => "error.tera.circular_extend",
        TeraErrorKind::MissingParent { .. } => "error.tera.missing_parent",
        TeraErrorKind::TemplateNotFound(_) => "error.tera.template_not_found",
        TeraErrorKind::FilterNotFound(_) => "error.tera.filter_not_found",
        TeraErrorKind::TestNotFound(_) => "error.tera.test_not_found",
        TeraErrorKind::InvalidMacroDefinition(_) => "error.tera.invalid_macro_def",
        TeraErrorKind::FunctionNotFound(_) => "error.tera.function_not_found",
        TeraErrorKind::Json(_) => "error.tera.json",
        TeraErrorKind::CallFunction(_) => "error.tera.function_call",
        TeraErrorKind::CallFilter(_) => "error.tera.filter_call",
        TeraErrorKind::CallTest(_) => "error.tera.test_call",
        TeraErrorKind::Io(_) => "error.tera.io",
        TeraErrorKind::Utf8Conversion { .. } => "error.tera.utf8_conversion",
        _ => "error.tera.unknown",
    }
}

fn translate_tera_custom(msg: &str) -> &'static str {
    todo!()
}

fn translate_argon(err: &ArgonError) -> &'static str {
    match err {
        ArgonError::PasswordHash(_) => "error.argon.password_hash",
        ArgonError::Argon2(_) => "error.argon.argon2",
    }
}

fn translate_serde_json(err: &SerdeJsonError) -> &'static str {
    match err.classify() {
        SerdeJsonErrorCategory::Io => "error.serde_json.io",
        SerdeJsonErrorCategory::Syntax => "error.serde_json.syntax",
        SerdeJsonErrorCategory::Data => "error.serde_json.data",
        SerdeJsonErrorCategory::Eof => "error.serde_json.eof",
    }
}

fn translate_reqwest(err: &ReqwestError) -> &'static str {
    todo!()
}

fn translate_s3(err: &S3Error) -> &'static str {
    match err {
        S3Error::Utf8(_) => "error.s3.utf8",
        S3Error::MaxExpiry(_) => "error.s3.max_expiry",
        S3Error::Http(_, _) => "error.s3.http",
        S3Error::HttpFail => "error.s3.http_fail",
        S3Error::Credentials(_) => "error.s3.credentials",
        S3Error::Region(_) => "error.s3.region",
        S3Error::HmacInvalidLength(_) => "error.s3.hmac_invalid_length",
        S3Error::UrlParse(_) => "error.s3.url_parse",
        S3Error::Io(_) => "error.s3.io",
        S3Error::Reqwest(_) => "error.s3.reqwest",
        S3Error::HeaderToStr(_) => "error.s3.header_to_str",
        S3Error::FromUtf8(_) => "error.s3.from_utf8",
        S3Error::SerdeXml(_) => "error.s3.serde_xml",
        S3Error::InvalidHeaderValue(_) => "error.s3.invalid_header_value",
        S3Error::InvalidHeaderName(_) => "error.s3.invalid_header_name",
        S3Error::WLCredentials => "error.s3.wl_credentials",
        S3Error::RLCredentials => "error.s3.rl_credentials",
        S3Error::TimeFormatError(_) => "error.s3.time_format",
        S3Error::FmtError(_) => "error.s3.fmt",
        _ => "error.s3.unknown",
    }
}

fn translate_multipart(_err: &MultipartError) -> &'static str {
    "error.multipart"
}

fn translate_taskjoin(err: &JoinError) -> &'static str {
    if err.is_panic() {
        return "error.taskjoin.panicked";
    }
    "error.taskjoin.unknown"
}

fn translate_tryfromint(_err: &TryFromIntError) -> &'static str {
    "error.tryfromint"
}

fn translate_format(_err: &FmtError) -> &'static str {
    "error.fmt"
}

fn translate_io(_err: &IoError) -> &'static str {
    "error.io"
}

fn translate_form_validation(_err: &GardeError) -> &'static str {
    "error.garde.internal"
}

fn translate_multi_form_validation(err: &GardeReport) -> &'static str {
    "error.garde.report"
}

fn translate_form_rejection(err: &FormRejection) -> &'static str {
    match err {
        FormRejection::InvalidFormContentType(_) => "error.form_rejection.content_type",
        FormRejection::FailedToDeserializeForm(_) => "error.form_rejection.deserialize",
        FormRejection::FailedToDeserializeFormBody(_) => "error.form_rejection.body",
        FormRejection::BytesRejection(_) => "error.form_rejection.bytes",
        _ => "error.form_rejection.unknown",
    }
}

fn translate_custom_form_validation(err: &String) -> &'static str {
    todo!()
}

fn translate_url_parse(err: &ParseError) -> &'static str {
    todo!()
}

fn translate_image(err: &ImageError) -> &'static str {
    todo!()
}

fn translate_missing_query_pair(err: &str) -> &'static str {
    todo!()
}

fn translate_needs_login(err: &String) -> &'static str {
    todo!()
}

fn translate_s3_status(status: u16) -> &'static str {
    match status {
        100 => "error.s3_status.continue",
        304 => "error.s3_status.not_modified",
        400 => "error.s3_status.bad_request",
        403 => "error.s3_status.forbidden",
        404 => "error.s3_status.not_found",
        405 => "error.s3_status.method_not_allowed",
        408 => "error.s3_status.timed_out",
        409 => "error.s3_status.conflict",
        411 => "error.s3_status.length_required",
        412 => "error.s3_status.precondition_failed",
        416 => "error.s3_status.invalid_range",
        422 => "error.s3_status.unprocessable_entity",
        500 => "error.s3_status.internal_server_error",
        _ => "error.s3_status.unknown",
    }
}

fn translate_too_many_rows(expected: usize, actual: usize) -> &'static str {
    todo!()
}

fn translate_img_too_tall(err: &ImageTooBig) -> &'static str {
    todo!()
}

fn translate_img_too_wide(err: &ImageTooBig) -> &'static str {
    todo!()
}
