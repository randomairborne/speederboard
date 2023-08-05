use std::marker::PhantomData;

use sqlx::{postgres::PgTypeInfo, Postgres};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Id<T: IdMarker> {
    inner: i64,
    #[serde(skip)]
    phantom: PhantomData<T>,
}

impl<T: IdMarker> Id<T> {
    pub fn get(&self) -> i64 {
        self.inner
    }
    pub fn cast<O: IdMarker>(self) -> Id<O> {
        Id {
            inner: self.inner,
            phantom: PhantomData,
        }
    }
}

impl<T: IdMarker> std::fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<T: IdMarker> std::fmt::Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<T: IdMarker> sqlx::Type<Postgres> for Id<T> {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        PgTypeInfo::with_name("BIGINT")
    }
}

impl<'q, T: IdMarker, DB: sqlx::Database> sqlx::Encode<'q, DB> for Id<T>
where
    i64: sqlx::Encode<'q, DB>,
{
    fn encode(
        self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull
    where
        Self: Sized,
    {
        sqlx::Encode::<DB>::encode(self.inner, buf)
    }
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        sqlx::Encode::<DB>::encode_by_ref(&self.inner, buf)
    }
    fn produces(&self) -> Option<<DB as sqlx::Database>::TypeInfo> {
        sqlx::Encode::<DB>::produces(&self.inner)
    }
    fn size_hint(&self) -> usize {
        sqlx::Encode::<DB>::size_hint(&self.inner)
    }
}

impl<'q, T: IdMarker, DB: sqlx::Database> sqlx::Decode<'q, DB> for Id<T>
where
    i64: sqlx::Decode<'q, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'q>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let inner = sqlx::Decode::decode(value)?;
        Ok(Self {
            inner,
            phantom: PhantomData,
        })
    }
}

impl<T: IdMarker> From<i64> for Id<T> {
    fn from(inner: i64) -> Self {
        Self {
            inner,
            phantom: PhantomData
        }
    }
}

impl<T: IdMarker> From<Id<T>> for i64 {
    fn from(id: Id<T>) -> Self {
        id.inner
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait IdMarker {}

pub struct UserMarker;
impl IdMarker for UserMarker {}
pub struct GameMarker;
impl IdMarker for GameMarker {}
pub struct CategoryMarker;
impl IdMarker for CategoryMarker {}
pub struct RunMarker;
impl IdMarker for RunMarker {}
