use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;
use tokio::io::AsyncBufReadExt;

pub async fn check_file_exist(filename: &str) -> bool {
    let metadata = tokio::fs::metadata(filename).await;
    metadata.is_ok()
}

pub async fn copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    let mut reader = tokio::fs::OpenOptions::new().read(true).open(src).await?;
    let mut writer = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(dst)
        .await?;
    tokio::io::copy(&mut reader, &mut writer).await?;
    Ok(())
}

pub fn create_tar_gz<F: Fn(&str) -> &str>(
    tar_path: &str,
    paths: &[String],
    convert_name_func: F,
) -> std::io::Result<()> {
    let tar_gz = std::fs::File::create(tar_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    for path in paths {
        println!("{}", path);
        tar.append_path_with_name(path, convert_name_func(path))?;
    }
    tar.finish()?;

    Ok(())
}

pub async fn process<'a, R, F, Fut>(reader: &'a mut R, func: F) -> std::io::Result<()>
where
    R: AsyncBufReadExt + Unpin,
    F: Fn(String) -> Fut + Send + Sync + 'static + Clone,
    Fut: std::future::Future<Output = std::io::Result<()>> + std::marker::Send,
{
    let mut set = tokio::task::JoinSet::new();
    while let Some(line) = reader.lines().next_line().await? {
        let func = func.clone();
        set.spawn(async move { func(line).await });
    }
    while let Some(res) = set.join_next().await {
        let out = res?;
        // 移除 .unwrap()，只打印错误，允许循环继续处理其他文件
        if let Err(e) = out {
            eprintln!("处理文件时出错: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
    #[tokio::test]
    async fn test_copy() {
        copy("Cargo.toml", "Cargo.toml.png").await.unwrap();
    }

    async fn myhandle(filename: String) -> std::io::Result<()> {
        println!("{}", filename);
        Ok(())
    }

    #[tokio::test]
    async fn test_process() {
        // let mut cursor = std::io::Cursor::new(b"lorem\nipsum\r\ndolor");
        let f = tokio::fs::OpenOptions::new()
            .read(true)
            .open("src/lib.rs")
            .await
            .unwrap();
        let mut f = tokio::io::BufReader::new(f);
        process(&mut f, myhandle).await.unwrap();
    }

    #[test]
    fn test_revert_filename() {
        let filename = "金刚北苑.skp";
        let (_, exe) = convert_filename(filename);
        let revert_fname = revert_filename(&exe);
        println!("{}", revert_fname);
        assert_eq!(filename, revert_fname);
    }

    #[test]
    fn test_create_tar_gz() {
        let mut v = vec![
            "/tmp/test/Cargo.toml".to_string(),
            "/tmp/test/.gitignore".to_string(),
        ];
        v.iter_mut().for_each(|s| *s = convert_filename(s).1);
        create_tar_gz("/tmp/test.tar.gz", &v, |filename| {
            let pos = filename.rfind('/');
            revert_filename(&filename[pos.unwrap() + 1..])
        })
        .unwrap();
    }
}
