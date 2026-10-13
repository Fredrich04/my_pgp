use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn Error>>;

fn rust_sources(dir: &Path, found: &mut Vec<PathBuf>) -> TestResult {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            rust_sources(&path, found)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
    Ok(())
}

fn files_in(folder: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut found = Vec::new();
    rust_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join(folder),
        &mut found,
    )?;
    Ok(found)
}

fn src_files() -> Result<Vec<PathBuf>, Box<dyn Error>> {
    files_in("src")
}

fn all_rust_files() -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut found = files_in("src")?;
    found.extend(files_in("tests")?);
    Ok(found)
}

fn contains_outside_string_literal(code: &str, pattern: &str) -> bool {
    code.match_indices(pattern)
        .any(|(index, _)| !code[..index].ends_with('"'))
}

fn production_part(source: &str) -> &str {
    source.split("#[cfg(test)]").next().unwrap_or(source)
}

#[test]
fn test_no_unwrap_expect_or_panic_in_production_code() -> TestResult {
    for path in src_files()? {
        let source = fs::read_to_string(&path)?;
        let code = production_part(&source);
        for forbidden in [
            ".unwrap()",
            ".expect(",
            "panic!(",
            "todo!(",
            "unimplemented!(",
        ] {
            assert!(
                !code.contains(forbidden),
                "{} contains {}: production code must handle errors explicitly (REGLES.md § 12.8)",
                path.display(),
                forbidden
            );
        }
    }
    Ok(())
}

#[test]
fn test_no_unwrap_or_expect_in_tests() -> TestResult {
    for path in files_in("tests")? {
        let source = fs::read_to_string(&path)?;
        for forbidden in [".unwrap()", ".expect("] {
            assert!(
                !contains_outside_string_literal(&source, forbidden),
                "{} contains {}: tests must return Result and use ? (REGLES.md § 12.8, § 14)",
                path.display(),
                forbidden
            );
        }
    }
    Ok(())
}

#[test]
fn test_every_test_name_starts_with_test() -> TestResult {
    for path in all_rust_files()? {
        let source = fs::read_to_string(&path)?;
        let lines: Vec<&str> = source.lines().map(str::trim).collect();
        for (index, pair) in lines.windows(2).enumerate() {
            if pair[0] == "#[test]" {
                assert!(
                    pair[1].starts_with("fn test_"),
                    "{}:{}: test must follow test_<action>_<expected_result> (REGLES.md § 34.1), found {:?}",
                    path.display(),
                    index + 2,
                    pair[1]
                );
            }
        }
    }
    Ok(())
}

#[test]
fn test_no_comments_in_source_code() -> TestResult {
    for path in all_rust_files()? {
        let source = fs::read_to_string(&path)?;
        for (number, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            assert!(
                !trimmed.starts_with("//") && !trimmed.starts_with("/*"),
                "{}:{} contains a comment; the team forbids comments in code, documentation goes to code.md",
                path.display(),
                number + 1
            );
        }
    }
    Ok(())
}

#[test]
fn test_no_unsafe_code_anywhere() -> TestResult {
    for path in src_files()? {
        let source = fs::read_to_string(&path)?;
        let code = source.replace("#![forbid(unsafe_code)]", "");
        assert!(
            !code.contains("unsafe"),
            "unsafe in {}: the team forbids unsafe code (REGLES.md § 37.2 bis)",
            path.display()
        );
    }
    Ok(())
}

#[test]
fn test_crate_roots_forbid_unsafe_code() -> TestResult {
    for root in ["src/lib.rs", "src/main.rs"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(root);
        let source = fs::read_to_string(&path)?;
        assert!(
            source.starts_with("#![forbid(unsafe_code)]"),
            "{} must start with #![forbid(unsafe_code)] (REGLES.md § 37.2 bis)",
            root
        );
    }
    Ok(())
}

#[test]
fn test_project_has_no_external_dependency() -> TestResult {
    let manifest = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))?;
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|rest| rest.split("\n[").next())
        .ok_or("missing [dependencies] section")?;
    assert!(
        dependencies.trim().is_empty(),
        "the project must have no external crate (REGLES.md § 37.2 bis), found: {}",
        dependencies
    );
    assert!(
        !manifest.contains("[dev-dependencies]"),
        "test-only crates are forbidden too (decision of 2026-09-24)"
    );
    Ok(())
}

fn source_folders(dir: &Path, found: &mut Vec<PathBuf>) -> TestResult {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            found.push(path.clone());
            source_folders(&path, found)?;
        }
    }
    Ok(())
}

#[test]
fn test_every_source_folder_is_documented() -> TestResult {
    let mut folders = Vec::new();
    source_folders(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut folders,
    )?;
    for path in folders {
        assert!(
            path.join("code.md").is_file(),
            "{} lacks code.md (REGLES.md § 35, documentation per folder)",
            path.display()
        );
        assert!(
            path.join("explication.md").is_file(),
            "{} lacks explication.md (REGLES.md § 35, documentation per folder)",
            path.display()
        );
    }
    Ok(())
}

fn parse_reference(span: &str) -> Option<(&str, usize, &str)> {
    let (file, rest) = span.split_once(".rs:")?;
    let (line, name) = rest.split_once(' ')?;
    let valid_file = !file.is_empty() && file.chars().all(|c| c.is_alphanumeric() || c == '_');
    let valid_name = !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_');
    if valid_file && valid_name {
        Some((file, line.parse().ok()?, name))
    } else {
        None
    }
}

fn declares(line: &str, name: &str) -> bool {
    ["fn", "struct", "enum", "const", "type", "trait"]
        .iter()
        .any(|keyword| {
            line.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .collect::<Vec<_>>()
                .windows(2)
                .any(|pair| pair[0] == *keyword && pair[1] == name)
        })
}

#[test]
fn test_documented_line_references_are_exact() -> TestResult {
    let mut folders = Vec::new();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    source_folders(&src, &mut folders)?;
    folders.push(src);
    let mut checked = 0;
    for folder in folders {
        for doc in ["code.md", "explication.md"] {
            let path = folder.join(doc);
            if !path.is_file() {
                continue;
            }
            let text = fs::read_to_string(&path)?;
            for span in text.split('`').skip(1).step_by(2) {
                let Some((file, line, name)) = parse_reference(span) else {
                    continue;
                };
                let source = fs::read_to_string(folder.join(format!("{file}.rs")))?;
                let found = source.lines().nth(line.saturating_sub(1)).unwrap_or("");
                assert!(
                    declares(found, name),
                    "{}: `{}` is stale, line {} of {}.rs is {:?} (REGLES.md § 39.5)",
                    path.display(),
                    span,
                    line,
                    file,
                    found
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked > 0,
        "no documented line reference found: code.md/explication.md must locate code (REGLES.md § 39.2, § 39.5)"
    );
    Ok(())
}
