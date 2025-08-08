use std::collections::BTreeMap;
use std::str;

use webidl::{self, ast};

/// Helper method for inserting into a BTreeMap-based multi-map
pub fn multimap_insert<K: Ord, V: PartialEq>(m: &mut BTreeMap<K, Vec<V>>, key: K, value: V) {
    let v = m.entry(key).or_default();
    if !v.contains(&value) {
        v.push(value);
    }
}

/// Helper method for appending to a BTreeMap-based multi-map
pub fn multimap_append<K: Ord + Clone, V: PartialEq>(
    m: &mut BTreeMap<K, Vec<V>>,
    other: BTreeMap<K, Vec<V>>,
) {
    for (k, v) in other {
        for value in v {
            multimap_insert(m, k.clone(), value);
        }
    }
}

/// Best-effort attempt to render HTML into a doc-comment which can
/// be placed in the generated code.
pub fn convert_html_to_doc_comment(html: &str) -> String {
    use regex::RegexBuilder;

    // Create doc comments
    let doc_comment_regex = RegexBuilder::new("^").multi_line(true).build().unwrap();

    let md = html2text::from_read(std::io::Cursor::new(html), 100).unwrap();
    let mut doc = doc_comment_regex.replace_all(md.trim_end(), "/// ").into();
    doc += "\n";
    doc
}

/// Appends an underscore to a name if it conflicts with a reserved word.
pub fn unreserve<S: Into<String>>(name: S) -> String {
    const RESERVED_WORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "Self", "self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "abstract", "alignof", "become", "box", "do", "final", "macro",
        "offsetof", "override", "priv", "proc", "pure", "sizeof", "typeof", "unsized", "virtual",
        "yield",
    ];

    let mut s = name.into();
    if RESERVED_WORDS.contains(&&*s) {
        s += "_";
    }
    s
}

/// Converts to lower snake-case
pub fn snake(name: &str) -> String {
    use heck::SnakeCase;
    name.to_snake_case()
}

/// Converts to upper snake-case
pub fn shouty_snake(name: &str) -> String {
    use heck::ShoutySnakeCase;
    name.to_shouty_snake_case()
}

/// Converts to upper camel case
pub fn camel(name: &str) -> String {
    use heck::CamelCase;
    name.to_camel_case()
}

/// Parse WebIDL content into an AST
pub fn parse_defs(src: &[u8]) -> Vec<ast::Definition> {
    let src = str::from_utf8(src).expect("IDL contained invalid UTF-8");

    match webidl::parse_string(src) {
        Ok(defs) => defs,
        Err(error) => panic!("Failed to parse IDL:\n{}\nError: {:?}", src, error),
    }
}
