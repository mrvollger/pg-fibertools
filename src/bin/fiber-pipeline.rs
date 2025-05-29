use assert_cmd::prelude::*;
use clap::Parser;
use log;
use std::fs::File;
use std::io::copy;
use std::process::{Child, Command, Stdio}; // Add methods on commands
// add anyhow for error handling
use anyhow::{Error, Result};
use std::os::unix::prelude::*;

#[derive(Parser, Debug)]
/// Execute a complex pipeline involving multiple tools.
pub struct FiberPipelineArgs {
    /// Input BAM file
    #[clap()]
    input: String,
    /// Output BAM file
    #[clap()]
    output: String,
    /// Number of threads to use
    #[clap(short, long, default_value_t = 8)]
    threads: usize,
    /// Write uncompressed output
    #[clap(short = 'u', long)]
    uncompressed: bool,
    /// Optional file to save the output of vg inject
    #[clap(long)]
    gam: Option<String>,
}

pub fn run_change_pan_spec(args: &FiberPipelineArgs) -> Child {
    Command::cargo_bin("change-pan-spec")
        .expect("change-pan-spec binary not found")
        .args(&["-t", &args.threads.to_string(), "-u", &args.input])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start change-pan-spec, step 1")
}

pub fn run_vg_inject(change_pan_spec: &mut Child, args: &FiberPipelineArgs) -> Child {
    // Step 2: vg inject
    let mut vg_inject = Command::new("vg")
        .args(&["inject", "-t", &args.threads.to_string()])
        .stdin(
            change_pan_spec
                .stdout
                .take()
                .expect("Failed to pipe output from change-pan-spec, step 1.5"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start vg inject, step 2");

    // Optionally save the output of vg inject to a file
    if let Some(gam_file) = &args.gam {
        let mut file = File::create(gam_file).expect("Failed to create gam file");
        copy(
            &mut vg_inject
                .stdout
                .as_mut()
                .expect("Failed to capture output of vg inject"),
            &mut file,
        )
        .expect("Failed to write output of vg inject to a GAM file");
    }
    vg_inject
}

pub fn run_vg_surject(vg_inject: &mut Child, args: &FiberPipelineArgs) -> Child {
    // Step 3: vg surject
    Command::new("vg")
        .args(&["surject", "-t", &args.threads.to_string()])
        .stdin(
            vg_inject
                .stdout
                .take()
                .expect("Failed to pipe output from vg inject"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start vg surject")
}

pub fn run_second_change_pan_spec(vg_surject: &mut Child, args: &FiberPipelineArgs) -> Child {
    // Step 4: change-pan-spec
    Command::cargo_bin("change-pan-spec")
        .expect("change-pan-spec binary not found")
        .args(&["-t", &args.threads.to_string(), "-u"])
        .stdin(
            vg_surject
                .stdout
                .take()
                .expect("Failed to pipe output from vg surject"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start second change-pan-spec")
}

pub fn run_samtools_sort(change_pan_spec_2: &mut Child, args: &FiberPipelineArgs) -> Child {
    // Step 5: samtools sort -N -u
    Command::new("samtools")
        .args(&["sort", "-N", "-u", "-@", &args.threads.to_string()])
        .stdin(
            change_pan_spec_2
                .stdout
                .take()
                .expect("Failed to pipe output from second change-pan-spec"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start samtools sort -N -u")
}

pub fn run_sync_tags(samtools_sort_1: &mut Child, args: &FiberPipelineArgs) -> Child {
    // Step 6: sync-tags
    Command::cargo_bin("sync-tags")
        .expect("sync-tags binary not found")
        .args(&[
            "-t",
            &args.threads.to_string(),
            "-",
            format!("samtools sort -N -u -@ {} {}", args.threads, &args.input).as_str(),
        ])
        .stdin(
            samtools_sort_1
                .stdout
                .take()
                .expect("Failed to pipe output from samtools sort -N -u"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start sync-tags")
}

pub fn run_final_samtools_sort(sync_tags: &mut Child, args: &FiberPipelineArgs) -> Child {
    // Step 7: samtools sort --write-index
    Command::new("samtools")
        .args(&[
            "sort",
            "-@",
            &args.threads.to_string(),
            "--write-index",
            "-o",
            &args.output,
        ])
        .stdin(
            sync_tags
                .stdout
                .take()
                .expect("Failed to pipe output from sync-tags"),
        )
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Failed to start samtools sort --write-index")
}

fn main() -> Result<(), anyhow::Error> {
    let args = FiberPipelineArgs::parse();

    // Set log level to info
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();
    log::info!(
        "Starting pipeline with input: {} and output: {}",
        args.input,
        args.output
    );

    // check and make sure samtools and vg are installed
    Command::new("samtools")
        .arg("--version")
        .output()
        .expect("samtools is not installed or not found in PATH");
    Command::new("vg")
        .arg("--version")
        .output()
        .expect("vg is not installed or not found in PATH");
    // ensure the input file path exists and that it is not a pipe
    let in_path = std::path::Path::new(&args.input);
    if !in_path.exists() {
        return Err(Error::msg(format!(
            "Input file does not exist: {}",
            args.input
        )));
    }

    // Step 1: change-pan-spec
    let mut change_pan_spec = run_change_pan_spec(&args);

    // Step 2: vg inject
    let mut vg_inject = run_vg_inject(&mut change_pan_spec, &args);

    // Step 3: vg surject
    let mut vg_surject = run_vg_surject(&mut vg_inject, &args);

    // Step 4: change-pan-spec
    let mut change_pan_spec_2 = run_second_change_pan_spec(&mut vg_surject, &args);

    // Step 5: samtools sort -N -u
    let mut samtools_sort_1 = run_samtools_sort(&mut change_pan_spec_2, &args);

    // Step 6: sync-tags
    let mut sync_tags = run_sync_tags(&mut samtools_sort_1, &args);

    // Step 7: samtools sort --write-index
    let mut samtools_sort_2 = run_final_samtools_sort(&mut sync_tags, &args);

    // Wait for all commands to finish
    change_pan_spec.wait().expect("change-pan-spec failed");
    vg_inject.wait().expect("vg inject failed");
    vg_surject.wait().expect("vg surject failed");
    change_pan_spec_2
        .wait()
        .expect("second change-pan-spec failed");
    samtools_sort_1.wait().expect("samtools sort -N -u failed");
    sync_tags.wait().expect("sync-tags failed");
    samtools_sort_2
        .wait()
        .expect("samtools sort --write-index failed");

    log::info!("Pipeline completed successfully.");
    Ok(())
}
