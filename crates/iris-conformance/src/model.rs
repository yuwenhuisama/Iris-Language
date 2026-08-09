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
    pub independent_sources: Vec<String>,
    /// Ordered `(package_id, source)` programs sharing ONE runtime.
    ///
    /// `D-431` makes a global's identity `(package_id, $name)` with no flat
    /// cross-package namespace, which cannot be observed from a single program
    /// and is NOT the same as `independent_sources`, whose programs each run
    /// against a fresh runtime.
    pub package_sources: Vec<(String, String)>,
    /// The `fixtures/meta/...` directory this record loads, when it names one.
    ///
    /// Chapter 08 vectors name a concrete on-disk package tree rather than
    /// carrying their program inline, which `IRIS-V1-META-C003` requires of a
    /// publishable package.
    pub package_fixture: Option<String>,
    /// An expression evaluated AFTER the package loads.
    ///
    /// A package source file is declarations only under `IRIS-V1-META-C011`,
    /// so a row observing a Module member needs a send made after the load
    /// rather than a trailing statement inside the package.
    pub package_probe: Option<String>,
    pub expect: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Chapter {
    Grammar,
    Runtime,
    Control,
    Types,
    Meta,
    /// The async chapter, whose rows observe Task and await behaviour.
    Async,
    /// The collections chapter.
    Collections,
    /// The native host and FFI chapter.
    Ffi,
    /// The language identity chapter.
    Identity,
    /// The serialization and standard library chapter.
    Library,
}

impl Chapter {
    pub const fn directory(self) -> &'static str {
        match self {
            Self::Grammar => "GRAMMAR",
            Self::Runtime => "RUNTIME",
            Self::Control => "CONTROL",
            Self::Types => "TYPES",
            Self::Meta => "META",
            Self::Async => "ASYNC",
            Self::Collections => "COLLECTIONS",
            Self::Ffi => "FFI",
            Self::Identity => "IDENTITY",
            Self::Library => "LIBRARY",
        }
    }
}

impl Corpus {
    pub fn workspace() -> Result<Self, String> {
        Ok(Self(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        ))
    }
    pub fn records(&self) -> Result<Vec<Record>, String> {
        self.records_for(Chapter::Grammar)
    }
    pub fn runtime_records(&self) -> Result<Vec<Record>, String> {
        self.records_for(Chapter::Runtime)
    }
    pub fn records_for(&self, chapter: Chapter) -> Result<Vec<Record>, String> {
        let directory = self
            .0
            .join("conformance/iris-v1/vectors")
            .join(chapter.directory());
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
        source: input_source(input)?,
        independent_sources: input_independent_sources(input)?,
        package_sources: input_package_sources(input)?,
        package_fixture: match input.get("package_fixture") {
            Some(Value::String(path)) => Some(path.clone()),
            Some(_) => return Err("string field package_fixture required".into()),
            None => None,
        },
        package_probe: match input.get("package_probe") {
            Some(Value::String(probe)) => Some(probe.clone()),
            Some(_) => return Err("string field package_probe required".into()),
            None => None,
        },
        expect: render(root.get("expect").ok_or("expect missing")?),
        tags,
    })
}

fn input_independent_sources(
    input: &std::collections::BTreeMap<String, Value>,
) -> Result<Vec<String>, String> {
    match input.get("independent_sources") {
        Some(Value::Array(sources)) => sources.iter().map(value_string).collect(),
        Some(_) => Err("array field independent_sources required".into()),
        None => Ok(Vec::new()),
    }
}

fn input_package_sources(
    input: &std::collections::BTreeMap<String, Value>,
) -> Result<Vec<(String, String)>, String> {
    let Some(value) = input.get("package_sources") else {
        return Ok(Vec::new());
    };
    let Value::Array(entries) = value else {
        return Err("array field package_sources required".into());
    };
    entries
        .iter()
        .map(|entry| {
            let entry = object(entry)?;
            Ok((
                string(entry, "package_id")?.to_owned(),
                string(entry, "source_text")?.to_owned(),
            ))
        })
        .collect()
}

fn input_source(input: &std::collections::BTreeMap<String, Value>) -> Result<String, String> {
    match input.get("source_text") {
        Some(Value::String(source)) => Ok(source.clone()),
        Some(_) => Err("string field source_text required".into()),
        // Independent sources, package sources and an on-disk package fixture
        // each carry their own programs, so none needs a single `source_text`
        // or a prose `fixture_ref`.
        None => match input
            .get("independent_sources")
            .or_else(|| input.get("package_sources"))
            .or_else(|| input.get("package_fixture"))
        {
            Some(_) => Ok(String::new()),
            None => string(input, "fixture_ref").map(str::to_owned),
        },
    }
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
        Value::Bool(value) => value.to_string(),
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
