use super::Error;

impl Error {
    pub(super) fn translation_key(&self) -> &'static str {
        match self {
            Self::Sqlx(err) => translate_sqlx(err),
            Self::DeadpoolRedis(err) => todo!(),
            Self::Redis(err) => todo!(),
            Self::Tera(err) => todo!(),
            Self::Argon2(err) => todo!(),
            Self::OneshotRecv(err) => todo!(),
            Self::SerdeJson(err) => todo!(),
            Self::Reqwest(err) => todo!(),
            Self::S3(err) => todo!(),
            Self::Multipart(err) => todo!(),
            Self::TaskJoin(err) => todo!(),
            Self::TryFromInt(err) => todo!(),
            Self::InvalidMultipart(err) => todo!(),
            Self::Impossible(err) => todo!(),
            Self::Format(err) => todo!(),
            Self::Io(err) => todo!(),
            Self::FormValidation(err) => todo!(),
            Self::MultiFormValidation(err) => todo!(),
            Self::FormRejection(err) => todo!(),
            Self::CustomFormValidation(err) => todo!(),
            Self::UrlParse(err) => todo!(),
            Self::Image(err) => todo!(),
            Self::MissingQueryPair(err) => todo!(),
            Self::NeedsLogin(err) => todo!(),
            Self::S3Status(err) => translate_s3(*err),
            Self::TooManyRows(_, _) => todo!(),
            Self::ImageTooTall(err) => todo!(),
            Self::ImageTooWide(err) => todo!(),
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

fn translate_sqlx(err: &sqlx::Error) -> &'static str {
    // todo: maybe support further nesting?
    match err {
        sqlx::Error::Configuration(_) => "error.sqlx.configuration",
        sqlx::Error::Database(_) => "error.sqlx.database",
        sqlx::Error::Io(_) => "error.sqlx.io",
        sqlx::Error::Tls(_) => "error.sqlx.tls",
        sqlx::Error::Protocol(_) => "error.sqlx.protocol",
        sqlx::Error::RowNotFound => "error.sqlx.row_not_found",
        sqlx::Error::TypeNotFound { type_name } => "error.sqlx.type_not_found",
        sqlx::Error::ColumnIndexOutOfBounds { index, len } => {
            "error.sqlx.column_index_out_of_bounds"
        }
        sqlx::Error::ColumnNotFound(_) => "error.sqlx.column_not_found",
        sqlx::Error::ColumnDecode { index, source } => "error.sqlx.column_decode",
        sqlx::Error::Decode(_) => "error.sqlx.decode",
        sqlx::Error::AnyDriverError(_) => "error.sqlx.any_driver",
        sqlx::Error::PoolTimedOut => "error.sqlx.pool_timeout",
        sqlx::Error::PoolClosed => "error.sqlx.pool_close",
        sqlx::Error::WorkerCrashed => "error.sqlx.worker_crash",
        sqlx::Error::Migrate(_) => "error.sqlx.migrate",
        _ => "error.sqlx.unknown",
    }
}

fn translate_s3(status: u16) -> &'static str {
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
        _ => "error.s3.unknown"
    }
}