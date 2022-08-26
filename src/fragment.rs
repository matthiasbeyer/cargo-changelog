use std::collections::HashMap;

pub struct Fragment {
    text: String,

    data: Vec<FragmentData>,
}

#[derive(Debug, serde::Deserialize)]
pub enum FragmentData {
    Bool(bool),
    Int(u64),
    String(String),
    Map(HashMap<String, FragmentData>)
}
