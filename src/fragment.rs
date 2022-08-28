use std::collections::HashMap;
use std::io::Read;
use std::io::Write;

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
        unimplemented!()
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> miette::Result<()> {
        unimplemented!()
    }
}

#[derive(Debug, serde::Serialize)]
pub enum FragmentData {
    Bool(bool),
    Int(u64),
    String(String),
    List(Vec<FragmentData>),
    Map(HashMap<String, FragmentData>),
}
