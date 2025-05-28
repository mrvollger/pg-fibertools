use clap::Parser;
use log;
use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn pretty_cmd(cmd: &Command) -> String {
    format!(
        "{} {:?}",
        cmd.get_envs()
            .map(|(key, val)| format!("{:?}={:?}", key, val))
            .fold(String::new(), |a, b| a + &b),
        cmd
    )
}

#[derive(Parser, Debug)]
/// Pipe the output of three command-line tools: a -> b -> c.
/// Optionally save the output of b to a file.
pub struct FiberJectArgs {
    /// Bam file to process
    #[clap(short, long, default_value = "-")]
    bam: String,
    #[clap(short, long, default_value = "-")]
    out: String,
    // Command b
    //#[clap(short = 'b', long)]
    //command_b: String,
    /// Command c
    //#[clap(short = 'c', long)]
    //command_c: String,
    /// File to save the output of command injection (GAM)
    #[clap(short, long)]
    gam: Option<String>,
}

fn main() {
    let args = FiberJectArgs::parse();

    // Set log level to info
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    // the commands to run
    let mut cmd_inject = Command::new("time");
    let mut cmd_surject = Command::new("cat");
    let mut cmd_clean = Command::new("grep Cargo");

    // Run inject command
    cmd_inject.arg(&args.bam).stdout(Stdio::piped());
    let cmd_inject_str = pretty_cmd(&cmd_inject);
    let mut process_inject = cmd_inject
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to run: {}", e));
    log::info!("Running command: {}", cmd_inject_str);

    // Run surject command with the output of inject command
    cmd_surject
        .stdin(
            process_inject
                .stdout
                .take()
                .unwrap_or_else(|| panic!("Failed to pipe output from command")),
        )
        .stdout(Stdio::piped());
    let cmd_surject_str = pretty_cmd(&cmd_surject);
    log::info!("Running command: {}", cmd_surject_str);
    // execute the command
    let mut process_surject = cmd_surject
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to run: {}", e));

    // Optionally save the output injection to a GAM file
    if let Some(output_file) = &args.gam {
        let mut file = File::create(output_file).expect("Failed to create output file");
        std::io::copy(
            &mut process_surject
                .stdout
                .as_mut()
                .expect("Failed to capture output of command b"),
            &mut file,
        )
        .expect("Failed to write output of command b to file");
    }

    // Run command clean
    cmd_clean.stdin(
        process_surject
            .stdout
            .take()
            .unwrap_or_else(|| panic!("Failed to pipe output from command")),
    );
    //.stdout(Stdio::inherit());
    let cmd_clean_str = pretty_cmd(&cmd_clean);
    log::info!("Running command: {}", cmd_clean_str);
    // execute the command
    let process_clean = cmd_clean
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to run: {}", e));

    // Wait for all commands to finish
    for p in &mut [process_inject, process_surject, process_clean] {
        p.wait().unwrap_or_else(|e| panic!("Error: {}", e));
    }

    log::info!("Pipeline completed successfully.");
}
