use crate::NativeError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub schema_version: u32,
    pub package_id: String,
    pub version: String,
    pub api_major: u32,
    pub iris_major: u32,
    pub abi: AbiRequirement,
    pub permissions: Permissions,
    pub reflection_policy: String,
    pub modules: Vec<ModuleMetadata>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbiRequirement {
    pub major: u32,
    pub minimum_minor: u32,
    pub required_features: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Permissions {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleMetadata {
    pub name: String,
    pub functions: Vec<FunctionMetadata>,
    pub resources: Vec<ResourceMetadata>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionMetadata {
    pub name: String,
    pub parameters: Vec<ParameterMetadata>,
    pub returns: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterMetadata {
    pub name: String,
    #[serde(rename = "type")]
    pub value_type: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceMetadata {
    pub name: String,
    pub contracts: Vec<String>,
    pub managed_roots: bool,
    pub storage: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeType {
    Nil,
    Bool,
    Integer,
    String,
    Bytes,
    Resource(String),
}
impl NativeType {
    pub fn parse(name: &str) -> Result<Self, NativeError> {
        Ok(match name {
            "Nil" => Self::Nil,
            "Bool" => Self::Bool,
            "Integer" => Self::Integer,
            "String" => Self::String,
            "Bytes" => Self::Bytes,
            name => Self::Resource(
                name.strip_prefix("resource:")
                    .filter(|name| identifier(name))
                    .ok_or(NativeError::Metadata)?
                    .to_owned(),
            ),
        })
    }
}
fn identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}
impl Metadata {
    pub(crate) fn validate(&self) -> Result<(), NativeError> {
        if self.schema_version != 1
            || self.iris_major != 1
            || self.api_major == 0
            || self.abi.major != 1
            || self.abi.minimum_minor != 0
            || self.abi.required_features != 0
            || self.reflection_policy != "deny"
            || self.package_id.is_empty()
            || self.version.is_empty()
        {
            return Err(NativeError::Metadata);
        }
        let mut modules = BTreeSet::new();
        for module in &self.modules {
            if !module.name.split("::").all(identifier) || !modules.insert(&module.name) {
                return Err(NativeError::Metadata);
            }
            let mut resources = BTreeSet::new();
            for resource in &module.resources {
                if !identifier(&resource.name)
                    || !resources.insert(&resource.name)
                    || resource.contracts != ["Closeable"]
                    || resource.managed_roots
                    || resource.storage != "external-cookie-v1"
                {
                    return Err(NativeError::Metadata);
                }
            }
            let mut functions = BTreeSet::new();
            for function in &module.functions {
                if !identifier(&function.name) || !functions.insert(&function.name) {
                    return Err(NativeError::Metadata);
                }
                let mut parameters = BTreeSet::new();
                for parameter in &function.parameters {
                    if !identifier(&parameter.name) || !parameters.insert(&parameter.name) {
                        return Err(NativeError::Metadata);
                    }
                }
                for name in function
                    .parameters
                    .iter()
                    .map(|parameter| &parameter.value_type)
                    .chain(std::iter::once(&function.returns))
                {
                    if let NativeType::Resource(name) = NativeType::parse(name)?
                        && !resources.contains(&name)
                    {
                        return Err(NativeError::Metadata);
                    }
                }
            }
        }
        Ok(())
    }
}
