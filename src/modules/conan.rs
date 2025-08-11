use super::{Context, Module, ModuleConfig};
use crate::formatter::VersionFormatter;

use crate::configs::conan::ConanConfig;
use crate::formatter::StringFormatter;

/// Creates a module with the current `Conan` version
pub fn module<'a>(context: &'a Context) -> Option<Module<'a>> {
    let mut module = context.new_module("conan");
    let config = ConanConfig::try_load(module.config);

    let is_conan_project = context
        .try_begin_scan()?
        .set_files(&config.detect_files)
        .set_extensions(&config.detect_extensions)
        .set_folders(&config.detect_folders)
        .is_match();

    if !is_conan_project {
        return None;
    }

    let parsed = StringFormatter::new(config.format).and_then(|formatter| {
        formatter
            .map_meta(|variable, _| match variable {
                "symbol" => Some(config.symbol),
                _ => None,
            })
            .map_style(|variable| match variable {
                "style" => Some(Ok(config.style)),
                _ => None,
            })
            .map(|variable| match variable {
                "version" => {
                    let conan_version =
                        parse_conan_version(&context.exec_cmd("conan", &["--version"])?.stdout)?;
                    VersionFormatter::format_module_version(
                        module.get_name(),
                        &conan_version,
                        config.version_format,
                    )
                    .map(Ok)
                }
                _ => None,
            })
            .parse(None, Some(context))
    });

    module.set_segments(match parsed {
        Ok(segments) => segments,
        Err(error) => {
            log::warn!("Error in module `conan`: \n{error}");
            return None;
        }
    });

    Some(module)
}

fn parse_conan_version(conan_version: &str) -> Option<String> {
    Some(
        conan_version
            //split into ["conan" "version" "3.10.2", ...]
            .split_whitespace()
            // get down to "3.10.2"
            .nth(2)?
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use crate::test::ModuleRenderer;
    use nu_ansi_term::Color;
    use std::fs::File;
    use std::io;

    #[test]
    fn folder_without_conanfile() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let actual = ModuleRenderer::new("conan").path(dir.path()).collect();
        let expected = None;
        assert_eq!(expected, actual);
        dir.close()
    }

    #[test]
    fn folder_with_conanfile_txt() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join("conanfile.txt"))?.sync_all()?;
        let actual = ModuleRenderer::new("conan").path(dir.path()).collect();
        let expected = Some(format!("via {}", Color::Blue.bold().paint("△ v2.19.1 ")));
        assert_eq!(expected, actual);
        dir.close()
    }

    #[test]
    fn folder_with_conanfile_py() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join("conanfile.py"))?.sync_all()?;
        let actual = ModuleRenderer::new("conan").path(dir.path()).collect();
        let expected = Some(format!("via {}", Color::Blue.bold().paint("△ v2.19.1 ")));
        assert_eq!(expected, actual);
        dir.close()
    }
}

