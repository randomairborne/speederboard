use crate::language::Language;
use std::{collections::HashMap, sync::Arc};
use tera::Value;

pub struct GetTranslation {
    translations: Arc<HashMap<(Language, String), String>>,
}

pub struct Translation {
    lang: Language,
    key: String,
    contents: String,
}

impl Translation {
    pub fn new(
        lang: impl Into<Language>,
        key: impl Into<String>,
        contents: impl Into<String>,
    ) -> Self {
        Self {
            lang: lang.into(),
            key: key.into(),
            contents: contents.into(),
        }
    }
}

impl GetTranslation {
    pub fn new(translations: Vec<Translation>) -> Self {
        let mut inners: HashMap<(Language, String), String> =
            HashMap::with_capacity(translations.len());
        for translation in translations {
            inners.insert((translation.lang, translation.key), translation.contents);
        }
        Self {
            translations: Arc::new(inners),
        }
    }
}

impl tera::Function for GetTranslation {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let lang_str = args
            .get("lang")
            .ok_or(tera::Error::msg("Missing lang argument to gettrans()"))?;
        let key = args
            .get("key")
            .ok_or(tera::Error::msg("Missing key argument to gettrans()"))?;
        let Value::String(lang_str) = lang_str else {
            return Err(tera::Error::msg(
                "Lang argument to gettrans() was not a string",
            ));
        };
        let lang = Language::from_lang_code(lang_str).unwrap_or_default();
        let Value::String(key) = key else {
            return Err(tera::Error::msg(
                "Key argument to gettrans() was not a string",
            ));
        };
        let Some(translation) = self.translations.get(&(lang, key.clone())) else {
            return Err(tera::Error::msg(format!(
                "Translation `{key}` for `{}` does not exist!",
                lang.lang_code()
            )));
        };
        Ok(Value::String(translation.clone()))
    }

    fn is_safe(&self) -> bool {
        false
    }
}

pub fn get_translations() -> Vec<Translation> {
    trace!("Reading translations");
    let files = std::fs::read_dir("./translations/")
        .expect("Failed to open ./translations/")
        .collect::<Result<Vec<std::fs::DirEntry>, std::io::Error>>()
        .expect("Failed to read ./translations/");
    let mut translations: Vec<Translation> = Vec::with_capacity(files.len());
    for file in files {
        let file_name = file.path();
        if !file_name
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("lang"))
        {
            continue;
        }
        let lang_string = file_name
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        let lang = Language::from_lang_code(&lang_string).unwrap_or_default();
        let file_contents = std::fs::read(file.path()).unwrap_or_else(|source| {
            panic!("Failed to open file {} ({})", file.path().display(), source)
        });
        let translations_for_lang: HashMap<String, String> = serde_json::from_slice(&file_contents)
            .unwrap_or_else(|source| {
                panic!(
                    "Failed to deserialize file {} ({})",
                    file.path().display(),
                    source
                )
            });
        for (key, contents) in translations_for_lang {
            let translation = Translation::new(lang, key, contents);
            translations.push(translation);
        }
    }
    trace!("Read and parsed translations");
    translations
}
