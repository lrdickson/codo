// Standard libraries
use std;
use std::env;

// Crates
use clap;
use log::error;
use log::debug;

// Internal
mod codo_error;
mod config;
mod image;

fn arg_value(matches: &clap::ArgMatches, input_command_index: usize, flag_name: &str, default_value: &str) -> String  {
    // Return the default value if the argument wasn't passed
    if !arg_passed(matches, input_command_index, flag_name) {
        return default_value.to_string();
    }

    // Get the flag value
    match matches.value_of(flag_name) {
        Some(value) => value.to_string(),
        None => {
            error!("Failed to get value for {}", flag_name);
            default_value.to_string()
        }
    }
}

fn arg_passed(matches: &clap::ArgMatches, input_command_index: usize, flag_name: &str) -> bool  {
    // Get the flag index
    let flag_index = match matches.index_of(flag_name) {
        Some(index) => index,
        None => return false
    };
    debug!("{} index: {}", flag_name, flag_index);

    // Flag only counts if it comes before the input command
    return flag_index < input_command_index;
}

fn main() {
    // Start the enviromental logger
    env_logger::init();

    // Get the flags
    let mut app = clap::App::new("codo")
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
             .help("Image of the container to run")
             .takes_value(true))
        .arg(clap::Arg::with_name("COMMAND")
             .help("Command to be run in the container")
             .multiple(true)
             .required(false)
             .index(1));

    // Determine if any arguments were passed
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        match app.print_help() {
            Ok(_) => (),
            Err(err) => error!("Failed to print help: {}", err)
        };
        println!();
        return;
    }

    // Get a vector of argmuments to be parsed
    let args_that_take_values = vec!["-i", "--image"];
    let mut args_for_clap: Vec<String> = Vec::new();
    
    // Get the command to be run
    let matches = app.get_matches();
    let mut input_command: Vec<String> = Vec::new();
    let input_command_index = match matches.index_of("COMMAND") {
        Some(index) => {
            input_command.append(&mut args[index..].to_vec());
            index
        },
        None => args.len(),
    };
    debug!("Input command: {:?}", input_command);
    debug!("Input command index: {}", input_command_index);

    // Get the image being used
    let codo_config = match config::codo_config() {
        Ok(ok) => ok,
        Err(err) => {
            println!("Failed to read config file: {}", err);
            return;
        }
    };
    let default_image_name = codo_config[config::DEFAULT_IMAGE]
                .as_str()
                .expect("Failed to get default image");
    let image_name = arg_value(&matches, input_command_index, "image", default_image_name);
    debug!("Image: {:?}", image_name);

    // Build the image if the build argument was passed
    let build_arg = arg_passed(&matches, input_command_index, "build");
    debug!("Build: {:?}", build_arg);
    if build_arg { 
        match image::build(&image_name) {
            Ok(_) => (),
            Err(err) => {
                println!("Failed to build image: {}", err);
                return;
            }
        }; 
    }

    // Return if not given a command to run
    if input_command.len() < 1 {
        debug!("No arguments passed. Exiting.");
        return;
    }

    // Build the container run command
    let mut command_contents: Vec<String> = vec!["sudo", "docker", "run", "-ti", "--rm"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Add binding to working directory
    let bind_param: String;
    match env::current_dir() {
        Ok(ok) => match ok.into_os_string().into_string() {
            Ok(working_dir) => {
                command_contents.push("-v".to_string());
                bind_param = format!("{}:/codo", working_dir);
                command_contents.push(bind_param);
                command_contents.push("-w".to_string());
                command_contents.push("/codo".to_string());
            },
            Err(err) => {
                error!("Failed to get working directory: {:?}", err);
            }
        },
        Err(err) => {
            error!("Failed to get working directory: {}", err);
        }
    };

    // Add the DISPLAY environmental variable
    let display_param: String;
    match env::var("DISPLAY") {
        Ok(display) => {
            command_contents.push("-e".to_string());
            display_param = format!("DISPLAY={}", display);
            command_contents.push(display_param);
            command_contents.push("-v".to_string());
            command_contents.push("/tmp/.X11-unix:/tmp/.X11-unix".to_string());
        }
        Err(err) => error!("Failed to get environment variable DISPLAY: {}", err)
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
    let image_with_tag = image::add_codo_tag(&image_name);
    let images_info = match image::images_info() {
        Ok(info) => info,
        Err(err) => {
            println!("Failed to get image info: {}", err);
            return;
        }
    };
    if !(images_info.contains_key(&image_with_tag)) {
        match image::build(&image_name) {
            Ok(_) => (),
            Err(err) => {
                println!("Failed to build image: {}", err);
                return;
            }
        }; 
    }
    command_contents.push(image_with_tag);
    
    // Add the input command
    command_contents.append(&mut input_command);

    // Start the container
    debug!("Running {:?}", command_contents);
    let inherit_io = true;
    match image::run_command(&command_contents, inherit_io) {
        Ok(_) => (),
        Err(err) => {
            println!("Failed to execute command: {}", err);
        }
    };
}
