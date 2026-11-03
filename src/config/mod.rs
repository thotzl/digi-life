//! App configuration (`config.json`, the single source of truth for tunables).
//!
//! `AppConfig` is a plain struct with one field per section. Each section type lives next to the code that reads it
//! (contract: `wiki/contracts/config-section.md`):
//!
//! | Section      | Type                                         |
//! | :----------- | :------------------------------------------- |
//! | `core`       | `config::CoreConfig`                         |
//! | `ocean`      | `config::ocean::OceanConfig`                 |
//! | `trainer`    | `biology::trainer_config::TrainerConfig`     |
//! | `genome`     | `biology::genome::GenomeConfig`              |
//! | `brain`      | `shared::brain::BrainConfig`                 |
//! | `metabolism` | `biology::metabolism::MetabolismConfig`      |
//! | `physics`    | `shared::physics::PhysicsConfig`             |
//!
//! The config is passed explicitly into the model; `AppConfig::global()` is called only by the entry points (Tauri
//! setup in `main.rs`, the headless runner).

pub mod ocean;

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::biology::genome::GenomeConfig;
use crate::biology::metabolism::MetabolismConfig;
use crate::biology::trainer_config::TrainerConfig;
use crate::shared::brain::BrainConfig;
use crate::shared::physics::PhysicsConfig;

pub use ocean::OceanConfig;

/// Section `core`: settings shared by every mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CoreConfig {
    /// World seed: Ocean world generation, `INIT_STATE.seed`, and the base of every trainer run seed
    /// (`trainer_sim::run_seed`)
    pub seed: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppConfig {
    pub core: CoreConfig,
    pub ocean: OceanConfig,
    pub trainer: TrainerConfig,
    pub genome: GenomeConfig,
    pub brain: BrainConfig,
    pub metabolism: MetabolismConfig,
    pub physics: PhysicsConfig,
}

static GLOBAL_CONFIG: OnceLock<AppConfig> = OnceLock::new();
const EMBEDDED_CONFIG: &str = include_str!("../../config.json");

impl AppConfig {
    /// The process-wide config, loaded once. Entry points only (Tauri setup, headless runner); everything below them
    /// receives the config (or one section) as a parameter.
    pub fn global() -> &'static Self {
        GLOBAL_CONFIG.get_or_init(Self::load)
    }

    /// The config compiled into the binary. Debug builds let a `config.json` in the working directory override it
    /// (`tauri dev` runs from the repo root); an invalid override is logged and ignored. Release builds always use the
    /// embedded config.
    pub fn load() -> Self {
        #[cfg(debug_assertions)]
        if let Ok(file_content) = std::fs::read_to_string("config.json") {
            match serde_json::from_str::<Self>(&file_content) {
                Ok(parsed) => {
                    log::info!("loaded override from working directory config.json");
                    return parsed;
                }
                Err(e) => log::error!(
                    "working directory config.json is invalid ({}); using the embedded config",
                    e
                ),
            }
        }
        log::info!("using the embedded config.json");
        Self::embedded()
    }

    pub fn embedded() -> Self {
        serde_json::from_str(EMBEDDED_CONFIG).expect("embedded config.json must match AppConfig")
    }
}

