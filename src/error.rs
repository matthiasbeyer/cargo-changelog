use std::path::PathBuf;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("IO")]
    Io(#[from] std::io::Error),

    #[error("UTF8 error")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("git error")]
    Git(#[from] git2::Error),

    #[error("Repository dirty")]
    GitRepoDirty,

    #[error("TOML deserialization error")]
    Toml(#[from] toml::de::Error),

    #[error("Time formatting error")]
    TimeFormat(#[from] time::error::Format),

    #[error("Error getting cargo meta information")]
    Cargo(#[from] cargo_metadata::Error),

    #[error("Error in handlebars template")]
    HandlebarsTemplate(#[from] Box<handlebars::TemplateError>),

    #[error("Error during template rendering")]
    HandlebarsRender(#[from] handlebars::RenderError),

    #[error("Error while walking directory")]
    WalkDir(#[from] walkdir::Error),

    #[error("Repository has no worktree")]
    NoWorkTree,

    #[error(
        "Configuration file does not exist, tried {:?}",
        crate::config::CONFIG_FILE_NAMES
    )]
    ConfigDoesNotExist,

    #[error("Not a file: {0}")]
    NotAFile(PathBuf),

    #[error("No version found in Cargo.toml, that should never happen...")]
    NoVersionInCargoToml,

    #[error(
        "Versions are not all the same in the workspace, cannot decide what you want to release!"
    )]
    WorkspaceVersionsNotEqual,

    #[error("EDITOR and VISUAL are not set, cannot find editor")]
    EditorEnvNotSet,

    #[error("Environment variable '{0}' is not unicode")]
    EnvNotUnicode(String),

    #[error("Specified version '{version}' does not exist")]
    ExactVersionDoesNotExist { version: String },

    #[error(transparent)]
    SemVer(#[from] semver::Error),

    #[error("Fragment Error: {}", .1.display())]
    FragmentError(#[source] FragmentError, PathBuf),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("Version error")]
    Version(#[from] VersionError),

    #[error("Text provider error")]
    TextProvider(#[from] TextProviderError),

    #[error("Verification failed")]
    Verification(#[from] VerificationError),

    #[error("Multiple errors")]
    Multiple {
        #[related]
        errors: Vec<Error>,
    },
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum FragmentError {
    #[error("IO")]
    Io(#[from] std::io::Error),

    #[error("Expected header seperator: '+++', found: '{0}'")]
    ExpectedSeperator(String),

    #[error("Header seperator '+++' missing")]
    HeaderSeperatorMissing,

    #[error("TOML serialization error")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialization error")]
    TomlDe(#[from] toml::de::Error),

    #[error("Type Error: Expected {exp}, got {recv} for field {field_name}")]
    DataType {
        exp: String,
        recv: String,
        field_name: String,
    },

    #[error("Error during interactive session")]
    Interactive(#[from] InteractiveError),

    #[error("Required value '{}', but value is missing", .0)]
    RequiredValueMissing(String),

    #[error("Not a valid command: '{}'", .0)]
    NoValidCommand(String),

    #[error("Exited unsuccessful: '{}'", .0)]
    CommandNoSuccess(String),

    #[error("No UTF8 output from Command: '{}'", .0)]
    NoUtf8Output(String, #[source] std::string::FromUtf8Error),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum VersionError {
    #[error("UTF8 Error with path: {}", .0.display())]
    Utf8(PathBuf),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum TextProviderError {
    #[error("IO Error")]
    Io(#[from] std::io::Error),

    #[error("UTF8 Error")]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum VerificationError {
    #[error("Version error")]
    Version(#[from] VersionError),

    #[error("Error while parsing fragment {0}")]
    FragmentParsing(PathBuf, #[source] FragmentError),

    #[error("Error while walking directory")]
    WalkDir(#[from] walkdir::Error),

    #[error("Header field '{}' missing", .0)]
    RequiredHeaderFieldMissing(String),

    #[error("Header field '{}' should have type '{}', but has '{}'", .field_name, .required, .actual)]
    HeaderFieldTypesDontMatch {
        field_name: String,
        required: String,
        actual: &'static str,
    },

    #[error("Header field '{}' should be one of {}, but has type '{}'", .field_name, .possible.join(", "), .actual)]
    HeaderFieldTypesDontMatchOneOf {
        field_name: String,
        possible: Vec<String>,
        actual: &'static str,
    },

    #[error("Header field '{}' should be one of {}, but has value '{}'", .field_name, .possible.join(", "), .actual)]
    HeaderFieldHasNotPossibleValue {
        field_name: String,
        possible: Vec<String>,
        actual: String,
    },

    #[error("Multiple verification errors in {}", .fragment_path.display())]
    Multiple {
        fragment_path: PathBuf,
        #[related]
        multiple: Vec<VerificationError>,
    },
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum InteractiveError {
    #[error("User interrupted interactive session")]
    Interrupted,

    #[error("IO Error")]
    Io(#[from] std::io::Error),

    #[error("Type Error: Expected {}, got {}", .0.type_name(), .1.type_name())]
    TypeError(
        crate::fragment::FragmentDataType,
        crate::fragment::FragmentData,
    ),

    #[error("Failed to parse intefer")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Index error: Tried to select entry {}, but list has only {}", .0, .1)]
    IndexError(usize, usize),
}
