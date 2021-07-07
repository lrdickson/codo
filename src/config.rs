// Standard
use std;
use std::fs;
use std::path;

// Crates
use dirs;
use log::error;
use yaml_rust::{Yaml, YamlLoader};

pub const DEFAULT_IMAGE: &str = "default-image";

const DEFAULT_CODO_CONFIG: &str = "
default-image: fedora
";

pub fn get_codo_config<'a>() -> Result<Yaml, Box<dyn std::error::Error>> {
    // Get the default codo config as a fallback
    let default_codo_config = YamlLoader::load_from_str(DEFAULT_CODO_CONFIG)
        .expect("Failed to parse default codo config.")[0]
        .to_owned();

    // Get the codo config file
    let mut codo_config_file = match get_codo_config_dir() {
        Some(dir) => dir,
        None => return Ok(default_codo_config)
    };
    codo_config_file.push("codo.yaml");

    // Check if the codo config file exists
    let codo_config_path = path::Path::new(&codo_config_file);
    if !( codo_config_path.exists() && codo_config_path.is_file() ) {
        return Ok(default_codo_config);
    }

    // Parse the config file
    let codo_config = fs::read_to_string(&codo_config_file)?;
    let codo_config = YamlLoader::load_from_str(&codo_config)?[0].to_owned();
    return Ok(codo_config);
}

pub fn get_codo_config_dir() -> Option<path::PathBuf> {
    // Get the codo config dir
    let mut codo_config_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None =>  return None
    };
    codo_config_dir.push(".config");
    codo_config_dir.push("codo");

    // Make sure the config directory exists
    match fs::create_dir_all(&codo_config_dir) {
        Ok(_) => Some(codo_config_dir),
        Err(err) => {
            error!("Failed to create config directory: {:?}", err);
            None
        }
    }
}

pub fn get_image_config_dir<S: AsRef<str>>(image_name: S) -> Option<path::PathBuf> {
    // Get the image config dir
    let mut image_config_dir = match get_codo_config_dir() {
        Some(dir) => dir,
        None => return None
    };
    image_config_dir.push("images");
    image_config_dir.push(image_name.as_ref());

    // Check if the directory exists
    let image_config_path = path::Path::new(&image_config_dir);
    if image_config_path.exists() && image_config_path.is_dir() {
        return Some(image_config_dir);
    } else {
        return None;
    }
}
