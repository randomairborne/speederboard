use std::{collections::HashMap, fmt::Write, sync::Arc};

use markdown::mdast::Node;
use simpleinterpolation::Interpolation;
use tera::Value;

use crate::{config::Config, language::Language, Error};

pub struct GetTranslation {
    translations: Arc<HashMap<(Language, String), Interpolation>>,
}

#[derive(Debug, Clone)]
pub struct Translation {
    lang: Language,
    key: String,
    contents: Interpolation,
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
            contents: Interpolation::new(contents.into()).unwrap(),
        }
    }
}

impl GetTranslation {
    pub fn new(translations: Vec<Translation>) -> Self {
        let mut inners: HashMap<(Language, String), Interpolation> =
            HashMap::with_capacity(translations.len());
        for translation in translations {
            if inners
                .insert(
                    (translation.lang, translation.key.clone()),
                    translation.contents,
                )
                .is_some()
            {
                panic!(
                    "Translation collision: ({:?}, {})",
                    translation.lang, translation.key
                );
            }
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
        if let Some(translation) = self.translations.get(&(lang, key.clone())) {
            Ok(Value::String(
                translation.render_transform(args, stringify_value),
            ))
        } else {
            warn!(
                code = lang.lang_code(),
                key = key,
                "Translation does not exist!",
            );
            let default_lang = Language::default();
            if let Some(en_translation) = self.translations.get(&(default_lang, key.clone())) {
                Ok(Value::String(
                    en_translation.render_transform(args, stringify_value),
                ))
            } else {
                error!(
                    code = lang.lang_code(),
                    fallback_code = default_lang.lang_code(),
                    key = key,
                    "Translation does not exist, and neither does fallback!",
                );
                Err(tera::Error::msg(format!(
                    "Translation `{key}` does not exist for `{}` or fallback `{}`",
                    lang.lang_code(),
                    default_lang.lang_code()
                )))
            }
        }
    }

    fn is_safe(&self) -> bool {
        true
    }
}

fn stringify_value(value: &Value) -> String {
    match value {
        Value::Null => "nil".to_owned(),
        Value::Bool(val) => val.to_string(),
        Value::Number(val) => val.to_string(),
        Value::String(string) => tera::escape_html(string),
        Value::Array(val) => tera::escape_html(&format!("{val:?}")),
        Value::Object(val) => tera::escape_html(&format!("{val:?}")),
    }
}

#[derive(serde::Deserialize, Clone)]
#[serde(untagged)]
enum TranslationLeaf {
    Leaf(String),
    Branch(HashMap<String, TranslationLeaf>),
}

pub fn get_translations(config: &Config) -> Result<Vec<Translation>, Error> {
    let translation_path = config.translation_dir.trim_end_matches('/');
    trace!("Reading translations");
    let files = std::fs::read_dir(translation_path)
        .expect("Failed to open translation directory")
        .collect::<Result<Vec<std::fs::DirEntry>, std::io::Error>>()
        .expect("Failed to read translation directory");
    let mut translations: Vec<Translation> = Vec::new();
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
            .ok_or(Error::NoFileStem)?
            .to_os_string()
            .into_string()
            .map_err(|_| Error::InvalidOsString)?;
        let lang = Language::from_lang_code(&lang_string).unwrap_or_default();
        let file_contents = std::fs::read(file.path())?;
        let translations_for_lang: HashMap<String, TranslationLeaf> =
            serde_json::from_slice(&file_contents)?;
        let mut flattened_translations = flatten_translations(lang, None, translations_for_lang);
        translations.append(&mut flattened_translations);
    }
    debug!("Read and parsed translations");
    trace!(?translations, "got translations");
    Ok(translations)
}

fn flatten_translations(
    lang: Language,
    prefix: Option<&str>,
    map: HashMap<String, TranslationLeaf>,
) -> Vec<Translation> {
    let mut translations: Vec<Translation> = Vec::new();
    for (key, value) in map.into_iter() {
        let trans_key = if let Some(prefix) = prefix {
            format!("{prefix}.{key}")
        } else {
            key
        };
        match value {
            TranslationLeaf::Leaf(contents) => {
                let contents =
                    convert_markdown_to_html(contents).expect("Failed to convert markdown to HTML");
                let translation = Translation::new(lang, trans_key, contents);
                translations.push(translation);
            }
            TranslationLeaf::Branch(inner_map) => {
                let mut inner_translations =
                    flatten_translations(lang, Some(&trans_key), inner_map);
                translations.append(&mut inner_translations);
            }
        };
    }
    translations
}

