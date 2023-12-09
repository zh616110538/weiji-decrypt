use std::fs;
use std::io::prelude::*;
use std::process::{Command, Stdio};

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
        visit_dirs(std::path::Path::new(target_path), stdin)?;
    }

    let _ = child.wait()?;

    Ok(())
}

fn visit_dirs(dir: &std::path::Path, stdin: &mut std::process::ChildStdin) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, stdin)?;
            } else {
                // 在这里处理文件，比如打印它的路径
                writeln!(stdin, "{}", entry.path().display())?;
            }
        }
    }
    Ok(())
}
