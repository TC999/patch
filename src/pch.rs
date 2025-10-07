use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

/// Patch diff类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffType {
    NoDiff,
    ContextDiff,
    NormalDiff,
    EdDiff,
    NewContextDiff,
    UniDiff,
    GitBinaryDiff,
}

/// Patch头部信息
#[derive(Debug, Clone)]
pub struct PatchHeader {
    pub old_file: Option<String>,
    pub new_file: Option<String>,
    pub index_file: Option<String>,
    pub old_time: Option<String>,
    pub new_time: Option<String>,
    pub old_mode: Option<u32>,
    pub new_mode: Option<u32>,
    pub sha1_old: Option<String>,
    pub sha1_new: Option<String>,
}

/// 一个hunk块
#[derive(Debug, Clone)]
pub struct PatchHunk {
    pub orig_start: usize,
    pub orig_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<HunkLine>,
    pub func: Option<String>,
}

/// hunk中的一行
#[derive(Debug, Clone)]
pub struct HunkLine {
    pub kind: LineKind,
    pub content: String,
}

/// 行类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Context,  // ' '
    Add,      // '+'
    Remove,   // '-'
}

/// Patch主结构
#[derive(Debug, Clone)]
pub struct Patch {
    pub diff_type: DiffType,
    pub header: PatchHeader,
    pub hunks: Vec<PatchHunk>,
}

/// 解析patch文件
impl Patch {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("打开patch文件失败: {}", e))?;
        let mut reader = BufReader::new(file);

        let mut hunks = Vec::new();
        let mut header = PatchHeader {
            old_file: None,
            new_file: None,
            index_file: None,
            old_time: None,
            new_time: None,
            old_mode: None,
            new_mode: None,
            sha1_old: None,
            sha1_new: None,
        };

        let mut diff_type = DiffType::NoDiff;

        let mut line = String::new();
        while reader.read_line(&mut line).unwrap_or(0) > 0 {
            if line.starts_with("--- ") {
                header.old_file = Some(extract_filename(&line));
            } else if line.starts_with("+++ ") {
                header.new_file = Some(extract_filename(&line));
            } else if line.starts_with("@@ ") {
                diff_type = DiffType::UniDiff;
                let hunk = parse_unified_hunk(&mut reader, &line)?;
                hunks.push(hunk);
            }
            line.clear();
        }

        Ok(Patch {
            diff_type,
            header,
            hunks,
        })
    }
}

/// 提取文件名
fn extract_filename(line: &str) -> String {
    // "--- oldfile\t2025-10-01 ..." 取第一个空格后到第一个tab或换行
    let s = line.trim_start_matches(|c| c == '-' || c == '+').trim();
    s.split('\t').next().unwrap_or("").split_whitespace().next().unwrap_or("").to_string()
}

/// 解析unified diff的hunk块
fn parse_unified_hunk<R: BufRead>(reader: &mut R, first_line: &str) -> Result<PatchHunk, String> {
    // 头形如: "@@ -1,5 +1,6 @@ 函数名"
    let mut orig_start = 0;
    let mut orig_count = 0;
    let mut new_start = 0;
    let mut new_count = 0;
    let mut func_name = None;

    let mut parts = first_line.split("@@").nth(1).unwrap_or("").trim().split(' ');
    if let Some(ranges) = parts.next() {
        let mut rng_parts = ranges.split('+');
        if let Some(orig) = rng_parts.next() {
            let o = orig.trim_start_matches('-');
            let mut n = o.split(',');
            orig_start = n.next().unwrap_or("1").parse().unwrap_or(1);
            orig_count = n.next().unwrap_or("1").parse().unwrap_or(1);
        }
        if let Some(new_) = rng_parts.next() {
            let n = new_.split(',');
            let mut n_iter = n;
            new_start = n_iter.next().unwrap_or("1").parse().unwrap_or(1);
            new_count = n_iter.next().unwrap_or("1").parse().unwrap_or(1);
        }
    }
    func_name = parts.next().map(|s| s.trim().to_string());

    let mut lines = Vec::new();
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap_or(0) > 0 {
        let c = buf.chars().next().unwrap_or(' ');
        match c {
            '+' => lines.push(HunkLine { kind: LineKind::Add, content: buf[1..].to_string() }),
            '-' => lines.push(HunkLine { kind: LineKind::Remove, content: buf[1..].to_string() }),
            ' ' => lines.push(HunkLine { kind: LineKind::Context, content: buf[1..].to_string() }),
            '@' => break, // 下一个hunk块
            _ => {} // 忽略
        }
        buf.clear();
    }

    Ok(PatchHunk {
        orig_start,
        orig_count,
        new_start,
        new_count,
        lines,
        func: func_name,
    })
}