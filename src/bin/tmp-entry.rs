use std::fs;
use std::io::prelude::*;
use std::process::{Command, Stdio};
use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    let decrypt_path = "decrypt.exe";
    let acdsee_path = "ACDSeePro6.exe";
    let target_path = "target";

    if fs::metadata(decrypt_path).is_ok() {
        fs::rename(decrypt_path, acdsee_path)?;
    }

    let mut child = Command::new(acdsee_path).stdin(Stdio::piped()).spawn()?;

    {
        let stdin = child.stdin.as_mut().unwrap();
        for entry in WalkDir::new(target_path) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                writeln!(stdin, "{}", entry.path().display())?;
            }
        }
    }

    let _ = child.wait()?;

    Ok(())
}
