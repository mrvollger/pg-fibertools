use rust_htslib::bam::{self, Read};

pub fn bam_reader_from_path_or_stdin(path: &str, threads: usize) -> bam::Reader {
    let mut bam = if path == "-" {
        bam::Reader::from_stdin().expect("Failed to open BAM file from stdin")
    } else {
        bam::Reader::from_path(path).expect("Failed to open BAM file from path")
    };
    bam.set_threads(threads)
        .expect("Failed to set threads for BAM reader");
    bam
}

pub fn bam_writer_from_path_or_stdout(
    path: &str,
    header: &bam::HeaderView,
    threads: usize,
    uncompressed: bool,
) -> bam::Writer {
    let mut header = bam::Header::from_template(header);
    // add a PG line to the header
    let mut pg_line = bam::header::HeaderRecord::new(b"PG");
    pg_line.push_tag(b"ID", "sync-tags");
    pg_line.push_tag(b"PN", "sync-tags");
    pg_line.push_tag(b"VN", env!("CARGO_PKG_VERSION"));
    // get the full command line call as a string
    let full_cmd = std::env::args()
        .map(|arg| arg.replace(' ', "\\ "))
        .collect::<Vec<String>>()
        .join(" ");
    pg_line.push_tag(b"CL", full_cmd);
    header.push_record(&pg_line);

    let mut writer = if path == "-" {
        bam::Writer::from_stdout(&header, bam::Format::Bam)
            .expect("Failed to create BAM writer for stdout")
    } else {
        bam::Writer::from_path(path, &header, bam::Format::Bam)
            .expect("Failed to create BAM writer from path")
    };
    writer
        .set_threads(threads)
        .expect("Failed to set threads for BAM writer");
    if uncompressed {
        writer
            .set_compression_level(bam::CompressionLevel::Uncompressed)
            .expect("Failed to set compression level");
    }
    writer
}
