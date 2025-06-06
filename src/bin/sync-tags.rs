use clap::Parser;
use log;
use pg_fibertools::*;
use rust_htslib::bam::Read;

#[derive(Parser)]
/// This program synchronizes tags between two BAM files. It applies any tags from the first BAM file to the second BAM file if the read names match and the tags do not already exist in the second BAM file and writes the output to stdout. The order of the reads must be the same in both BAM files. This can be done with, e.g. `samtools sort -N`.
pub struct SyncTagsArgs {
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
}

fn main() {
    let args = SyncTagsArgs::parse();

    // Set log level to info
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .init();
    /*
    log::debug!("debug is not as important as info");
    log::info!("hello world");
    log::warn!("this is a bad spot to be in");
    log::error!("this is a bad spot to be in");
     */

    // Open the first BAM file (source of tags)
    let mut bam1 = bam_reader_from_path_or_stdin(&args.bam1, args.threads);

    // Open the second BAM file (tags will be updated)
    let mut bam2 = bam_reader_from_path_or_stdin(&args.bam2, args.threads);

    // Create a writer for the output BAM file
    let header = bam2.header();
    let mut output_bam =
        bam_writer_from_path_or_stdout(&args.output, header, args.threads, args.uncompressed);

    let bam1_iter = bam1.records();
    let mut bam2_iter = bam2.records();
    let mut destination_rec = if let Some(destination_rec) = bam2_iter.next() {
        destination_rec.expect("Failed to read record from second BAM file")
    } else {
        log::error!("No records in the second BAM file.");
        return;
    };
    let mut dest_record_was_synced = false;

    // Iterate over the first BAM file
    for template_rec in bam1_iter {
        let template_rec = template_rec.expect("Failed to read record from first BAM file");
        // Iterate over the second BAM file
        while template_rec.qname() == destination_rec.qname() {
            dest_record_was_synced = true;
            // Move the tags from the template record to the next read
            // key could be MM, and value could be anything A+a,0,1,0,2,0,2,0,2,0
            for (key, value) in template_rec
                .aux_iter()
                .map(|x| x.expect("Unable to read tag from template BAM file"))
            {
                if !destination_rec.aux(key).is_ok() {
                    destination_rec
                        .push_aux(key, value)
                        .expect("Failed to push tag to next read");
                }
            }

            // Write the updated read to the output BAM file
            output_bam
                .write(&destination_rec)
                .expect("Failed to write record to output BAM file");

            // get the next read
            destination_rec = if let Some(next_read) = bam2_iter.next() {
                next_read.expect("Failed to read record from second BAM file")
            } else {
                log::warn!("No more records in the second BAM file.");
                break;
            };
            dest_record_was_synced = false;

            // TODO check that each dest record has been synced using dest_record_was_synced
        }
    }

    log::info!("Tags successfully synchronized and written to output BAM file.");
}
