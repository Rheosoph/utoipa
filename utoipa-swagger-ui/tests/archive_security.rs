#[path = "../build/archive.rs"]
mod archive;

use std::io::{Cursor, Write};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

fn swagger_archive() -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, contents) in [
        ("swagger-ui/dist/first.js", "first"),
        ("swagger-ui/dist/second.js", "second"),
        ("swagger-ui/README.md", "excluded"),
    ] {
        writer.start_file(name, options).unwrap();
        writer.write_all(contents.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn extracts_valid_distribution_files() {
    let output = tempfile::tempdir().unwrap();
    let mut zip = ZipArchive::new(Cursor::new(swagger_archive())).unwrap();
    let top_level = archive::extract_dist(&mut zip, output.path().to_str().unwrap()).unwrap();

    assert_eq!(top_level, "swagger-ui");
    assert_eq!(
        std::fs::read_to_string(output.path().join("swagger-ui/dist/first.js")).unwrap(),
        "first"
    );
    assert_eq!(
        std::fs::read_to_string(output.path().join("swagger-ui/dist/second.js")).unwrap(),
        "second"
    );
    assert!(!output.path().join("swagger-ui/README.md").exists());
}

#[test]
fn rejects_inconsistent_later_header_before_writing_any_files() {
    let mut bytes = swagger_archive();
    let second_header = {
        let mut zip = ZipArchive::new(Cursor::new(&bytes)).unwrap();
        let offset = zip.by_index(1).unwrap().header_start();
        offset as usize
    };
    // Change the local filename while retaining the original central directory name.
    bytes[second_header + 30] ^= 1;
    let mut zip = ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(
        zip.by_index(1).unwrap().name().unwrap(),
        "swagger-ui/dist/second.js"
    );
    let output = tempfile::tempdir().unwrap();

    assert!(archive::extract_dist(&mut zip, output.path().to_str().unwrap()).is_err());
    assert_eq!(std::fs::read_dir(output.path()).unwrap().count(), 0);
}
