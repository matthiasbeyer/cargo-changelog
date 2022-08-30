pub mod helpers {
    use crate::command::VersionData;
    use handlebars::handlebars_helper;

    handlebars_helper!(sort_versions: |args: Vec<VersionData>| {
        let mut args = args.clone();
        args.sort_by(|a, b| a.version().cmp(&b.version()));
        serde_json::to_value(args).unwrap() // handlebars deserializes this for us, so we can serialize
                                            // it back without issue
                                            // TODO: Make this helper nice
    });

    handlebars_helper!(reverse: |args: Vec<VersionData>| {
        let args: Vec<VersionData> = args.clone().into_iter().rev().collect();
        serde_json::to_value(args).unwrap() // handlebars deserializes this for us, so we can serialize
                                            // it back without issue
                                            // TODO: Make this helper nice
    });
}
