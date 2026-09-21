use std::{
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
    for (source, output) in CSS_ASSETS {
        println!("cargo:rerun-if-changed={source}");
        minify_css(&manifest_dir.join(source), &manifest_dir.join(output));
    }
    for (source, output) in JAVASCRIPT_ASSETS {
        println!("cargo:rerun-if-changed={source}");
        minify_javascript(&manifest_dir.join(source), &manifest_dir.join(output));
    }
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

fn minify_javascript(source: &Path, output: &Path) {
    let contents = read_to_string(source).unwrap_or_else(|error| {
        panic!("could not read {}: {error}", source.display());
    });
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
