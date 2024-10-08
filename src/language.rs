use std::fmt::Formatter;

use serde::{de::Error, Deserializer, Serializer};
use sqlx::{encode::IsNull, error::BoxDynError, Database};
use strum::{EnumCount, EnumIter};

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, EnumIter, EnumCount)]
pub enum Language {
    #[default]
    English,
    Spanish,
    French,
    German,
    Japanese,
    Chinese,
}

impl Language {
    pub const CODES: [&'static str; Self::COUNT] = [
        Self::English.lang_code(),
        Self::Spanish.lang_code(),
        Self::French.lang_code(),
        Self::German.lang_code(),
        Self::Japanese.lang_code(),
        Self::Chinese.lang_code(),
    ];
    const CODE_CHINESE: &'static str = "zh";
    const CODE_ENGLISH: &'static str = "en";
    const CODE_FRENCH: &'static str = "fr";
    const CODE_GERMAN: &'static str = "de";
    const CODE_JAPANESE: &'static str = "ja";
    const CODE_SPANISH: &'static str = "es";

    pub const fn display(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Spanish => "Español",
            Self::French => "Français",
            Self::German => "Deutsch",
            Self::Japanese => "日本語",
            Self::Chinese => "中文",
        }
    }

    pub const fn lang_code(self) -> &'static str {
        match self {
            Self::English => Self::CODE_ENGLISH,
            Self::Spanish => Self::CODE_SPANISH,
            Self::French => Self::CODE_FRENCH,
            Self::German => Self::CODE_GERMAN,
            Self::Japanese => Self::CODE_JAPANESE,
            Self::Chinese => Self::CODE_CHINESE,
        }
    }

    pub fn from_lang_code(data: &str) -> Option<Self> {
        let lang = match data {
            Self::CODE_ENGLISH => Self::English,
            Self::CODE_SPANISH => Self::Spanish,
            Self::CODE_FRENCH => Self::French,
            Self::CODE_GERMAN => Self::German,
            Self::CODE_JAPANESE => Self::Japanese,
            Self::CODE_CHINESE => Self::Chinese,
            _ => return None,
        };
        Some(lang)
    }
}

impl serde::Serialize for Language {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.lang_code())
    }
}

impl<'de> serde::Deserialize<'de> for Language {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(LanguageVisitor)
    }
}

struct LanguageVisitor;

impl<'de> serde::de::Visitor<'de> for LanguageVisitor {
    type Value = Language;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a string which is a valid language ID")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Language::from_lang_code(value).ok_or(E::unknown_variant(value, &Language::CODES))
    }
}

impl<'q, DB: sqlx::Database> sqlx::Encode<'q, DB> for Language
where
    &'static str: sqlx::Encode<'q, DB>,
{
    fn encode(self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<IsNull, BoxDynError>
    where
        Self: Sized,
    {
        sqlx::Encode::<DB>::encode(self.lang_code(), buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        sqlx::Encode::<DB>::encode_by_ref(&self.lang_code(), buf)
    }

    fn produces(&self) -> Option<<DB as sqlx::Database>::TypeInfo> {
        sqlx::Encode::<DB>::produces(&self.lang_code())
    }

    fn size_hint(&self) -> usize {
        sqlx::Encode::<DB>::size_hint(&self.lang_code())
    }
}

impl<'q, DB: sqlx::Database> sqlx::Decode<'q, DB> for Language
where
    String: sqlx::Decode<'q, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'q>) -> Result<Self, BoxDynError> {
        let inner: String = sqlx::Decode::decode(value)?;
        Ok(Self::from_lang_code(&inner).unwrap_or_default())
    }
}

impl<DB: sqlx::Database> sqlx::Type<DB> for Language
where
    str: sqlx::Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        str::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        str::compatible(ty)
    }
}
