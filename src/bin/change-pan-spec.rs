use clap::Parser;
use log;
use pg_fibertools::*;
use rust_htslib::bam::{self, Read};
use std::fmt::Debug;

// Args
#[derive(Parser, Debug)]
/// This program synchronizes tags between two BAM files. It applies any tags from the first BAM file to the second BAM file if the read names match and the tags do not already exist in the second BAM file and writes the output to stdout. The order of the reads must be the same in both BAM files. This can be done with, e.g. `samtools sort -N`.
pub struct ChangePanSpecArgs {
    /// First BAM file (source of tags)
    bam1: String,
    /// Second BAM file (tags will be updated)
    bam2: String,
    /// Output BAM file
    #[clap(short, long, default_value = "-")]
    output: String,
    /// Number of threads to use
    #[clap(short, long, default_value_t = 8)]
    threads: usize,
    /// Write uncompressed output
    #[clap(short = 'u', long)]
    uncompressed: bool,
    /// strip pan spec
    #[clap(short = 's', long)]
    strip_pan_spec: bool,
    /// pan spec delimiter
    #[clap(short = 'd', long, default_value = "#")]
    pan_spec_delimiter: String,
}

pub fn strip_pan_spec_header(header: &bam::Header, pan_spec_delimiter: &char) {
    //static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("([^#]+)#([^#]+)#(.+)").unwrap());
    let mut hash_map = header.to_hashmap();
    for (key, value) in hash_map.iter_mut() {
        if key.eq("SQ") {
            for sn_line in value.iter_mut() {
                let name = sn_line
                    .get_mut("SN")
                    .expect("SN tag not found within an @SQ line");
                let mut del_count = 0;
                let mut new_name = String::new();
                for char in name.chars() {
                    if del_count >= 2 {
                        new_name.push(char);
                    } else if char == *pan_spec_delimiter {
                        del_count += 1;
                    }
                }
                name.clear();
                name.push_str(&new_name);
            }
        }
    }
}

pub fn main() {
    let args = ChangePanSpecArgs::parse();

    // Set log level to info
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    // Open the first BAM file (source of tags)
    let mut bam1 = bam_reader_from_path_or_stdin(&args.bam1, args.threads);

    // Open the second BAM file (tags will be updated)
    let mut bam2 = bam_reader_from_path_or_stdin(&args.bam2, args.threads);

    // Create a writer for the output BAM file
    let header = bam2.header();
    let mut output_bam =
        bam_writer_from_path_or_stdout(&args.output, header, args.threads, args.uncompressed);

    // change the headers and reference names
}
