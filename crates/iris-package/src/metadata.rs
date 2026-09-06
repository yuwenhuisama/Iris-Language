use crate::{Manifest, PackageError, PackageId, Permissions, error::invalid};
use semver::Version;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataIdentity {
    schema_version: u32,
    package_id: PackageId,
    api_major: u32,
    version: Version,
    iris_major: u32,
    abi: Abi,
    permissions: Permissions,
    reflection_policy: String,
    modules: Vec<Module>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Module {
    name: String,
    functions: Vec<Function>,
    resources: Vec<Resource>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Function {
    name: String,
    parameters: Vec<Parameter>,
    returns: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameter {
    name: String,
    #[serde(rename = "type")]
    value_type: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resource {
    name: String,
    contracts: Vec<String>,
    managed_roots: bool,
    storage: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Abi {
    major: u32,
    minimum_minor: u32,
    required_features: u64,
}

pub(crate) fn verify(bytes: &[u8], manifest: &Manifest) -> Result<(), PackageError> {
    let metadata: MetadataIdentity = serde_json::from_slice(bytes)
        .map_err(|error| invalid("native metadata", error.to_string()))?;
    let native = manifest
        .native
        .as_ref()
        .ok_or_else(|| invalid("native metadata", "no native declaration"))?;
    if metadata.schema_version != 1
        || metadata.package_id != manifest.package_id
        || metadata.api_major != manifest.api_major
        || metadata.version != manifest.version
        || metadata.iris_major != manifest.iris_major
        || metadata.abi.major != native.abi_major
        || metadata.abi.minimum_minor != native.minimum_minor
        || metadata.abi.required_features != native.required_features
        || metadata.permissions != manifest.permissions
        || metadata.reflection_policy != "deny"
    {
        return Err(invalid(
            "native metadata",
            "manifest identity/ABI/permissions mismatch",
        ));
    }
    let mut modules = BTreeSet::new();
    for module in &metadata.modules {
        if !module.name.split("::").all(identifier) || !modules.insert(&module.name) {
            return Err(invalid("native metadata", "invalid or duplicate module"));
        }
        let mut resources = BTreeSet::new();
        for resource in &module.resources {
            if !identifier(&resource.name)
                || !resources.insert(resource.name.as_str())
                || resource.contracts != ["Closeable"]
                || resource.managed_roots
                || resource.storage != "external-cookie-v1"
            {
                return Err(invalid("native metadata", "invalid resource"));
            }
        }
        let mut functions = BTreeSet::new();
        for function in &module.functions {
            if !identifier(&function.name) || !functions.insert(&function.name) {
                return Err(invalid("native metadata", "invalid or duplicate function"));
            }
            let mut parameters = BTreeSet::new();
            for parameter in &function.parameters {
                if !identifier(&parameter.name) || !parameters.insert(&parameter.name) {
                    return Err(invalid("native metadata", "invalid or duplicate parameter"));
                }
            }
            for value_type in function
                .parameters
                .iter()
                .map(|parameter| &parameter.value_type)
                .chain(std::iter::once(&function.returns))
            {
                match value_type.as_str() {
                    "Nil" | "Bool" | "Integer" | "String" | "Bytes" => {}
                    name if name
                        .strip_prefix("resource:")
                        .is_some_and(|name| resources.contains(name)) => {}
                    _ => return Err(invalid("native metadata", "invalid native value type")),
                }
            }
        }
    }
    Ok(())
}

fn identifier(value: &str) -> bool {
    value.starts_with(|character: char| character.is_ascii_alphabetic() || character == '_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
