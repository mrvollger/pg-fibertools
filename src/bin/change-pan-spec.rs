use clap::Parser;
use log;
use pg_fibertools::*;
use rust_htslib::bam::{self, Header, Read};
use std::fmt::Debug;

#[derive(Parser, Debug)]
/// TODO
pub struct ChangePanSpecArgs {
    /// First BAM file (source of tags)
    bam: String,
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
    pan_spec_delimiter: char,
    /// Add a pan spec prefix to the BAM, e.g. GM12878#1#, or CHM13#0#
    #[clap(short = 'p', long)]
    pan_spec_prefix: Option<String>,
}

pub fn strip_pan_spec_header(header: &bam::Header, pan_spec_delimiter: &char) -> Header {
    let mut hash_map: std::collections::HashMap<
        String,
        Vec<linear_map::LinearMap<String, String>>,
    > = header.to_hashmap();
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
    header_from_hashmap(hash_map)
}

pub fn add_pan_spec_header(header: &bam::Header, pan_spec_prefix: &str) -> Header {
    let mut hash_map = header.to_hashmap();
    for (key, value) in hash_map.iter_mut() {
        if key.eq("SQ") {
            for sn_line in value.iter_mut() {
                let name = sn_line
                    .get_mut("SN")
                    .expect("SN tag not found within an @SQ line");
                let mut new_name = String::new();
                new_name.push_str(pan_spec_prefix);
                new_name.push_str(name);
                name.clear();
                name.push_str(&new_name);
            }
        }
    }
    header_from_hashmap(hash_map)
}

pub fn main() {
    let args = ChangePanSpecArgs::parse();

    // Set log level to info
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();

    // Open the first BAM file (source of tags)
    let mut bam = bam_reader_from_path_or_stdin(&args.bam, args.threads);

    // Create a writer for the output BAM file
    let mut header = bam::Header::from_template(bam.header());
    // update the header to change the pan spec of the reference names
    if args.strip_pan_spec {
        // strip the pan spec from the header
        header = strip_pan_spec_header(&header, &args.pan_spec_delimiter);
    } else if let Some(pan_spec_prefix) = &args.pan_spec_prefix {
        // add the pan spec prefix to the header
        header = add_pan_spec_header(&header, pan_spec_prefix);
    }
    let mut output_bam = bam_writer_from_header_and_path_or_stdout(
        &args.output,
        &mut header,
        args.threads,
        args.uncompressed,
    );

    // change the pan spec in each read
    for record in bam.records() {
        let mut record = record.expect("Failed to read record");
        if args.strip_pan_spec {
            // remove the pan spec from the read name
            todo!()
        } else if let Some(pan_spec_prefix) = &args.pan_spec_prefix {
            // add the pan spec prefix to the read name
            todo!()
        }
        output_bam.write(&record).expect("Failed to write record");
    }
}
