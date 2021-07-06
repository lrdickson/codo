// Crates
use dirs;
use yaml_rust::{Yaml, YamlLoader};

pub const DEFAULT_IMAGE: &str = "default-image";

const DEFAULT_CODO_CONFIG: &str = "
default-image: fedora
";

pub fn get_codo_config() -> Vec<Yaml> {
    // Get the default codo config as a fallback
    let default_codo_config = YamlLoader::load_from_str(DEFAULT_CODO_CONFIG)
        .expect("Failed to parse default codo config.");

    
    return default_codo_config;
}
