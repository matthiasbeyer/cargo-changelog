use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use crate::error::FragmentError;
use crate::format::Format;

#[derive(
    Clone, Debug, getset::Getters, getset::MutGetters, serde::Deserialize, serde::Serialize,
)]
pub struct Fragment {
    #[getset(get = "pub", get_mut = "pub")]
    header: HashMap<String, FragmentData>,
    #[getset(get = "pub")]
    text: String,
}

impl Fragment {
    pub fn empty() -> Self {
        Fragment {
            header: HashMap::new(),
            text: String::new(),
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, FragmentError> {
        let format;
        let mut buf = String::new();

        reader.read_to_string(&mut buf)?;

        let mut lines = buf.lines();
        if let Some(header_sep) = lines.next() {
            format = if header_sep == "---" {
                Format::Yaml
            } else if header_sep == "+++" {
                Format::Toml
            } else {
                return Err(FragmentError::ExpectedSeperator(header_sep.to_string()));
            }
        } else {
            return Err(FragmentError::HeaderSeperatorMissing);
        }

        let header = {
            let mut header = Vec::new();
            while let Some(line) = lines.next() {
                if line == "---" || line == "+++" {
                    break;
                }
                header.push(line);
            }

            match format {
                Format::Yaml => {
                    serde_yaml::from_str::<HashMap<String, FragmentData>>(&header.join("\n"))?
                }
                Format::Toml => {
                    toml::from_str::<HashMap<String, FragmentData>>(&header.join("\n"))?
                }
            }
        };

        let text = lines.collect::<String>();

        Ok(Fragment { header, text })
    }

    pub fn write_to<W: Write>(&self, writer: &mut W, format: Format) -> Result<(), FragmentError> {
        let (seperator, header) = match format {
            Format::Yaml => {
                let header = serde_yaml::to_string(&self.header)?;

                ("---", header)
            }
            Format::Toml => {
                let header = toml::to_string(&self.header)?;
                ("+++", header)
            }
        };

        writeln!(writer, "{}", seperator)?;
        writeln!(writer, "{}", header)?;
        writeln!(writer, "{}", seperator)?;
        writeln!(writer, "{}", self.text)?;
        Ok(())
    }

    #[cfg(test)]
    pub fn new(header: HashMap<String, FragmentData>, text: String) -> Self {
        Self { header, text }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum FragmentData {
    Bool(bool),
    Int(u64),
    Str(String),
}

impl FragmentData {
    pub fn display(&self) -> FragmentDataDisplay<'_> {
        FragmentDataDisplay(self)
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            FragmentData::Bool(_) => "bool",
            FragmentData::Int(_) => "int",
            FragmentData::Str(_) => "string",
        }
    }

    pub fn parse(s: &str) -> Result<Self, FragmentError> {
        use std::str::FromStr;

        let s_lower = s.to_lowercase();

        if s_lower == "true" {
            Ok(FragmentData::Bool(true))
        } else if s == "false" {
            Ok(FragmentData::Bool(false))
        } else if let Ok(u) = u64::from_str(s) {
            Ok(FragmentData::Int(u))
        } else {
            Ok(FragmentData::Str(s.to_string()))
        }
    }
}

pub struct FragmentDataDisplay<'a>(&'a FragmentData);

impl<'a> std::fmt::Display for FragmentDataDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            FragmentData::Bool(b) => write!(f, "{}", b),
            FragmentData::Int(i) => write!(f, "{}", i),
            FragmentData::Str(s) => write!(f, "{}", s),
        }
    }
}

/// Something that describes a FragmentData
#[derive(
    Clone, Debug, serde::Deserialize, serde::Serialize, getset::Getters, getset::CopyGetters,
)]
pub struct FragmentDataDesc {
    #[serde(rename = "type")]
    #[getset(get = "pub")]
    fragment_type: FragmentDataType,
    #[getset(get = "pub")]
    default_value: Option<FragmentData>,
    #[getset(get_copy = "pub")]
    required: bool,
    #[getset(get = "pub")]
    crawler: Option<Crawler>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum FragmentDataType {
    Ty(FragmentDataTypeDefinite),
    OneOf(Vec<String>),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum FragmentDataTypeDefinite {
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "string")]
    Str,
}

impl FragmentDataType {
    pub fn type_name(&self) -> String {
        match self {
            FragmentDataType::Ty(FragmentDataTypeDefinite::Bool) => "bool".to_string(),
            FragmentDataType::Ty(FragmentDataTypeDefinite::Int) => "int".to_string(),
            FragmentDataType::Ty(FragmentDataTypeDefinite::Str) => "string".to_string(),
            FragmentDataType::OneOf(list) => {
                let list = list
                    .iter()
                    .map(|el| format!("'{}'", el))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("one of: {}", list)
            }
        }
    }

