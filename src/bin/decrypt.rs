use clap::Parser;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use weiji_decrypt::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Tar File path
    #[arg(short, long)]
    tar_path: Option<String>,

    /// List File path，将所有要解压的文件全部写进一个文件中，代替从stdin读取的方式
    #[arg(short, long)]
    list_path: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let tar_path = match args.tar_path {
        Some(path) => path,
        None => {
            let now = chrono::Local::now();
            now.format("%m%d-%H%M%S.tar.gz").to_string()
        }
    };

    // 根据args决定从文件读取加密文件路径信息还是从stdin读取
    let mut reader: Box<dyn tokio::io::AsyncBufRead + Unpin> = match args.list_path {
        Some(path) => {
            let file = tokio::fs::OpenOptions::new().read(true).open(path).await?;
            Box::new(tokio::io::BufReader::new(file))
        }
        None => Box::new(tokio::io::BufReader::new(tokio::io::stdin())),
    };
    // paths里的文件名都是处理过之后的，也就是已经加上了.png.exe的后缀
    let paths = Arc::new(Mutex::new(vec![]));
    let p = paths.clone();
    let f = move |filename: String| {
        let p = p.clone();
        async move {
            decrypt(filename.clone()).await?;
            p.lock().unwrap().push(convert_filename(&filename).1);
            Ok(())
        }
    };
    process(&mut reader, f).await?;
    let paths = paths.lock().unwrap();
    match create_tar_gz(&tar_path, &paths, |filename| {
        let path = std::path::Path::new(filename);
        revert_filename(path.to_str().unwrap())
    }) {
        Ok(_) => paths.iter().for_each(|filename| {
            std::fs::remove_file(filename)
                .map_err(|e| eprintln!("{}", e))
                .unwrap()
        }),
        Err(e) => {
            eprintln!("{}", e);
            paths.iter().for_each(|filename| {
                std::fs::rename(filename, revert_filename(filename))
                    .map_err(|e| eprintln!("{}", e))
                    .unwrap();
            })
        }
    }

    Ok(())
}

pub fn revert_filename(filename: &str) -> &str {
    &filename[..filename.len() - 8]
}

pub fn convert_filename(filename: &str) -> (String, String) {
    let png = format!("{}.png", filename);
    let exe = format!("{}.exe", png);
    (png, exe)
}

// 思路：
// 1. 将要解密的文件后缀改为png
// 2. 用伪装成ACDSeePro6.exe的命令将文件读出来，此时数据会自动解密
// 3. 读出解密的数据后将数据保存成exe格式，exe格式并不会被加密
// 4. 将exe格式的文件带走到自己的电脑上，并将文件名还原为源文件名
pub async fn decrypt(filename: String) -> std::io::Result<()> {
    let (png, exe) = convert_filename(&filename);
    // let current_dir = std::env::current_dir().unwrap();
    let filename = PathBuf::from(filename);
    let png = PathBuf::from(png);
    let exe = PathBuf::from(exe);
    // 这句也不知道为什么不行，可能用了加密系统文件并不存在于问题
    // tokio::fs::rename(filename, &png).await?;
    copy(&filename, &png).await?;
    copy(&png, &exe).await?;
    tokio::fs::remove_file(png).await?;
    tokio::fs::remove_file(filename).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_revert_filename() {
        let paths = vec!["target/test.txt.png.exe".to_string()];
        paths.iter().for_each(|filename| {
            std::fs::rename(filename, revert_filename(filename))
                .map_err(|e| eprintln!("{}", e))
                .unwrap();
        });
    }
}
