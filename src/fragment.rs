use std::collections::HashMap;
use std::io::Read;
use std::io::Write;

use miette::IntoDiagnostic;

use crate::error::Error;

#[derive(Debug, getset::Getters)]
pub struct Fragment {
    #[getset(get = "pub")]
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

    pub fn from_reader<R: Read>(reader: &mut R) -> miette::Result<Self> {
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(Error::from)
            .into_diagnostic()?;

        let mut lines = buf.lines();
        if let Some(header_sep) = lines.next() {
            if header_sep != "---" {
                miette::bail!("Expected header seperator: '---', found: '{}'", header_sep)
            }
        } else {
            miette::bail!("Header seperator '---' missing")
        }

        let header = {
            let mut header = Vec::new();
            while let Some(line) = lines.next() {
                if line == "---" {
                    break;
                }
                header.push(line);
            }
            serde_yaml::from_str::<HashMap<String, FragmentData>>(&header.join("\n"))
                .map_err(Error::from)
                .into_diagnostic()?
        };

        let text = lines.collect::<String>();

        Ok(Fragment { header, text })
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> miette::Result<()> {
        unimplemented!()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum FragmentData {
    Bool(bool),
    Int(u64),
    Str(String),
    List(Vec<FragmentData>),
    Map(HashMap<String, FragmentData>),
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
}
