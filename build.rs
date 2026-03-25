use std::error::Error;

mod builder;

pub fn main() -> Result<(), Box<dyn Error>> {
    builder::tree_sitter::ParserManifests::all().build()
}
