use std::num::TryFromIntError;

use axum::extract::rejection::FormRejection;
use axum_extra::extract::multipart::MultipartError;
use deadpool_redis::PoolError;
use garde::Report;
use image::ImageError;
use redis::{ErrorKind as RedisErrorKind, RedisError};
use s3::error::S3Error;
use sqlx::Error as SqlxError;
use tera::ErrorKind as TeraErrorKind;
use tokio::{sync::oneshot::error::RecvError, task::JoinError};
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
            Self::OneshotRecv(err) => translate_recv(err),
            Self::SerdeJson(err) => translate_serde_json(err),
            Self::Reqwest(err) => translate_reqwest(err),
            Self::S3(err) => translate_s3(err),
            Self::Multipart(err) => translate_multipart(err),
            Self::TaskJoin(err) => translate_taskjoin(err),
            Self::TryFromInt(err) => translate_tryfromint(err),
            Self::Impossible(_) => "error.impossible",
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
        TeraErrorKind::Utf8Conversion { .. } => "error.tera.utf_8_conversion",
        _ => "error.tera.unknown",
    }
}

fn translate_tera_custom(msg: &str) -> &'static str {
    todo!()
}

fn translate_argon(err: &ArgonError) -> &'static str {
    todo!()
}

fn translate_recv(err: &RecvError) -> &'static str {
    todo!()
}

fn translate_serde_json(err: &serde_json::Error) -> &'static str {
    todo!()
}

fn translate_reqwest(err: &reqwest::Error) -> &'static str {
    todo!()
}

fn translate_s3(err: &S3Error) -> &'static str {
    todo!()
}

fn translate_multipart(err: &MultipartError) -> &'static str {
    todo!()
}

fn translate_taskjoin(err: &JoinError) -> &'static str {
    todo!()
}

fn translate_tryfromint(err: &TryFromIntError) -> &'static str {
    todo!()
}

fn translate_format(err: &std::fmt::Error) -> &'static str {
    todo!()
}

fn translate_io(err: &std::io::Error) -> &'static str {
    todo!()
}

fn translate_form_validation(err: &garde::Error) -> &'static str {
    todo!()
}

fn translate_multi_form_validation(err: &Report) -> &'static str {
    todo!()
}

fn translate_form_rejection(err: &FormRejection) -> &'static str {
    todo!()
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
        100 => "error.s3.continue",
        304 => "error.s3.not_modified",
        400 => "error.s3.bad_request",
        403 => "error.s3.forbidden",
        404 => "error.s3.not_found",
        405 => "error.s3.method_not_allowed",
        408 => "error.s3.timed_out",
        409 => "error.s3.conflict",
        411 => "error.s3.length_required",
        412 => "error.s3.precondition_failed",
        416 => "error.s3.invalid_range",
        422 => "error.s3.unprocessable_entity",
        500 => "error.s3.internal_server_error",
        _ => "error.s3.unknown",
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
