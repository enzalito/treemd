use cc;
use std::{
    path::PathBuf,
    process::{self, Command},
};

pub struct ParserDefinition {
    name: String,
    git_repo: String,
    revision: String,
}

pub fn build_parsers(definitions: &Vec<ParserDefinition>) {
    for definition in definitions {
        let parser_dir = PathBuf::from_iter([
            "target/tree-sitter-grammars/",
            &definition.name,
            &definition.revision,
        ]);

        build_parser(&definition.name, &parser_dir);
    }
}

pub fn build_parser(name: &String, dir: &PathBuf) {
    let source = dir.join("src/parser.c");
    if !source.exists() {
        eprintln!(
            "Build failed: missing tree-sitter parser source: {}",
            source.display()
        );
        process::exit(1);
    }

    cc::Build::new()
        .file(&source)
        .compile(&format!("tree-sitter-{}", name));
}
