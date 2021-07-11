// Standard
use std::collections::HashMap;
use std::env;
use std::error;
use std::fs;
use std::path;
use std::process::{self, Command, Stdio};

// Crate
use log::error;
use log::debug;

// Internal
use crate::config;
use crate::codo_error;


pub fn add_codo_tag(image_name: &str) -> String {
    // Get the codo suffix
    let default_tag = "codo".to_string();
    let tag = match users::get_current_username() {
        Some(user) => {
            match user.into_string() {
                Ok(user) => format!("{}-{}", default_tag, user),
                Err(_) => {
                    error!("Failed to get username as string");
                    default_tag
                }
            }
        },
        None => default_tag
    };

    // Check if a tag was passed
    if image_name.to_string().contains(":") {
        format!("{}-{}", image_name, tag)
    } else {
        format!("{}:latest-{}", image_name, tag)
    }
}

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

    // Get the username
    let mut user_found = false;
    let username: String;
    let uid: users::uid_t = users::get_current_uid();
    let gid: users::uid_t;
    match users::get_user_by_uid(uid) {
        Some(user) => {
            username = match user.name().to_owned().into_string() {
                Ok(name) => name,
                Err(_) => {
                    error!("Failed to get username as string");
                    "user".to_string()
                }
            };
            gid = user.primary_group_id();
            user_found = true;
        }
        None => {
            error!("Failed to get user information");
            username = "".to_string();
            gid = 0;
        }
    };

    // Create the extended dockerfile
    let mut extended_dockerfile: String = dockerfile.to_owned();
    if user_found {
        extended_dockerfile.push_str(format!("
            RUN export uid={uid} gid={gid}
            RUN mkdir -p /home/{username}
            RUN echo \"{username}:x:${{uid}}:${{gid}}:{username},,,:/home/{username}:/bin/bash\" >> /etc/passwd
            RUN echo \"{username}:x:${{uid}}:\" >> /etc/group
            RUN echo \"{username} ALL=(ALL) NOPASSWD: ALL\" >> /etc/sudoers
            RUN chmod 0440 /etc/sudoers
            RUN chown ${{uid}}:${{gid}} -R /home/{username}
            USER {username}
            ENV HOME /home/{username}
            ", 
            username = username,
            uid = uid,
            gid = gid).as_str());
    }
    debug!("Building Dockerfile: \n {}", extended_dockerfile);

    // Write the final dockerfile
    temp_dockerfile_path.push("Dockerfile");
    fs::write(&temp_dockerfile_path, extended_dockerfile)?;
    
    // Get the image tag
    let image_with_tag = add_codo_tag(image_name);

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
        image_with_tag,
        // Add the path to the Dockerfile
        "-f".to_string(),
        temp_dockerfile_path,
        // Pull the latest image
        "--pull".to_string(),
        // Give the build directory
        build_dir
    ];

    // Run the build command
    let inherit_io = true;
    run_command(&build_command, inherit_io)?;

    return Ok(());
}

pub fn images_info() -> Result<HashMap<String, HashMap<String, String>>, Box<dyn error::Error>> {
    // Run the command
    let images_info_command: Vec<String> = vec!["sudo".to_string(), "docker".to_string(), "images".to_string()];
    let inherit_io = false;
    let images_info = run_command(&images_info_command, inherit_io)?;

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

    // Create a map of the image info
    let mut images_info_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for image in images_info.iter() {
        let key = format!("{}:{}", image["REPOSITORY"], image["TAG"]);
        images_info_map.insert(key, image.to_owned());
    }
    /*
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
    */

    // Return the image info
    return Ok(images_info_map);
}

pub fn run_command(command: &Vec<String>, inherit_io: bool) -> Result<process::Output, Box<dyn error::Error>> {
    // Run the build command
    let output: process::Output;
    if inherit_io {
        output = Command::new(&command[0])
            .args(&command[1..])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
    } else {
        output = Command::new(&command[0])
            .args(&command[1..])
            .output()?;
    }

    // Check if the build was a success
    if !output.status.success() {
        let err: String = match output.status.code() {
            Some(code) => format!("Command {:?} failed with exit code {:?}.", command, code),
            None => format!("Command {:?} failed.", command),
        };
        let err = codo_error::Error::new(codo_error::ErrorKind::ContainerEngineFailure, &err);
        return Err(Box::new(err));
    }

    return Ok(output);
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

