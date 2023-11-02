use clap::Parser;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The output directory
    java_out_dir: String,
    /// Build the project in release mode
    #[arg(long, short, default_value = "false")]
    release: bool,
    /// Print the generated code
    #[arg(long, default_value = "false")]
    print_code: bool,
    /// The arguments to pass to cargo
    cargo_args: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let mut cargo_args = vec!["build".to_string()];
    if args.release {
        cargo_args.push("--release".to_string());
    }

    let mut command = Command::new("cargo");
    if args.print_code {
        command.env("DEBUG_JNI_BINDGEN", "true");
    }

    command
        .args(cargo_args)
        .env("JNI_BINDGEN_OUT_DIR", &args.java_out_dir)
        .args(&args.cargo_args)
        .status()
        .expect("Failed to build the project");
}
