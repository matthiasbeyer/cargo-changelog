use std::collections::HashMap;

#[derive(Debug, serde::Serialize)]
pub struct Fragment {
    data: HashMap<String, FragmentData>,
    text: String,
}

impl Fragment {
    pub fn empty() -> Self {
        Fragment {
            data: HashMap::new(),
            text: String::new(),
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
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
