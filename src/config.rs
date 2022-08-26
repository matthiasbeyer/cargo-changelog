use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
pub struct Configuration {
    changelog_header: String,
    version_prefix: String,
    add_version_date: bool,

    entry_template: PathBuf,
    entry_data: Vec<FragmentDataDescription>,

    group_by: Option<FragmentDataDescriptionName>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(transparent)]
pub struct FragmentDataDescriptionName(String);

#[derive(Debug, serde::Deserialize)]
pub struct FragmentDataDescription {
    key: FragmentDataDescriptionName,
    required: bool,
    default_value: Option<String>,
    value: FragmentDataValueType,
}

#[derive(Debug, serde::Deserialize)]
pub enum FragmentDataValueType {
    Bool,
    Int,
    String,
    Map(HashMap<String, FragmentDataDescription>),
}