fn convert_markdown_to_html(contents: String) -> Result<String, Error> {
    let ast = markdown::to_mdast(&contents, &Default::default())
        .expect("Markdown compile errors should be impossible");

    let mut translated = String::with_capacity(1024);
    node_to_string(&mut translated, ast)?;
    Ok(translated)
}

fn node_to_string(txt: &mut String, node: Node) -> std::fmt::Result {
    match node {
        Node::Root(v) => nodes_to_string(txt, v.children)?,
        Node::BlockQuote(v) => nodes_to_string(txt, v.children)?,
        Node::FootnoteDefinition(v) => nodes_to_string(txt, v.children)?,
        Node::MdxJsxFlowElement(v) => nodes_to_string(txt, v.children)?,
        Node::List(v) => nodes_to_string(txt, v.children)?,
        Node::MdxjsEsm(v) => txt.write_str(&v.value)?,
        Node::Toml(v) => txt.write_str(&v.value)?,
        Node::Yaml(v) => txt.write_str(&v.value)?,
        Node::Break(_) => txt.write_str("<br>")?,
        Node::InlineCode(v) => txt.write_str(&v.value)?,
        Node::InlineMath(v) => txt.write_str(&v.value)?,
        Node::Delete(v) => surround_nodes_with_tag(txt, "s", v.children)?,
        Node::Emphasis(v) => surround_nodes_with_tag(txt, "i", v.children)?,
        Node::MdxTextExpression(v) => txt.write_str(&v.value)?,
        Node::FootnoteReference(v) => txt.write_str(&v.identifier)?,
        Node::Html(v) => txt.write_str(&tera::escape_html(&v.value))?,
        Node::Image(v) => write!(txt, "<img src=\"{}\" alt=\"{}\" />", v.url, v.alt)?,
        Node::ImageReference(v) => txt.write_str(&v.alt)?,
        Node::MdxJsxTextElement(v) => nodes_to_string(txt, v.children)?,
        Node::Link(v) => write!(
            txt,
            "<a href=\"{}\">{}</a>",
            v.url,
            children_to_string(v.children)?
        )?,
        Node::LinkReference(v) => txt.write_str(&v.identifier)?,
        Node::Strong(v) => surround_nodes_with_tag(txt, "strong", v.children)?,
        Node::Text(v) => txt.push_str(&v.value),
        Node::Code(v) => surround_str_with_tag(txt, "code", &v.value)?,
        Node::Math(v) => txt.push_str(&v.value),
        Node::MdxFlowExpression(v) => txt.push_str(&v.value),
        Node::Heading(v) => write!(
            txt,
            "<h{0}>{1}</h{0}",
            v.depth,
            &children_to_string(v.children)?
        )?,
        Node::Table(v) => nodes_to_string(txt, v.children)?,
        Node::ThematicBreak(_) => txt.write_str("<hr>")?,
        Node::TableRow(v) => nodes_to_string(txt, v.children)?,
        Node::TableCell(v) => nodes_to_string(txt, v.children)?,
        Node::ListItem(v) => nodes_to_string(txt, v.children)?,
        Node::Definition(v) => write!(txt, "[{}]: {}", v.identifier, v.url)?,
        Node::Paragraph(v) => nodes_to_string(txt, v.children)?,
    }
    Ok(())
}

fn nodes_to_string(txt: &mut String, children: Vec<Node>) -> std::fmt::Result {
    for child in children {
        node_to_string(txt, child)?;
    }
    Ok(())
}

fn surround_nodes_with_tag(txt: &mut String, tag: &str, children: Vec<Node>) -> std::fmt::Result {
    write!(txt, "<{tag}>")?;
    nodes_to_string(txt, children)?;
    write!(txt, "</{tag}>")?;
    Ok(())
}

fn surround_str_with_tag(txt: &mut String, tag: &str, child: &str) -> std::fmt::Result {
    write!(txt, "<{tag}>{child}</{tag}>")?;
    Ok(())
}

fn children_to_string(nodes: Vec<Node>) -> Result<String, std::fmt::Error> {
    let mut string = String::with_capacity(1024);
    nodes_to_string(&mut string, nodes)?;
    Ok(string)
}
