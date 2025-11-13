use crate::command::VersionData;
use handlebars::handlebars_helper;

handlebars_helper!(sort_versions: |args: Vec<VersionData>| {
    let mut args = args;
    args.sort_by(|a, b| {
        let a_sv = a.version_as_semver();
        let b_sv = b.version_as_semver();

        if let (Some(a), Some(b)) = (a_sv, b_sv) {
            a.cmp(&b)
        } else {
            a.version().cmp(b.version())
        }
    });
    serde_json::to_value(args).unwrap() // handlebars deserializes this for us, so we can serialize
                                        // it back without issue
                                        // TODO: Make this helper nice
});
