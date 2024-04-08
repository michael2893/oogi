use std::{env, io};
use std::io::Write;
use std::path;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::time::SystemTime;
use serde_yaml;

fn main() {
    welcome();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let argument = &args[1];
        if argument == "set_default" {
            set_defaults();
            return;
        } else if argument == "ssh" {
            match read_config() {
                Ok(config) => {
                    if has_non_empty_values(&config) {
                        ssh();
                    } else {
                        println!("woof.yml does not contain keys with non-empty values.");
                    }
                },
                Err(err) => {
                    eprintln!("Error reading woof.yml: {}", err);
                }
            }
        }
        }

    if check_adc_existence() {
        println!("Google Application Default Credentials found.");
    } else {
        println!("Google Application Default Credentials not found.");
        println!("Logging you in...");
        login_with_gcloud();
        println!("Successful");
    }
}


fn ssh() -> Result<(), String> {
    let config = match read_config() {
        Ok(config) => config,
        Err(err) => return Err(format!("Error reading woof.yml: {}", err)),
    };
    println!("{:?}", config);
    let default_project_id = "".to_string();
    let default_zone = "".to_string();
    let default_vm_name = "".to_string();

    let project_id = config.get("project_id").unwrap_or(&default_project_id);
    let zone = config.get("zone").unwrap_or(&default_zone);
    let vm_name = config.get("vm-name").unwrap_or(&default_vm_name);

    let command = format!(
        "compute ssh --project={} --zone={} {}",
        project_id, zone, vm_name
    );
    match run_gcloud_command(&command) {
        Ok(output) => println!("{}", output),
        Err(err) => return Err(format!("Failed to execute gcloud command: {}", err)),
    }

    Ok(())
}

fn run_gcloud_command(command: &str) -> Result<String, String> {
    let args: Vec<&str> = command.split_whitespace().collect();
    let output = std::process::Command::new("gcloud")
        .args(&args)
        .output()
        .map_err(|err| format!("Failed to execute gcloud command: {}", err))?;

    if output.status.success() {
        let output_string = String::from_utf8_lossy(&output.stdout);
        Ok(output_string.to_string())
    } else {
        let error_string = String::from_utf8_lossy(&output.stderr);
        println!("Command failed with status: {}", output.status);
        println!("Error output:\n{}", error_string);
        Err(format!("Command failed with status: {}", output.status))
    }
}

fn has_non_empty_values(config: &HashMap<String, String>) -> bool {
    for value in config.values() {
        if !value.is_empty() {
            return true;
        }
    }
    false
}

fn is_gcloud_installed() -> bool {
    return Command::new("gcloud").arg("--version").output().is_ok();
}

fn login_with_gcloud() {
    if !is_gcloud_installed() {
        println!("The 'gcloud' command is not installed.");
        println!("You'll need 'gcloud' to proceed. Here's how to install it:");
        println!("**On macOS:**");
        println!("1. Visit the Google Cloud SDK download page: https://cloud.google.com/sdk/docs/install");
        println!("2. Download the gcloud SDK for macOS (64-bit architecture).");
        println!("3. Extract the downloaded archive and follow the installation instructions.");
        println!("**OR**");
        println!("1. Install Homebrew (package manager): https://brew.sh/");
        println!("2. Run `brew install google-cloud-sdk` to install gcloud.");
        println!();
        println!("For detailed documentation, refer to the Google Cloud documentation:");
        println!("* Official Documentation: https://cloud.google.com/docs");
        println!();
    } else {
        run_gcloud_command("auth application-default login").expect("Failed to login with gcloud");
    }
}

fn set_defaults() {
    let config = match read_config() {
        Ok(data) => data,
        Err(error) => {
            println!("Error reading woof.yml: {}", error);
            return;
        }
    };

    if config.is_empty() {
        get_and_store_project_data();
    } else {
        println!("Default projects (from woof.yml):");
        for (key, value) in config.iter() {
            println!("* {}: {}", key, value);
        }
    }

    match set_project_id(&config) {
        Ok(_) => println!("Project ID set successfully"),
        Err(error) => eprintln!("Error setting project ID: {}", error),
    }

}


