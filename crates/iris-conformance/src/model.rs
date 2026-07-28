use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::json::{self, Value};

#[derive(Clone, Debug)]
pub struct Corpus(pub PathBuf);

#[derive(Clone, Debug)]
pub struct Record {
    pub id: String,
    pub source: String,
    pub expect: String,
    pub tags: Vec<String>,
}

impl Corpus {
    pub fn workspace() -> Result<Self, String> {
        Ok(Self(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        ))
    }
    pub fn records(&self) -> Result<Vec<Record>, String> {
        let directory = self.0.join("conformance/iris-v1/vectors/GRAMMAR");
        let mut paths = fs::read_dir(&directory)
            .map_err(|error| error.to_string())?
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(|error| error.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort();
        paths.iter().map(|path| load(path)).collect()
    }
}

fn load(path: &Path) -> Result<Record, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let parsed = json::parse(&text)?;
    let root = object(&parsed)?;
    let required = [
        "schema_version",
        "id",
        "name",
        "category",
        "source",
        "input",
        "applicability",
        "expect",
        "tags",
    ];
    if root.len() != required.len() || required.iter().any(|key| !root.contains_key(*key)) {
        return Err(format!("{}: invalid schema fields", path.display()));
    }
    if string(root, "schema_version")? != "iris-v1-vector-schema-1" {
        return Err(format!("{}: invalid schema version", path.display()));
    }
    let input = object(root.get("input").ok_or("input missing")?)?;
    let tags = array(root.get("tags").ok_or("tags missing")?)?
        .iter()
        .map(value_string)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Record {
        id: string(root, "id")?.into(),
        source: string(input, "source_text")?.into(),
        expect: render(root.get("expect").ok_or("expect missing")?),
        tags,
    })
}

pub fn parse_expect(input: &str) -> Result<Value, String> {
    json::parse(input)
}
pub fn object(value: &Value) -> Result<&std::collections::BTreeMap<String, Value>, String> {
    match value {
        Value::Object(value) => Ok(value),
        _ => Err("JSON object expected".into()),
    }
}
pub fn array(value: &Value) -> Result<&[Value], String> {
    match value {
        Value::Array(value) => Ok(value),
        _ => Err("JSON array expected".into()),
    }
}
pub fn string<'a>(
    value: &'a std::collections::BTreeMap<String, Value>,
    key: &str,
) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(|value| match value {
            Value::String(text) => Some(text.as_str()),
            _ => None,
        })
        .ok_or_else(|| format!("string field {key} required"))
}
pub fn value_string(value: &Value) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        _ => Err("JSON string expected".into()),
    }
}
pub fn render(value: &Value) -> String {
    match value {
        Value::Array(values) => format!(
            "[{}]",
            values.iter().map(render).collect::<Vec<_>>().join(",")
        ),
        Value::Bool => "true".into(),
        Value::Null => "null".into(),
        Value::Number => "0".into(),
        Value::Object(values) => format!(
            "{{{}}}",
            values
                .iter()
                .map(|(key, value)| format!("{}:{}", quote(key), render(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::String(text) => quote(text),
    }
}
fn quote(text: &str) -> String {
    format!(
        "\"{}\"",
        text.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    )
}
