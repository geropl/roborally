use std::path::PathBuf;
use std::fs;
use std::ffi::OsStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_directory = PathBuf::from("../protocol");
    let mut proto_files: Vec<PathBuf> = vec![];
    for entry in fs::read_dir(&proto_directory)? {
        let entry = entry?;
        if let Some(ext) = entry.path().extension() {
            if ext == OsStr::new("proto") {
                let file_name = PathBuf::from(entry.path().file_name().unwrap());
                proto_files.push(file_name);
            }
        }
    }

    tonic_build::configure()
        .compile(proto_files.as_slice(), &[proto_directory])?;
    Ok(())
}