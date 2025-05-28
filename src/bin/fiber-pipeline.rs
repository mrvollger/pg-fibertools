use clap::Parser;
use log;
use std::process::{Command, Stdio};

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
}

fn main() {
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

    // Step 1: change-pan-spec
    let mut change_pan_spec = Command::new("change-pan-spec")
        .args(&["-t", &args.threads.to_string(), "-u", &args.input])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start change-pan-spec");

    // Step 2: vg inject
    let mut vg_inject = Command::new("vg")
        .args(&["inject", "-t", &args.threads.to_string()])
        .stdin(
            change_pan_spec
                .stdout
                .take()
                .expect("Failed to pipe output from change-pan-spec"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start vg inject");

    // Step 3: vg surject
    let mut vg_surject = Command::new("vg")
        .args(&["surject", "-t", &args.threads.to_string()])
        .stdin(
            vg_inject
                .stdout
                .take()
                .expect("Failed to pipe output from vg inject"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start vg surject");

    // Step 4: change-pan-spec
    let mut change_pan_spec_2 = Command::new("change-pan-spec")
        .args(&["-t", &args.threads.to_string(), "-u"])
        .stdin(
            vg_surject
                .stdout
                .take()
                .expect("Failed to pipe output from vg surject"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start second change-pan-spec");

    // Step 5: samtools sort -N -u
    let mut samtools_sort_1 = Command::new("samtools")
        .args(&["sort", "-N", "-u", "-@", &args.threads.to_string()])
        .stdin(
            change_pan_spec_2
                .stdout
                .take()
                .expect("Failed to pipe output from second change-pan-spec"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start samtools sort -N -u");

    // Step 6: sync-tags
    let mut sync_tags = Command::new("sync-tags")
        .args(&[
            "-",
            &format!("<(samtools sort -N -u -@ {} {})", args.threads, args.input),
        ])
        .stdin(
            samtools_sort_1
                .stdout
                .take()
                .expect("Failed to pipe output from samtools sort -N -u"),
        )
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start sync-tags");

    // Step 7: samtools sort --write-index
    let mut samtools_sort_2 = Command::new("samtools")
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
        .expect("Failed to start samtools sort --write-index");

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
}
