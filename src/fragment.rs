use std::collections::HashMap;
use std::io::Read;
use std::io::Write;

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
    List(Vec<FragmentData>),
    Map(HashMap<String, FragmentData>),
}

impl FragmentData {
    pub fn type_name(&self) -> &'static str {
        match self {
            FragmentData::Bool(_) => "bool",
            FragmentData::Int(_) => "int",
            FragmentData::Str(_) => "string",
            FragmentData::List(_) => "list",
            FragmentData::Map(_) => "map",
        }
    }
}

/// Something that describes a FragmentData
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, getset::Getters)]
pub struct FragmentDataDesc {
    #[serde(rename = "type")]
    #[getset(get = "pub")]
    fragment_type: FragmentDataType,
    #[getset(get = "pub")]
    default_value: Option<FragmentData>,
    required: bool,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum FragmentDataType {
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "string")]
    Str,
    List(Box<FragmentDataType>),
    Map(HashMap<String, FragmentDataType>),
}

impl FragmentDataType {
    pub fn type_name(&self) -> String {
        match self {
            FragmentDataType::Bool => "bool".to_string(),
            FragmentDataType::Int => "int".to_string(),
            FragmentDataType::Str => "string".to_string(),
            FragmentDataType::List(inner) => format!("list<{}>", inner.type_name()),
            FragmentDataType::Map(_) => "map".to_string(),
        }
    }

    pub fn matches(&self, data: &FragmentData) -> bool {
        match (self, data) {
            (FragmentDataType::Bool, FragmentData::Bool(_)) => true,
            (FragmentDataType::Int, FragmentData::Int(_)) => true,
            (FragmentDataType::Str, FragmentData::Str(_)) => true,
            (FragmentDataType::List(_t_inner), FragmentData::List(_d_inner)) => {
                unimplemented!()
            }
            (FragmentDataType::Map(_t_inner), FragmentData::Map(_d_inner)) => {
                unimplemented!()
            }
            (_, _) => false,
        }
    }
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
        assert!(f.text().is_empty());
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
        assert!(f.text().is_empty());
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
}