/// Embedded config for unit tests (parsed once). Tests pass it like the entry points pass `AppConfig::global()`.
#[cfg(test)]
pub fn test_config() -> &'static AppConfig {
    static TEST_CONFIG: OnceLock<AppConfig> = OnceLock::new();
    TEST_CONFIG.get_or_init(AppConfig::embedded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn on_disk() -> serde_json::Value {
        serde_json::from_str(include_str!("../../config.json")).unwrap()
    }

    /// Dotted paths of all leaf keys (`trainer.trainerCurriculum.chamberSize.standard`).
    fn leaf_paths(value: &serde_json::Value, prefix: &str, out: &mut BTreeSet<String>) {
        match value.as_object() {
            Some(object) => {
                for (key, child) in object {
                    let path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    leaf_paths(child, &path, out);
                }
            }
            None => {
                out.insert(prefix.to_string());
            }
        }
    }

    #[test]
    fn test_embedded_config_matches_config_json() {
        let embedded = AppConfig::embedded();
        let disk = on_disk();
        assert_eq!(
            embedded.ocean.food_spore_count as u64,
            disk["ocean"]["foodSporeCount"].as_u64().unwrap()
        );
        assert_eq!(embedded.core.seed, disk["core"]["seed"].as_str().unwrap());
        assert!(
            (embedded.physics.locomotion_speed_max as f64 - disk["physics"]["locomotionSpeedMax"].as_f64().unwrap())
                .abs()
                < 1e-6
        );
    }

    /// Every key of `config.json` maps onto a struct field and every field is in `config.json`: deserialize →
    /// serialize gives exactly the key set of the file (a dropped, renamed or defaulted key shows up as a difference).
    #[test]
    fn test_config_roundtrip_keeps_every_key() {
        let disk = on_disk();
        let roundtrip = serde_json::to_value(serde_json::from_value::<AppConfig>(disk.clone()).unwrap()).unwrap();
        let (mut expected, mut actual) = (BTreeSet::new(), BTreeSet::new());
        leaf_paths(&disk, "", &mut expected);
        leaf_paths(&roundtrip, "", &mut actual);
        assert_eq!(expected, actual, "config.json keys and AppConfig fields differ");
    }

    fn camel_to_snake(key: &str) -> String {
        let mut out = String::new();
        for c in key.chars() {
            if c.is_ascii_uppercase() {
                out.push('_');
                out.push(c.to_ascii_lowercase());
            } else {
                out.push(c);
            }
        }
        out
    }

    /// Production code of a Rust file: test modules (`#[cfg(test)] mod tests`, always at the end of a file here),
    /// comments and string literals removed, so only code can count as a read site.
    fn production_code(source: &str) -> String {
        let source = source.split("#[cfg(test)]\nmod tests").next().unwrap_or("");
        let source = source.split("#[cfg(test)]\r\nmod tests").next().unwrap_or("");
        let chars: Vec<char> = source.chars().collect();
        let mut out = String::with_capacity(source.len());
        let mut i = 0;
        while i < chars.len() {
            match (chars[i], chars.get(i + 1)) {
                ('/', Some('/')) => {
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                }
                ('/', Some('*')) => {
                    i += 2;
                    while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                        i += 1;
                    }
                    i += 2;
                }
                ('"', _) => {
                    i += 1;
                    while i < chars.len() && chars[i] != '"' {
                        i += if chars[i] == '\\' { 2 } else { 1 };
                    }
                    i += 1;
                    out.push_str("\"\"");
                }
                (c, _) => {
                    out.push(c);
                    i += 1;
                }
            }
        }
        out
    }

    /// Whether `code` reads `field` as a struct field: the token sequence `.` `field` not followed by `(` (a method
    /// call) — `config.trainer.mutation_ops_limit`, `c.step`, `spawn.standard`.
    fn reads_field(code: &str, field: &str) -> bool {
        let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
        code.match_indices('.').any(|(dot, _)| {
            let rest = code[dot + 1..].trim_start();
            rest.starts_with(field)
                && !rest[field.len()..].starts_with(is_ident)
                && !rest[field.len()..].trim_start().starts_with('(')
        })
    }

    fn collect_production_code(dir: &std::path::Path, out: &mut String) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_production_code(&path, out);
            } else if path.extension().map_or(false, |e| e == "rs") {
                out.push_str(&production_code(&std::fs::read_to_string(&path).unwrap()));
                out.push('\n');
            }
        }
    }

    /// Every config key is read by production code: for every field of every section (key list taken from the
    /// struct via serde, so nested blocks count), `src/` outside test modules, comments and strings contains a field
    /// access `.field`. Limitation: a generic field name (`standard`, `step`) also matches a same-named field of
    /// another type; the section's contract lists its readers.
    #[test]
    fn test_every_config_key_has_read_site() {
        let config = serde_json::to_value(AppConfig::embedded()).unwrap();
        let mut keys = BTreeSet::new();
        leaf_paths(&config, "", &mut keys);
        let mut fields = BTreeSet::new();
        for path in &keys {
            // Section names are not fields that code reads by themselves; every nested key below them is
            for key in path.split('.').skip(1) {
                fields.insert(camel_to_snake(key));
            }
        }
        let mut code = String::new();
        collect_production_code(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut code);
        let unread: Vec<&String> = fields.iter().filter(|field| !reads_field(&code, field)).collect();
        assert!(
            unread.is_empty(),
            "config keys without a read site in production code: {:?}",
            unread
        );
    }

    #[test]
    fn test_read_site_scan_ignores_comments_strings_and_tests() {
        let code = production_code(
            "let a = c.used;\n// c.in_comment\nlet s = \".in_string\";\nfn f() { x.call(); }\n#[cfg(test)]\nmod tests { fn t() { c.in_test; } }",
        );
        assert!(reads_field(&code, "used"));
        assert!(!reads_field(&code, "in_comment"));
        assert!(!reads_field(&code, "in_string"));
        assert!(!reads_field(&code, "in_test"));
        assert!(!reads_field(&code, "call"), "a method call is not a field read");
        assert!(!reads_field(&code, "use"), "a prefix of a field is not a read");
    }

    #[test]
    fn test_unknown_config_key_is_rejected() {
        for section in ["core", "ocean", "trainer", "genome", "brain", "metabolism", "physics"] {
            let mut disk = on_disk();
            disk[section]["ventsBaseCount"] = serde_json::json!(6);
            assert!(
                serde_json::from_value::<AppConfig>(disk).is_err(),
                "unknown key accepted in section {}",
                section
            );
        }
        let mut disk = on_disk();
        disk["rules"] = serde_json::json!({});
        assert!(
            serde_json::from_value::<AppConfig>(disk).is_err(),
            "unknown section accepted"
        );
    }
}
