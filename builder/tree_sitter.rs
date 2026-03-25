use cc;
use indoc::formatdoc;
use std::{error::Error, fs, iter::empty, path::PathBuf, process::Command};

pub struct ParserManifest {
    language: String,
    repo_url: String,
    revision: String,
}

impl ParserManifest {
    fn lib_name(&self) -> String {
        format!("tree_sitter_{}", self.language)
    }

    fn repo_dir(&self) -> PathBuf {
        PathBuf::from_iter(["target/tree-sitter-grammars/", &self.language])
    }

    fn clone_repo(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("git")
            .args([
                "clone",
                "--depth=1",
                &self.repo_url,
                &self.repo_dir().to_string_lossy(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to clone {}: {}",
                self.lib_name(),
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(())
    }

    fn fetch_repo(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("git")
            .current_dir(self.repo_dir())
            .args(["fetch", "origin"])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to fetch {}: {}",
                self.lib_name(),
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(())
    }

    fn checkout_repo(&self) -> Result<(), Box<dyn Error>> {
        let output = Command::new("git")
            .current_dir(self.repo_dir())
            .args(["checkout", &format!("origin/{}", self.revision)])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to checkout {}: {}",
                self.lib_name(),
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(())
    }

    fn get_repo_revision(&self) -> Result<String, Box<dyn Error>> {
        let output = Command::new("git")
            .current_dir(self.repo_dir())
            .args(["rev-parse", "HEAD"])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to rev-parse {}: {}",
                self.lib_name(),
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(sha)
    }

    fn compile_source(&self) -> Result<(), Box<dyn Error>> {
        let source = self.repo_dir().join("src/parser.c");

        cc::Build::new()
            .file(&source)
            .try_compile(&self.lib_name())?;

        Ok(())
    }

    fn load_query(&self, file: String) -> Result<String, Box<dyn Error>> {
        let out_dir = std::env::var("OUT_DIR")?;

        let in_path = self.repo_dir().join("queries").join(&file);
        let out_file = format!("{}-{}", &self.language, &file);
        let out_path = PathBuf::from_iter([&out_dir, &out_file]);

        fs::copy(in_path, &out_path)?;

        Ok(out_file)
    }

    fn gen_query_const(name: &str, file_path: String) -> String {
        if !file_path.is_empty() {
            format!(
                "    pub const {name}_QUERY: &str = include_str!(concat!(env!(\"OUT_DIR\"), \"/{file_path}\"));\n"
            )
        } else {
            "".into()
        }
    }

    fn gen_wrapper(&self) -> Result<(), Box<dyn Error>> {
        let out_dir = std::env::var("OUT_DIR")?;
        let file_name = self.lib_name() + ".rs";
        let file_path = PathBuf::from_iter([&out_dir, &file_name]);

        let highlights_file = self.load_query("highlights.scm".into())?;
        let injections_file = self.load_query("injections.scm".into()).unwrap_or_default();
        let locals_file = self.load_query("locals.scm".into()).unwrap_or_default();

        // TODO: also add injection_query and locals_query if they are found
        // TODO: HighlightConfiguration fn / const that constructs the struct accordingly
        let file_content = formatdoc! {"
            pub mod {0} {{
                use tree_sitter_language::LanguageFn;

                unsafe extern \"C\" {{
                    fn {0}() -> *const ();
                }}

                pub const LANGUAGE: LanguageFn = unsafe {{ LanguageFn::from_raw({0}) }};

            {1}{2}{3}}}",
            self.lib_name(),
            Self::gen_query_const("HIGHLIGHTS", highlights_file),
            Self::gen_query_const("INJECTIONS", injections_file),
            Self::gen_query_const("LOCALS", locals_file),
        };

        fs::write(file_path, file_content)?;

        Ok(())
    }
}

pub struct ParserManifests(Vec<ParserManifest>);

impl ParserManifests {
    pub fn build(&self) -> Result<(), Box<dyn Error>> {
        let cur_file = file!();
        println!("cargo:rerun-if-changed={cur_file}");

        for manifest in &self.0 {
            if !manifest.repo_dir().exists() {
                manifest.clone_repo()?;
            } else {
                match manifest.get_repo_revision() {
                    Ok(rev) if rev == manifest.revision => continue,
                    _ => manifest.fetch_repo()?,
                }
            }
            manifest.checkout_repo()?;

            manifest.compile_source()?;
            manifest.gen_wrapper()?;
        }

        self.gen_impl()
    }

    fn gen_impl(&self) -> Result<(), Box<dyn Error>> {
        let out_dir = std::env::var("OUT_DIR")?;
        let file_path = PathBuf::from_iter([&out_dir, "tree_sitter_parsers.rs"]);

        let mut file_content = String::new();
        for manifest in &self.0 {
            file_content.push_str(&format!(
                "include!(concat!(env!(\"OUT_DIR\"), \"/{}.rs\"));\n",
                manifest.lib_name()
            ));
        }

        fs::write(&file_path, file_content)?;

        Ok(())
    }

    pub fn all() -> ParserManifests {
        ParserManifests(vec![
            ParserManifest {
                language: "go".into(),
                repo_url: "git@github.com:tree-sitter/tree-sitter-go.git".into(),
                revision: "master".into(),
            },
            ParserManifest {
                language: "ada".into(),
                repo_url: "git@github.com:briot/tree-sitter-ada.git".into(),
                revision: "master".into(),
            },
        ])
    }
}
