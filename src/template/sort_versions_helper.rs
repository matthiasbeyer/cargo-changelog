use crate::command::VersionData;
use handlebars::handlebars_helper;

handlebars_helper!(sort_versions: |args: Vec<VersionData>| {
    let mut args = args;
    args.sort_by(|a, b| a.version().cmp(b.version()));
    serde_json::to_value(args).unwrap() // handlebars deserializes this for us, so we can serialize
                                        // it back without issue
                                        // TODO: Make this helper nice
});
