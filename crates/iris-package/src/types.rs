use crate::{PackageError, error::invalid};
use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! validated_string {
    ($name:ident, $validator:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = PackageError;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                $validator(&value)?;
                Ok(Self(value))
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

validated_string!(PackageId, package_id);
validated_string!(RelativePath, relative_path);
validated_string!(GitRevision, git_revision);
validated_string!(Sha256Digest, sha256_digest);
validated_string!(GitUrl, git_url);

fn package_id(value: &str) -> Result<(), PackageError> {
    if !value.contains('.')
        || !value.split('.').all(|part| {
            part.starts_with(|character: char| character.is_ascii_lowercase())
                && part.ends_with(|character: char| character.is_ascii_alphanumeric())
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
    {
        return Err(invalid("package_id", value));
    }
    Ok(())
}

fn relative_path(value: &str) -> Result<(), PackageError> {
    if value.is_empty()
        || value.contains(['\\', ':', '\0'])
        || value.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.eq_ignore_ascii_case(".git")
                || part.eq_ignore_ascii_case(".iris")
                || part.ends_with(['.', ' '])
                || part.chars().any(char::is_control)
                || matches!(
                    part.split('.')
                        .next()
                        .unwrap_or_default()
                        .to_ascii_uppercase()
                        .as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "COM4"
                        | "COM5"
                        | "COM6"
                        | "COM7"
                        | "COM8"
                        | "COM9"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                        | "LPT4"
                        | "LPT5"
                        | "LPT6"
                        | "LPT7"
                        | "LPT8"
                        | "LPT9"
                )
        })
    {
        return Err(invalid("relative path", value));
    }
    Ok(())
}

fn git_revision(value: &str) -> Result<(), PackageError> {
    if !matches!(value.len(), 40 | 64)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(invalid("full Git revision", value));
    }
    Ok(())
}

fn sha256_digest(value: &str) -> Result<(), PackageError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(invalid("SHA256", value));
    }
    Ok(())
}

fn git_url(value: &str) -> Result<(), PackageError> {
    let parsed = url::Url::parse(value).map_err(|_| invalid("Git URL", "rejected URL"))?;
    if value.chars().any(char::is_control)
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !parsed.username().is_empty()
        || parsed.path().is_empty()
        || !matches!(parsed.scheme(), "https" | "ssh" | "file")
        || (parsed.scheme() != "file" && parsed.host_str().is_none())
        || (parsed.scheme() == "file" && parsed.to_file_path().is_err())
    {
        return Err(invalid("credential-free HTTPS/SSH Git URL", "rejected URL"));
    }
    Ok(())
}

impl GitUrl {
    pub fn is_local(&self) -> bool {
        self.0.starts_with("file:")
    }
}

impl Sha256Digest {
    pub(crate) const fn from_hash(value: String) -> Self {
        Self(value)
    }
}
