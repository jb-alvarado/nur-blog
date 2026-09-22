use std::{
    collections::BTreeMap,
    fs::{create_dir_all, read_to_string, write},
    path::{Path, PathBuf},
};

use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
};
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

const CSS_ASSETS: &[(&str, &str)] = &[
    ("web/blog.css", "assets/blog.min.css"),
    ("web/admin.css", "assets/admin.min.css"),
];
const JAVASCRIPT_ASSETS: &[(&str, &str)] = &[
    ("web/admin.js", "assets/admin.min.js"),
    ("web/blog.js", "assets/blog.min.js"),
];

fn main() {
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo provides CARGO_MANIFEST_DIR"),
    );
    println!("cargo:rerun-if-changed=locales");
    let admin_translations = admin_translations(&manifest_dir.join("locales"));
    for (source, output) in CSS_ASSETS {
        println!("cargo:rerun-if-changed={source}");
        minify_css(&manifest_dir.join(source), &manifest_dir.join(output));
    }
    for (source, output) in JAVASCRIPT_ASSETS {
        println!("cargo:rerun-if-changed={source}");
        minify_javascript(
            &manifest_dir.join(source),
            &manifest_dir.join(output),
            (*source == "web/admin.js").then_some(admin_translations.as_str()),
        );
    }
}

fn admin_translations(locale_dir: &Path) -> String {
    let mut translations = BTreeMap::<String, BTreeMap<String, String>>::new();
    let entries = std::fs::read_dir(locale_dir).unwrap_or_else(|error| {
        panic!("could not read {}: {error}", locale_dir.display());
    });
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| panic!("could not read locale entry: {error}"))
            .path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let locale = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_else(|| panic!("locale filename is not valid UTF-8: {}", path.display()));
        let source = read_to_string(&path)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()));
        let values: BTreeMap<String, serde_json::Value> = serde_json::from_str(&source)
            .unwrap_or_else(|error| panic!("could not parse {}: {error}", path.display()));
        let messages = values
            .into_iter()
            .filter_map(|(key, value)| {
                let key = key.strip_prefix("admin.")?.to_owned();
                let value = value
                    .as_str()
                    .unwrap_or_else(|| {
                        panic!(
                            "admin translation {key} in {} is not a string",
                            path.display()
                        )
                    })
                    .to_owned();
                Some((key, value))
            })
            .collect();
        translations.insert(locale.to_owned(), messages);
    }
    serde_json::to_string(&translations).expect("admin translations serialize")
}

fn minify_css(source: &Path, output: &Path) {
    let contents = read_to_string(source).unwrap_or_else(|error| {
        panic!("could not read {}: {error}", source.display());
    });
    let mut stylesheet =
        StyleSheet::parse(&contents, ParserOptions::default()).unwrap_or_else(|error| {
            panic!("could not parse {}: {error}", source.display());
        });
    stylesheet
        .minify(MinifyOptions::default())
        .unwrap_or_else(|error| {
            panic!("could not minify {}: {error}", source.display());
        });
    let minified = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            ..PrinterOptions::default()
        })
        .unwrap_or_else(|error| {
            panic!("could not print {}: {error}", source.display());
        });
    write_asset(output, minified.code);
}

fn minify_javascript(source: &Path, output: &Path, admin_translations: Option<&str>) {
    let mut contents = read_to_string(source).unwrap_or_else(|error| {
        panic!("could not read {}: {error}", source.display());
    });
    if let Some(translations) = admin_translations {
        contents = contents.replace("__NUR_BLOG_TRANSLATIONS__", translations);
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &contents, SourceType::mjs()).parse();

    if parsed.diagnostics.has_errors() {
        panic!(
            "could not parse {}: {:?}",
            source.display(),
            parsed.diagnostics
        );
    }

    let mut program = parsed.program;
    let minified = Minifier::new(MinifierOptions::default()).minify(&allocator, &mut program);
    let generated = Codegen::new()
        .with_options(CodegenOptions {
            minify: true,
            comments: CommentOptions::disabled(),
            ..CodegenOptions::default()
        })
        .with_scoping(minified.scoping)
        .build(&program);

    write_asset(output, generated.code);
}

fn write_asset(output: &Path, contents: impl AsRef<[u8]>) {
    create_dir_all(output.parent().expect("asset output has a parent"))
        .unwrap_or_else(|error| panic!("could not create the asset directory: {error}"));
    write(output, contents).unwrap_or_else(|error| {
        panic!("could not write {}: {error}", output.display());
    });
}
