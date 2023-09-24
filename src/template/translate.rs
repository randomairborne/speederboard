use std::fmt::Display;
use std::{collections::HashMap, sync::Arc};
use tera::Value;

pub struct GetTranslation {
    translations: Arc<HashMap<String, String>>,
}

pub struct Translation {
    lang: String,
    key: String,
    contents: String,
}

impl Translation {
    pub fn new(
        lang: impl Into<String>,
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
        let mut inners: HashMap<String, String> = HashMap::with_capacity(translations.len());
        for translation in translations {
            inners.insert(
                trans_key(translation.lang, translation.key),
                translation.contents,
            );
        }
        Self {
            translations: Arc::new(inners),
        }
    }
}

impl tera::Function for GetTranslation {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let lang = args
            .get("lang")
            .ok_or(tera::Error::msg("Missing lang argument to gettrans()"))?;
        let key = args
            .get("key")
            .ok_or(tera::Error::msg("Missing key argument to gettrans()"))?;
        let Value::String(lang) = lang else {
            return Err(tera::Error::msg(
                "Lang argument to gettrans() was not a string",
            ));
        };
        let Value::String(key) = key else {
            return Err(tera::Error::msg(
                "Key argument to gettrans() was not a string",
            ));
        };
        let Some(translation) = self.translations.get(&trans_key(lang, key)).cloned() else {
            return Err(tera::Error::msg(format!(
                "Translation `{key}` for `{lang}` does not exist!"
            )));
        };
        Ok(Value::String(translation))
    }

    fn is_safe(&self) -> bool {
        false
    }
}

fn trans_key(lang: impl Display, key: impl Display) -> String {
    format!("{lang}.{key}")
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
        let lang = file_name
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
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
            let translation = Translation::new(lang.clone(), key, contents);
            translations.push(translation);
        }
    }
    trace!("Read and parsed translations");
    translations
}
