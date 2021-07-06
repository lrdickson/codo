// Standard libraries
use std;
use std::env;
use std::process::{Command, Stdio};

// Crates
use clap;
use log::error;
use log::debug;

// Internal
mod config;

fn main() {
    // Start the enviromental logger
    env_logger::init();

    // Get the flags
    let matches = clap::App::new("codo")
        .version("0.1")
        .author("Lyndsey R. M. Dickson")
        .about("Runs a single command in a container")
        .arg(clap::Arg::with_name("build")
             .short("b")
             .long("build")
             .help("Build the selected image")
             .takes_value(false))
        .arg(clap::Arg::with_name("image")
             .short("i")
             .long("image")
             .help("Image to build container out of")
             .takes_value(true))
        .arg(clap::Arg::with_name("COMMAND")
             .help("Command to be run in the container")
             .multiple(true)
             .required(false)
             .index(1))
        .get_matches();

    // Get the image being used
    let codo_config = &config::get_codo_config()[0];
    let image_name = match matches.value_of("image") {
        Some(value) => value,
        None => {
            codo_config[config::DEFAULT_IMAGE]
                .as_str()
                .expect("Failed to get default image")
        }
    };
    debug!("Image: {:?}", image_name);

    // Get the command to be run
    let mut input_command: Vec<&str> = match matches.values_of("COMMAND") {
        Some(values) => values.collect(),
        None => Vec::new()
    };
    debug!("Input command: {:?}", input_command);

    // Check if any arguments were passed
    if input_command.len() < 1 {
        debug!("No arguments passed. Exitting.");
        return;
    }

    // Build the container run command
    let mut command_contents: Vec<&str> = vec!["sudo", "docker", "run", "-ti", "--rm"];

    // Add binding to working directory
    let mut bind_working_dir = true;
    let working_dir = match env::current_dir() {
        Ok(ok) => match ok.into_os_string().into_string() {
            Ok(ok) => ok,
            Err(err) => {
                error!("Failed to get working directory: {:?}", err);
                bind_working_dir = false;
                "".to_string()
            }
        },
        Err(err) => {
            error!("Failed to get working directory: {:?}", err);
            bind_working_dir = false;
            "".to_string()
        }
    };
    let bind_param: String;
    if bind_working_dir {
        command_contents.push("-v");
        bind_param = format!("{}:/codo", working_dir);
        command_contents.push(bind_param.as_str());
        command_contents.push("-w");
        command_contents.push("/codo");
    }

    // Add the image name
    let full_image_name = image_name;
    command_contents.push(full_image_name);
    
    // Add the input command
    command_contents.append(&mut input_command);

    // Start the container
    debug!("Running {:?}", command_contents);
    match Command::new(command_contents[0])
        .args(&command_contents[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output() {
        Ok(result) => result,
        Err(err) => {
            error!("Failed to execute command: {:?}", err);
            return;
        }
    };
}
