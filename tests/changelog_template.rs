use std::collections::HashMap;

use handlebars::Handlebars;
use predicates::prelude::*;

const TEMPLATE: &'static str = include_str!("../assets/default_template.handlebars.md");

#[test]
fn default_template_renders_with_empty_data() {
    let mut hb = Handlebars::new();
    let data: HashMap<String, Vec<String>> = HashMap::new();
    hb.register_template_string("t", TEMPLATE).unwrap();
    let template = hb.render("t", &data);
    assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
    let template = template.unwrap();

    assert!(
        predicates::str::contains("CHANGELOG").eval(&template),
        "Does not contain 'CHANGELOG': {}",
        template
    );
}

#[derive(serde::Serialize)]
struct FragmentMock {
    text: String,
    header: HashMap<String, u64>,
}

#[derive(serde::Serialize)]
struct VersionMock {
    version: String,
    entries: Vec<FragmentMock>,
}

#[test]
fn default_template_renders_with_one_entry() {
    let mut hb = Handlebars::new();
    let mut data: HashMap<String, Vec<_>> = HashMap::new();
    data.insert(
        "versions".to_string(),
        vec![VersionMock {
            version: "0.1.0".to_string(),
            entries: vec![FragmentMock {
                text: "test for 0.1.0".to_string(),
                header: {
                    let mut hdr = HashMap::new();
                    hdr.insert("issue".to_string(), 123);
                    hdr
                },
            }],
        }],
    );
    hb.register_template_string("t", TEMPLATE).unwrap();
    let template = hb.render("t", &data);
    assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
    let template = template.unwrap();

    assert!(
        predicates::str::contains("## v0.1.0").eval(&template),
        "Does not contain '## v0.1.0': {}",
        template
    );

    assert!(
        predicates::str::contains("test for 0.1.0").eval(&template),
        "Does not contain 'test text': {}",
        template
    );
}

#[test]
fn default_template_renders_with_one_entry_with_header() {
    let mut hb = Handlebars::new();
    let mut data: HashMap<String, Vec<_>> = HashMap::new();
    data.insert(
        "versions".to_string(),
        vec![VersionMock {
            version: "0.1.0".to_string(),
            entries: vec![FragmentMock {
                text: "test for 0.1.0".to_string(),
                header: {
                    let mut hdr = HashMap::new();
                    hdr.insert("issue".to_string(), 123);
                    hdr
                },
            }],
        }],
    );
    hb.register_template_string("t", TEMPLATE).unwrap();
    let template = hb.render("t", &data);
    assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
    let template = template.unwrap();

    assert!(
        predicates::str::contains("(#123)").eval(&template),
        "Does not contain '(#123)': {}",
        template
    );
}

#[test]
fn default_template_renders_versions_sorted() {
    let mut hb = Handlebars::new();
    let mut data: HashMap<String, Vec<_>> = HashMap::new();
    data.insert(
        "versions".to_string(),
        vec![
            VersionMock {
                version: "0.1.0".to_string(),
                entries: vec![FragmentMock {
                    text: "test for 0.1.0".to_string(),
                    header: {
                        let mut hdr = HashMap::new();
                        hdr.insert("issue".to_string(), 123);
                        hdr
                    },
                }],
            },
            VersionMock {
                version: "0.2.0".to_string(),
                entries: vec![FragmentMock {
                    text: "test for 0.2.0".to_string(),
                    header: {
                        let mut hdr = HashMap::new();
                        hdr.insert("issue".to_string(), 234);
                        hdr
                    },
                }],
            },
        ],
    );
    hb.register_template_string("t", TEMPLATE).unwrap();
    let template = hb.render("t", &data);
    assert!(template.is_ok(), "Not ok: {:?}", template.unwrap_err());
    let template = template.unwrap();

    assert!(
        predicates::str::contains("## v0.1.0").eval(&template),
        "Does not contain '## v0.1.0': {}",
        template
    );
    assert!(
        predicates::str::contains("## v0.2.0").eval(&template),
        "Does not contain '## v0.2.0': {}",
        template
    );

    let line_number_of_010 = {
        template
            .lines()
            .enumerate()
            .filter(|(_n, line)| *line == "## v0.1.0")
            .next()
            .map(|(n, _)| n)
            .unwrap()
    };

    let line_number_of_020 = {
        template
            .lines()
            .enumerate()
            .filter(|(_n, line)| *line == "## v0.2.0")
            .next()
            .map(|(n, _)| n)
            .unwrap()
    };

    assert!(
        line_number_of_020 < line_number_of_010,
        "line with v0.1.0 should come _after_ line with v0.2.0: {}",
        template
    );
}
