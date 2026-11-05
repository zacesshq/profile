//! Keeps `spec/profile.schema.json` in step with the types the tool reads, so the spec an app
//! implements is the format this tool accepts.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;
use zacess_profile::profile::{Palette, Syntax};
use zacess_profile::{Profile, derive};

fn schema() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec/profile.schema.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn keys(v: &Value) -> BTreeSet<String> {
    v["properties"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect()
}

fn resolve<'a>(schema: &'a Value, v: &'a Value) -> &'a Value {
    match v.get("$ref").and_then(Value::as_str) {
        Some(r) => {
            let name = r.trim_start_matches("#/$defs/");
            &schema["$defs"][name]
        }
        None => v,
    }
}

#[test]
fn schema_lists_exactly_the_fields_the_tool_reads() {
    let s = schema();
    let needs = resolve(&s, &s["properties"]["needs"]);
    let contrast = resolve(&s, &needs["properties"]["contrast"]);
    let font = resolve(&s, &needs["properties"]["font"]);
    let palette = resolve(&s, &s["properties"]["palette"]);
    let syntax = resolve(&s, &palette["properties"]["syntax"]);
    let terminal = resolve(&s, &palette["properties"]["terminal"]);

    // a profile with every optional field set, written out by serde, names every field once
    let full = "spec_version = 1\nname = \"All\"\nid = \"all\"\ndescription = \"d\"\n\
        appearance = \"dark\"\n[needs]\ncursor_size = \"large\"\nfocus_ring_width_px = 3\n\
        line_spacing = 1.5\ncharacter_spacing = 0.1\ncolumn_width = 80\nsingle_column = true\n\
        hide_decoration = false\n[needs.contrast]\napca = 75\n[needs.font]\nfamily = \"f\"\n\
        fallback = \"monospace\"\nsize_pt = 14\n";
    let derived = derive(&Profile::parse(full).unwrap()).unwrap().profile;
    let json = serde_json::to_value(&derived).unwrap();
    let names =
        |v: &Value| -> BTreeSet<String> { v.as_object().unwrap().keys().cloned().collect() };

    assert_eq!(keys(&s), names(&json));
    assert_eq!(keys(needs), names(&json["needs"]));
    assert_eq!(keys(contrast), names(&json["needs"]["contrast"]));
    assert_eq!(keys(font), names(&json["needs"]["font"]));
    let mut palette_keys: BTreeSet<String> = Palette::ROLES.iter().map(|s| s.to_string()).collect();
    palette_keys.insert("syntax".into());
    palette_keys.insert("terminal".into());
    assert_eq!(keys(palette), palette_keys);
    let syntax_keys: BTreeSet<String> = Syntax::ROLES.iter().map(|s| s.to_string()).collect();
    assert_eq!(keys(syntax), syntax_keys);
    assert_eq!(
        keys(terminal),
        ["ansi", "background_slots", "dim"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    );
}

#[test]
fn schema_requires_what_the_tool_requires() {
    let s = schema();
    let required: Vec<&str> = s["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(required, ["spec_version", "name"]);
    assert_eq!(s["additionalProperties"], Value::Bool(false));
}

fn parses(body: &str) -> bool {
    Profile::parse(&format!("spec_version = 1\nname = \"x\"\n{body}")).is_ok()
}

/// Where a field sits in TOML, so a schema range can be tried against the parser.
fn snippet(table: &str, key: &str, value: &str) -> String {
    if table.is_empty() {
        format!("{key} = {value}")
    } else {
        format!("[{table}]\n{key} = {value}")
    }
}

#[test]
fn every_numeric_range_in_the_schema_is_the_parser_range() {
    let s = schema();
    let needs = resolve(&s, &s["properties"]["needs"]).clone();
    let tables = [
        ("needs", needs.clone()),
        (
            "needs.contrast",
            resolve(&s, &needs["properties"]["contrast"]).clone(),
        ),
        (
            "needs.font",
            resolve(&s, &needs["properties"]["font"]).clone(),
        ),
    ];
    let mut tried = 0;
    for (table, def) in &tables {
        for (key, field) in def["properties"].as_object().unwrap() {
            let (Some(min), Some(max)) = (field["minimum"].as_f64(), field["maximum"].as_f64())
            else {
                continue;
            };
            let integer = field["type"] == "integer";
            let step = if integer { 1.0 } else { 0.01 };
            let show = |v: f64| {
                if integer {
                    format!("{}", v as i64)
                } else {
                    format!("{v:.2}")
                }
            };
            assert!(
                parses(&snippet(table, key, &show(min))),
                "{table}.{key} = {min}"
            );
            assert!(
                parses(&snippet(table, key, &show(max))),
                "{table}.{key} = {max}"
            );
            assert!(
                !parses(&snippet(table, key, &show(min - step))),
                "{table}.{key} below"
            );
            assert!(
                !parses(&snippet(table, key, &show(max + step))),
                "{table}.{key} above"
            );
            tried += 1;
        }
    }
    assert!(tried >= 12, "only {tried} ranges found in the schema");
}

#[test]
fn every_enum_in_the_schema_is_what_the_parser_accepts() {
    let s = schema();
    let needs = resolve(&s, &s["properties"]["needs"]);
    let font = resolve(&s, &needs["properties"]["font"]);
    for (table, key, field) in [
        ("", "appearance", &s["properties"]["appearance"]),
        ("needs", "cursor_size", &needs["properties"]["cursor_size"]),
        ("needs.font", "fallback", &font["properties"]["fallback"]),
    ] {
        for value in field["enum"].as_array().unwrap() {
            let v = value.as_str().unwrap();
            // appearance must agree with the background, so give it a matching one
            let extra = match (key, v) {
                ("appearance", "light") => "\n[palette]\nbackground = \"#ffffff\"",
                _ => "",
            };
            assert!(
                parses(&format!(
                    "{}{extra}",
                    snippet(table, key, &format!("\"{v}\""))
                )),
                "{key} {v}"
            );
        }
        assert!(!parses(&snippet(table, key, "\"bogus\"")), "{key}");
    }
}

#[test]
fn every_pattern_in_the_schema_is_what_the_parser_accepts() {
    let s = schema();
    let pattern = |v: &Value| regex::Regex::new(v["pattern"].as_str().unwrap()).unwrap();
    let id = pattern(&s["properties"]["id"]);
    let name = pattern(&s["properties"]["name"]);
    let named = pattern(&s["then"]["properties"]["name"]);
    let needs = resolve(&s, &s["properties"]["needs"]);
    let family = pattern(&resolve(&s, &needs["properties"]["font"])["properties"]["family"]);
    let colour = pattern(&s["$defs"]["colour"]);

    for sample in ["a", "large-text-2", "a--b", "-a", "a-", "A", "a_b", ""] {
        let ok = Profile::parse(&format!(
            "spec_version = 1\nname = \"x\"\nid = \"{sample}\""
        ))
        .is_ok();
        assert_eq!(ok, id.is_match(sample), "id {sample:?}");
    }
    for sample in [
        "Mine",
        "  spaced  ",
        "Zacess Light",
        " ",
        "",
        "tab\there",
        "line\nbreak",
        "\u{85}x",
    ] {
        let escaped = sample.escape_default().to_string();
        let ok = Profile::parse(&format!("spec_version = 1\nname = \"{escaped}\"")).is_ok();
        let schema_ok = name.is_match(sample) && named.is_match(sample);
        assert_eq!(ok, schema_ok, "name {sample:?}");
    }
    // without an id the name must give a file name; with one it need not
    assert!(!named.is_match("日本語"));
    assert!(Profile::parse("spec_version = 1\nname = \"日本語\"\nid = \"nihongo\"").is_ok());
    for sample in [
        "Atkinson Hyperlegible Mono",
        "",
        "Mono\"",
        "Mono\\",
        "Mo\tno",
    ] {
        let escaped = sample.escape_default().to_string();
        assert_eq!(
            parses(&format!("[needs.font]\nfamily = \"{escaped}\"")),
            family.is_match(sample),
            "family {sample:?}"
        );
    }
    for sample in ["#1a2B3c", "#1a2b3", "1a2b3c", "#gggggg"] {
        assert_eq!(
            parses(&format!("[palette]\nbackground = \"{sample}\"")),
            colour.is_match(sample),
            "colour {sample:?}"
        );
    }
}

#[test]
fn array_and_version_constraints_match_the_parser() {
    let s = schema();
    let palette = resolve(&s, &s["properties"]["palette"]);
    let terminal = resolve(&s, &palette["properties"]["terminal"]);
    let ansi = &terminal["properties"]["ansi"];
    assert_eq!(ansi["minItems"], 16);
    assert_eq!(ansi["maxItems"], 16);
    let sixteen = vec!["\"#000000\""; 16].join(", ");
    let fifteen = vec!["\"#000000\""; 15].join(", ");
    assert!(parses(&format!("[palette.terminal]\nansi = [{sixteen}]")));
    assert!(!parses(&format!("[palette.terminal]\nansi = [{fifteen}]")));
    for key in ["dim", "background_slots"] {
        let field = &terminal["properties"][key];
        assert_eq!(field["uniqueItems"], true, "{key}");
        assert_eq!(field["items"]["maximum"], 15, "{key}");
        assert!(
            parses(&format!("[palette.terminal]\n{key} = [0, 15]")),
            "{key}"
        );
        assert!(
            !parses(&format!("[palette.terminal]\n{key} = [16]")),
            "{key}"
        );
        assert!(
            !parses(&format!("[palette.terminal]\n{key} = [1, 1]")),
            "{key}"
        );
    }
    assert_eq!(s["properties"]["spec_version"]["const"], 1);
    assert!(Profile::parse("spec_version = 1\nname = \"x\"").is_ok());
    assert!(Profile::parse("spec_version = 2\nname = \"x\"").is_err());
}
