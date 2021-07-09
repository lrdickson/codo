// Standard
use std::collections::HashMap;
use std::error;
use std::fs;
use std::io;
use std::path;
use std::process::{Command, Stdio};

// Crate
use log::error;

// Internal
use crate::config;
use crate::codo_error;


pub fn build<S: AsRef<str>>(image_name: S) -> Result<(), Box<dyn error::Error>> {
    let image_name = image_name.as_ref();

    // Get the image config directory
    let image_config_dir = match config::image_config_dir(image_name) {
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
                Ok(ok) => tag.push_str(&format!("-{}", ok)),
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
        // Add the image tag
        "-t".to_string(),
        tag,
        // Add the path to the Dockerfile
        "-f".to_string(),
        temp_dockerfile_path,
        // Pull the latest image
        "--pull".to_string(),
        // Give the build directory
        image_config_dir
    ];

    // Run the build command
    Command::new(&build_command[0])
        .args(&build_command[1..])
        .stdout(Stdio::inherit())
        .output()?;

    return Ok(());
}

fn images_info() -> Result<HashMap<String, HashMap<String, String>>, Box<dyn error::Error>> {
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

    // Turn into a vector of hash maps
    let images_info: Vec<HashMap<String, String>> = images_info.map(|s: &str| -> HashMap<String, String> {
        let mut m: HashMap<String, String> = HashMap::new();
        for (k, v) in split_columns(s).iter().zip(images_header.iter()) {
            m.insert(k.to_owned(), v.to_owned());
        }
        return m;
    }).collect();
    let mut images_info_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for image in images_info.iter() {
        let key = format!("{}:{}", image["REPOSITORY"], image["TAG"]);
        images_info_map.insert(key, image.to_owned());
    }

    return Ok(images_info_map);
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

