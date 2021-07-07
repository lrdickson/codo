// Standard libraries
use std;
use std::env;
use std::process::{Command, Stdio};

// Crates
use clap;
use log::error;
use log::debug;

// Internal
mod build;
mod config;

fn get_arg_value<'a>(matches: &'a clap::ArgMatches, input_command_index: usize, flag_name: &'a str, default_value: &'a str) -> &'a str  {
    // Return the default value if the argument wasn't passed
    if arg_passed(matches, input_command_index, flag_name) {
        return default_value;
    }

    // Get the flag value
    match matches.value_of(flag_name) {
        Some(value) => value,
        None => {
            error!("Failed to get value for {:?}", flag_name);
            default_value
        }
    }
}

fn arg_passed<'a>(matches: &'a clap::ArgMatches, input_command_index: usize, flag_name: &'a str) -> bool  {
    // Get the flag index
    let flag_index = match matches.index_of(flag_name) {
        Some(index) => index,
        None => return false
    };

    // Flag only counts if it comes before the input command
    return flag_index > input_command_index;
}

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

    // Get the command to be run
    let args: Vec<String> = env::args().collect();
    let mut input_command: Vec<&str> = Vec::new();
    let input_command_index = match matches.index_of("COMMAND") {
        Some(index) => {
            input_command = vec![];
            for s in args[index..].iter() { input_command.push(s.as_str()); }
            index
        },
        None => args.len(),
    };
    debug!("Input command: {:?}", input_command);

    // Get the image being used
    let codo_config = match config::get_codo_config() {
        Ok(ok) => ok,
        Err(err) => {
            println!("Failed to read config file: {:?}", err);
            return;
        }
    };
    let default_image_name = codo_config[config::DEFAULT_IMAGE]
                .as_str()
                .expect("Failed to get default image");
    let image_name = get_arg_value(&matches, input_command_index, "image", default_image_name);
    debug!("Image: {:?}", image_name);

    // Determine if the image needs be built
    let build_image = arg_passed(&matches, input_command_index, "build");
    debug!("Build: {:?}", build_image);
    if build_image { build::build_image(image_name); }

    // Check if any arguments were passed
    if input_command.len() < 1 {
        debug!("No arguments passed. Exiting.");
        return;
    }

    // Build the container run command
    let mut command_contents: Vec<&str> = vec!["sudo", "docker", "run", "-ti", "--rm"];

    // Add binding to working directory
    let bind_param: String;
    match env::current_dir() {
        Ok(ok) => match ok.into_os_string().into_string() {
            Ok(working_dir) => {
                command_contents.push("-v");
                bind_param = format!("{}:/codo", working_dir);
                command_contents.push(bind_param.as_str());
                command_contents.push("-w");
                command_contents.push("/codo");
            },
            Err(err) => {
                error!("Failed to get working directory: {:?}", err);
            }
        },
        Err(err) => {
            error!("Failed to get working directory: {:?}", err);
        }
    };

    /*
    // Create the storage directory
    let home_dir: path::PathBuf;
    match dirs::home_dir() {
        Some(dir) => {
            let mut image_storage_dir = dir;
            image_storage_dir.push("codo");
            image_storage_dir.push(image_name);
            match fs::create_dir_all(image_storage_dir) {
                Ok(ok) => (),
                Err(err) => ()
            }
        },
        None => ()
    };
    */

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
