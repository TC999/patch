use std::fs;
use std::io;

pub fn safe_open(path: &str) -> io::Result<fs::File> {
    // 检查路径合法性等
    fs::File::open(path)
}