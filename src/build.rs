// Standard
use std::error;
use std::fs;
use std::io;
use std::path;
use std::process::{Command, Stdio};

// Crate
use log::error;

// Internal
use crate::config;

pub fn build_image<S: AsRef<str>>(image_name: S) -> Result<(), Box<dyn error::Error>> {
    let image_name = image_name.as_ref();

    // Get the image config directory
    let image_config_dir = match config::get_image_config_dir(image_name) {
        Some(value) => value,
        None => {
            let err = io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to get config directory for {}", image_name)); 
            return Err(Box::new(err));
        }
    };

    // Get the Dockerfile
    let mut dockerfile = image_config_dir.clone();
    dockerfile.push("CodoDockerfile");
    let dockerfile = fs::read_to_string(&dockerfile)?;

    // Create the directory for the temporary dockerfile
    let mut temp_dockerfile_path = path::PathBuf::new();
    temp_dockerfile_path.push("tmp");
    temp_dockerfile_path.push("codo");
    fs::create_dir_all(&temp_dockerfile_path)?;

    // Write the final dockerfile
    temp_dockerfile_path.push("Dockerfile");
    fs::write(&temp_dockerfile_path, dockerfile)?;
    
    // Get the image tag
    let mut tag: String = format!("{}:codo", image_name);
    match users::get_current_username() {
        Some(user) => {
            match user.into_string() {
                Ok(ok) => tag.push_str(ok.as_str()),
                Err(_) => error!("Failed to get username as string")
            };
        },
        None => ()
    };

    // Create the build command
    let temp_dockerfile_path = temp_dockerfile_path
        .into_os_string().into_string().expect("Failed to convert temp Dockerfile path to string");
    let image_config_dir = image_config_dir
        .into_os_string().into_string().expect("Failed to convert image config directory to string");
    let build_command: Vec<String> = vec![
        "sudo".to_string(),
        "docker".to_string(),
        "build".to_string(),
        "-t".to_string(),
        tag,
        "-f".to_string(),
        temp_dockerfile_path,
        image_config_dir
    ];

    // Run the build command
    match Command::new(&build_command[0])
        .args(&build_command[1..])
        .stdout(Stdio::inherit())
        .output() {
        Ok(_) => Ok(()),
        Err(err) => {
            Err(Box::new(err))
        }
    }
}

