use std::fs::{self, File, OpenOptions, Metadata};
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// 是否允许不安全路径（如访问工作目录以外的文件）
pub static mut UNSAFE: bool = false;

/// 检查路径是否安全（不包含绝对路径或 ..）
pub fn is_safe_path(path: &Path) -> bool {
    if path.is_absolute() {
        return false;
    }
    for comp in path.components() {
        use std::path::Component::*;
        if let ParentDir = comp {
            return false;
        }
    }
    true
}

/// 安全地打开文件
pub fn safe_open<P: AsRef<Path>>(path: P, write: bool) -> io::Result<File> {
    let path = path.as_ref();
    unsafe {
        if !UNSAFE && !is_safe_path(path) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("不允许访问不安全路径: {:?}", path)
            ));
        }
    }

    let mut opts = OpenOptions::new();
    opts.read(!write).write(write);

    // unix: O_NOFOLLOW 可防止符号链接攻击
    #[cfg(unix)]
    {
        opts.custom_flags(libc::O_NOFOLLOW);
    }

    opts.open(path)
}

/// 安全读取符号链接内容
pub fn safe_readlink<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let path = path.as_ref();
    unsafe {
        if !UNSAFE && !is_safe_path(path) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("不允许访问不安全路径: {:?}", path)
            ));
        }
    }
    fs::read_link(path).map(|p| p.to_string_lossy().into_owned())
}

/// 安全删除文件
pub fn safe_unlink<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    unsafe {
        if !UNSAFE && !is_safe_path(path) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("不允许删除不安全路径: {:?}", path)
            ));
        }
    }
    if path.is_dir() {
        fs::remove_dir(path)
    } else {
        fs::remove_file(path)
    }
}

/// 安全创建目录
pub fn safe_mkdir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref();
    unsafe {
        if !UNSAFE && !is_safe_path(path) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("不允许创建不安全路径: {:?}", path)
            ));
        }
    }
    fs::create_dir_all(path)
}

/// 安全获取文件状态
pub fn safe_stat<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    let path = path.as_ref();
    unsafe {
        if !UNSAFE && !is_safe_path(path) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("不允许stat不安全路径: {:?}", path)
            ));
        }
    }
    fs::metadata(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_safe_path() {
        assert!(is_safe_path(Path::new("foo/bar")));
        assert!(!is_safe_path(Path::new("/etc/passwd")));
        assert!(!is_safe_path(Path::new("../etc/passwd")));
    }
}