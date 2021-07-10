// Standard
use std::collections::HashMap;
use std::env;
use std::error;
use std::fs;
use std::path;
use std::process::{Command, Stdio};

// Crate
use log::error;
use log::debug;

// Internal
use crate::config;
use crate::codo_error;


pub fn build(image_name: &str) -> Result<(), Box<dyn error::Error>> {
    // Create the directory for the temporary dockerfile
    let mut temp_dockerfile_path = env::temp_dir();
    temp_dockerfile_path.push("codo");
    fs::create_dir_all(&temp_dockerfile_path)?;

    // Get the Dockerfile
    let dockerfile: String;
    let build_dir: path::PathBuf;

    // Get the image config directory
    match config::image_config_dir(image_name) {
        Some(mut image_config_dir) => {
            // Record the build directory
            build_dir = image_config_dir.clone();

            // Read the Dockerfile
            image_config_dir.push("CodoDockerfile");
            dockerfile = fs::read_to_string(&image_config_dir)?;
        },
        None => {
            // Set the build directory to the temporary dockerfile path
            build_dir = temp_dockerfile_path.clone();

            // Create a default dockerfile
            dockerfile = format!("FROM {}\n", image_name);
        }
    };

    // Write the final dockerfile
    temp_dockerfile_path.push("Dockerfile");
    fs::write(&temp_dockerfile_path, dockerfile)?;
    
    // Get the image tag
    let tag: String = format!("{}:{}", image_name, codo_tag());

    // Create the build command
    let temp_dockerfile_path = temp_dockerfile_path
        .into_os_string().into_string().expect("Failed to convert temp Dockerfile path to string");
    let build_dir = build_dir
        .into_os_string().into_string().expect("Failed to convert build directory to string");
    let build_command: Vec<String> = vec![
        "sudo".to_string(),
        "docker".to_string(),
        "build".to_string(),
        // Add the image tag
        "-t".to_string(),
        tag,
        // Add the path to the Dockerfile
        "-f".to_string(),
        temp_dockerfile_path,
        // Pull the latest image
        "--pull".to_string(),
        // Give the build directory
        build_dir
    ];

    // Run the build command
    Command::new(&build_command[0])
        .args(&build_command[1..])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    return Ok(());
}

pub fn codo_tag() -> String {
    let default_tag = "codo".to_string();
    match users::get_current_username() {
        Some(user) => {
            match user.into_string() {
                Ok(user) => format!("{}-{}", default_tag, user),
                Err(_) => {
                    error!("Failed to get username as string");
                    return default_tag;
                }
            }
        },
        None => { return default_tag; }
    }
}

pub fn images_info() -> Result<HashMap<String, HashMap<String, HashMap<String, String>>>, Box<dyn error::Error>> {
    // Run the command
    let images_info_command: Vec<&str> = vec!["sudo", "docker", "images"];
    let images_info = Command::new(&images_info_command[0]).args(&images_info_command[1..]).output()?;
    if !images_info.status.success() {
        let err: String = match images_info.status.code() {
            Some(code) => format!("Command {:?} failed with exit code {:?}.", images_info_command, code),
            None => format!("Command {:?} failed.", images_info_command),
        };
        let err = codo_error::Error::new(codo_error::ErrorKind::ContainerEngineFailure, &err);
        return Err(Box::new(err));
    }

    // Get the header line
    let images_info = String::from_utf8(images_info.stdout)?;
    let mut images_info = (&images_info).lines();
    let images_header: &str = match images_info.next() {
        Some(header) => header,
        None => {
            let err = format!("Failed to get header from {:?}", images_info_command);
            let err = codo_error::Error::new(codo_error::ErrorKind::ContainerEngineFailure, &err);
            return Err(Box::new(err));
        }
    };
    let images_header: Vec<String> = split_columns(images_header);
    debug!("Image info header: {:?}", images_header);

    // Turn into a vector of hash maps
    let images_info: Vec<HashMap<String, String>> = images_info.map(|s: &str| -> HashMap<String, String> {
        let mut m: HashMap<String, String> = HashMap::new();
        for (k, v) in images_header.iter().zip(split_columns(s).iter()) {
            m.insert(k.to_owned(), v.to_owned());
        }
        return m;
    }).collect();
    debug!("Image info vector of hashmaps: {:?}", images_info);

    // Sort out the images by repository and tag
    let mut repositories: HashMap<String, HashMap<String, HashMap<String, String>>> = HashMap::new();
    for image in images_info.iter() {
        // Check if there is a map for this repository
        if !repositories.contains_key(&image["REPOSITORY"]) {
            let new_repository = HashMap::new();
            repositories.insert(image["REPOSITORY"].to_owned(), new_repository);
        }
        repositories.get_mut(&image["REPOSITORY"])
            .unwrap()
            .insert(image["TAG"].to_owned(), image.to_owned());
    }

    // Return the image info
    return Ok(repositories);
}

fn split_columns(columns: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for column in columns.split("   ") {
        if column != "" {
            result.push(column.trim().to_string());
        }
    }

    return result;
}