fn set_project_id(config: &HashMap<String, String>) -> Result<(), String> {
    let project_id = match config.get("home-project") {
        Some(id) => id,
        None => return Err("Project ID not found in configuration".to_string()),
    };

    let command = format!("config set project {}", project_id);

    match run_gcloud_command(&command) {
        Ok(output) => println!("Project ID set to {}: {}", project_id, output),
        Err(err) => return Err(format!("Failed to set project ID: {}", err)),
    }

    Ok(())
}
fn get_and_store_project_data() {
    let mut project_data = HashMap::new();

    get_project_data(&mut project_data, "SSH");
    get_project_data(&mut project_data, "home-project");

    let yaml_string = serde_yaml::to_string(&project_data).expect("Failed to serialize project data");
    let config_dir = path::PathBuf::from("~/.oogi");
    let config_path = config_dir.join("woof.yml");
    fs::create_dir_all(&config_dir).expect("Failed to create .oogi directory");
    fs::write(&config_path, yaml_string).expect("Failed to write woof.yml");
}

fn get_project_data(data: &mut HashMap<String, String>, project_type: &str) {
    print!("Enter the default project name for {}: ", project_type);
    io::stdout().flush().unwrap();
    let mut project_name = String::new();
    io::stdin().read_line(&mut project_name).expect("Failed to read project name");
    let project_name = project_name.trim();
    let project_name = if project_name.is_empty() {
        "".to_string()
    } else {
        project_name.to_string()
    };

    data.insert(project_type.to_string(), project_name);
}

fn read_config() -> Result<HashMap<String, String>, String> {
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => return Err("Failed to get home directory".to_string()),
    };

    let config_dir = home_dir.join(".oogi");
    let config_path = config_dir.join("woof.yml");

    if !config_path.exists() {
        return Err("woof.yml does not exist in ~/.oogi directory".to_string());
    }

    let contents = fs::read_to_string(&config_path)
        .expect("Failed to read woof.yml");

    let config: HashMap<String, String> = serde_yaml::from_str(&contents)
        .unwrap_or_else(|err| {
            eprintln!("Error: Failed to parse woof.yml: {}", err);  // More detailed error message
            HashMap::new()
        });

    Ok(config)
}


fn run_bash_command(command: &str) -> Result<String, String> {
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let output_string = String::from_utf8_lossy(&output.stdout);
        Ok(output_string.to_string())
    } else {
        let error_string = String::from_utf8_lossy(&output.stderr);
        println!("Command failed with status: {}", output.status);
        println!("Error output:\n{}", error_string);
        Err(format!("Command failed with status: {}", output.status))
    }
}
fn check_adc_existence() -> bool {
    let output = Command::new("sh")
        .arg("-c")
        .arg("[ -f ~/.config/gcloud/application_default_credentials.json ]")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        return match file_modified_within_24_hours("~/.config/gcloud/application_default_credentials.json") {
            Ok(true) => true,
            Ok(false) => {
                println!("The file was modified more than 24 hours ago, regenerating...");
                let cmd = run_gcloud_command("auth application-default login").expect("Failed to login with gcloud");
                !cmd.is_empty()
            }
            Err(err) => false
        }
    }
    true
}

fn file_modified_within_24_hours(file_path: &str) -> Result<bool, String> {
    match get_last_modified_time(file_path) {
        Ok(last_modified_time) => {
            let current_time = SystemTime::now();
            let twenty_four_hours_ago = current_time - std::time::Duration::from_secs(24 * 60 * 60);
            Ok(last_modified_time > twenty_four_hours_ago)
        }
        Err(err) => Err(err),
    }
}

fn get_last_modified_time(file_path: &str) -> Result<SystemTime, String> {
    let output = Command::new("stat")
        .arg("-f")
        .arg("%m")
        .arg(file_path)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let last_modified_time_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let last_modified_secs = last_modified_time_str.parse::<u64>().map_err(|e| format!("Failed to parse last modified time: {}", e))?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(last_modified_secs))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn welcome() {
    let welcome_msg = r#"
        /\_/\
       ( o.o )
       /  ^  \
      /  '-'  \
     /          \
    /            \   Welcome to Oogi:
   /  _         _ \  A GCP CLI tool written in Rust 🐶
  /| |  | |  | | | \
 /_|_/   \_/   \_|_\
   "#;
    println!("{}", String::from(welcome_msg));
    println!("\n\n");
}
