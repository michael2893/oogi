use std::env;
use std::process::Command;

fn main() {
    welcome();
    login_with_gcloud();
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
        println!("2. Run `brew cask install google-cloud-sdk` to install gcloud.");
        println!();
        println!("For detailed documentation, refer to the Google Cloud documentation:");
        println!("* Official Documentation: https://cloud.google.com/docs");
        println!();
    } else {
        run_command("gcloud", "auth application-default login").expect("Failed!");
    }
}

fn parse_identity() -> String {
    let args: Vec<String> = env::args().collect();
    let id = &args[0];
    return id.parse().unwrap();
}

fn run_command(program: &str, command: &str) -> Result<String, String> {
    let mut gcloud_command = Command::new(program);
    gcloud_command
        .arg("auth")
        .arg("application-default")
        .arg("login");

    let output = gcloud_command.output().expect("Failed to execute command");

    if output.status.success() {
        let output_string = String::from_utf8_lossy(&output.stdout);
        return Ok(output_string.to_string());
    } else {
        // Convert error output to String
        let error_string = String::from_utf8_lossy(&output.stderr);
        println!(" command failed with status: {}", output.status);
        println!("Error output:\n{}", error_string);
        return Err(format!(" command failed with status: {}", output.status));
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
