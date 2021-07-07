// Standard
use std::fs;

// Internal
mod config;

pub fn build_image<S: AsRef<str>>(image_name: S) -> () {
    let image_name = image_name.as_ref();

    // Get the image config directory
    let image_config_dir = match config::get_image_config_dir(image_name) {
        Some(value) => value,
        None => {
            println!("Unable to get the config directory for {:?}", image_name);
            return;
        }
    };

    // Get the Dockerfile
    let mut dockerfile = image_config_dir.clone();
    dockerfile.push("CodoDockerfile");
    let dockerfile = match fs::read_to_string(&dockerfile) {
        Ok(ok) => ok,
        Err(err) => {}
    };
}