    pub fn matches(&self, data: &FragmentData) -> bool {
        match (self, data) {
            (FragmentDataType::Ty(FragmentDataTypeDefinite::Bool), FragmentData::Bool(_)) => true,
            (FragmentDataType::Ty(FragmentDataTypeDefinite::Int), FragmentData::Int(_)) => true,
            (FragmentDataType::Ty(FragmentDataTypeDefinite::Str), FragmentData::Str(_)) => true,
            (FragmentDataType::OneOf(possible_values), FragmentData::Str(s)) => {
                possible_values.contains(s)
            }
            (_, _) => false,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "lowercase")]
pub enum Crawler {
    Path(PathBuf),
    Command(String),
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn read_empty_fragment() {
        let s = indoc::indoc!(
            r#"---
        ---
        "#
        );

        let f = Fragment::from_reader(&mut Cursor::new(s));
        assert!(f.is_ok(), "Not ok: {:?}", f);
        let f = f.unwrap();
        assert!(f.text().is_empty());
        assert!(f.header().is_empty());
    }

    #[test]
    fn read_empty_header() {
        let s = indoc::indoc!(
            r#"---
        ---
        This is some text
        "#
        );

        let f = Fragment::from_reader(&mut Cursor::new(s));
        assert!(f.is_ok(), "Not ok: {:?}", f);
        let f = f.unwrap();
        assert_eq!(f.text(), "This is some text");
        assert!(f.header().is_empty());
    }

    #[test]
    fn read_empty_content() {
        let s = indoc::indoc!(
            r#"---
        foo: bar
        ---
        "#
        );

        let f = Fragment::from_reader(&mut Cursor::new(s));
        assert!(f.is_ok(), "Not ok: {:?}", f);
        let f = f.unwrap();
        assert!(f.text().is_empty(), "Not empty: '{}'", f.text());
        assert!(
            f.header().contains_key("foo"),
            "'foo' key missing from header: {:?}",
            f.header()
        );
        assert!(
            std::matches!(f.header().get("foo").unwrap(), FragmentData::Str(_)),
            "'foo' key does not point to String: {:?}",
            f.header()
        );

        let foo = match f.header().get("foo").unwrap() {
            FragmentData::Str(s) => s,
            other => panic!("Expected String, found: {:?}", other),
        };

        assert_eq!(
            foo,
            "bar",
            "'foo' key content is not 'bar': {:?}",
            f.header()
        );
    }

    #[test]
    fn read_toml_header() {
        let s = indoc::indoc!(
            r#"+++
        foo = "bar"
        +++
        "#
        );

        let f = Fragment::from_reader(&mut Cursor::new(s));
        assert!(f.is_ok(), "Not ok: {:?}", f);
        let f = f.unwrap();
        assert!(f.text().is_empty(), "Not empty: '{}'", f.text());
        assert!(
            f.header().contains_key("foo"),
            "'foo' key missing from header: {:?}",
            f.header()
        );
        assert!(
            std::matches!(f.header().get("foo").unwrap(), FragmentData::Str(_)),
            "'foo' key does not point to String: {:?}",
            f.header()
        );

        let foo = match f.header().get("foo").unwrap() {
            FragmentData::Str(s) => s,
            other => panic!("Expected String, found: {:?}", other),
        };

        assert_eq!(
            foo,
            "bar",
            "'foo' key content is not 'bar': {:?}",
            f.header()
        );
    }

    #[test]
    fn test_write_to_yaml() {
        let mut buffer = std::io::Cursor::new(Vec::with_capacity(1024));
        let mut header = HashMap::new();
        header.insert("foo".to_string(), FragmentData::Bool(true));
        header.insert("bar".to_string(), FragmentData::Str(String::from("baz")));

        let frag = Fragment::new(header, String::from("testtext"));
        let res = frag.write_to(&mut buffer, Format::Yaml);
        assert!(res.is_ok(), "Error writing: {}", res.unwrap_err());

        let buffer = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(buffer.contains("---\n"));
        assert!(buffer.contains("foo: true\n"));
        assert!(buffer.contains("bar: baz\n"));
        assert!(buffer.contains("\n---\ntesttext\n"));
    }

    #[test]
    fn test_deserializing_data_desc_with_one_of() {
        let s = r#"
            required = false
            type = ["foo", "bar"]
        "#;

        let f: Result<FragmentDataDesc, _> = toml::from_str(s);
        assert!(
            f.is_ok(),
            "Not ok: {}, should look like: {}",
            f.unwrap_err(),
            {
                let fdd = FragmentDataDesc {
                    fragment_type: FragmentDataType::OneOf(vec![
                        "foo".to_string(),
                        "bar".to_string(),
                    ]),
                    required: false,
                    default_value: None,
                    crawler: None,
                };
                toml::to_string(&fdd).unwrap()
            }
        );
    }
}
