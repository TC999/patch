use std::fs::File;
use std::io::{BufRead, BufReader};
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
        let reader = BufReader::new(file);

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

        let lines: Vec<String> = reader.lines()
            .filter_map(|l| l.ok())
            .collect();
        
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            if line.starts_with("*** ") {
                // Context diff format
                header.old_file = Some(extract_filename(line));
                diff_type = DiffType::ContextDiff;
            } else if line.starts_with("--- ") && diff_type == DiffType::ContextDiff {
                // Second header line in context diff
                header.new_file = Some(extract_filename(line));
            } else if line.starts_with("--- ") {
                // Unified diff format
                header.old_file = Some(extract_filename(line));
            } else if line.starts_with("+++ ") {
                header.new_file = Some(extract_filename(line));
            } else if line.starts_with("***************") {
                // Context diff hunk separator
                diff_type = DiffType::ContextDiff;
                if i + 1 < lines.len() {
                    let (hunk, lines_consumed) = parse_context_hunk_from_vec(&lines, i + 1)?;
                    hunks.push(hunk);
                    i += lines_consumed;
                    continue;
                }
            } else if line.starts_with("@@ ") {
                diff_type = DiffType::UniDiff;
                let (hunk, lines_consumed) = parse_unified_hunk_from_vec(&lines, i)?;
                hunks.push(hunk);
                i += lines_consumed;
                continue;
            } else if line.contains('c') || line.contains('a') || line.contains('d') {
                // Normal diff format (e.g., "3c3", "2,4d1", "1a2,3")
                // Check if it looks like a normal diff command
                let parts: Vec<&str> = line.split(|c: char| c == 'a' || c == 'c' || c == 'd').collect();
                if parts.len() == 2 && parts[0].chars().all(|c| c.is_numeric() || c == ',') {
                    diff_type = DiffType::NormalDiff;
                    let (hunk, lines_consumed) = parse_normal_hunk_from_vec(&lines, i)?;
                    hunks.push(hunk);
                    i += lines_consumed;
                    continue;
                }
            }
            i += 1;
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

/// 从行向量解析unified diff的hunk块，返回(hunk, consumed_lines)
fn parse_unified_hunk_from_vec(lines: &[String], start_idx: usize) -> Result<(PatchHunk, usize), String> {
    if start_idx >= lines.len() {
        return Err("Invalid hunk start index".to_string());
    }
    
    let first_line = &lines[start_idx];
    
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

    let mut hunk_lines = Vec::new();
    let mut i = start_idx + 1;
    
    while i < lines.len() {
        let line = &lines[i];
        
        // 如果遇到新的hunk或者文件头，停止当前hunk的解析
        if line.starts_with("@@") || line.starts_with("---") || line.starts_with("+++") {
            break;
        }
        
        let c = line.chars().next().unwrap_or(' ');
        match c {
            '+' => hunk_lines.push(HunkLine { kind: LineKind::Add, content: line[1..].to_string() }),
            '-' => hunk_lines.push(HunkLine { kind: LineKind::Remove, content: line[1..].to_string() }),
            ' ' => hunk_lines.push(HunkLine { kind: LineKind::Context, content: line[1..].to_string() }),
            _ => {} // 忽略其他行（如注释等）
        }
        i += 1;
    }
    
    let consumed = i - start_idx;

    Ok((PatchHunk {
        orig_start,
        orig_count,
        new_start,
        new_count,
        lines: hunk_lines,
        func: func_name,
    }, consumed))
}

/// 从行向量解析context diff的hunk块，返回(hunk, consumed_lines)
fn parse_context_hunk_from_vec(lines: &[String], start_idx: usize) -> Result<(PatchHunk, usize), String> {
    if start_idx >= lines.len() {
        return Err("Invalid hunk start index".to_string());
    }
    
    // Context diff 格式:
    // *** 1,5 ****  <- 原始范围
    //   context
    // ! changed
    // - removed
    // --- 1,5 ----  <- 新范围  
    //   context
    // ! changed
    // + added
    
    let first_line = &lines[start_idx];
    let mut orig_start = 0;
    let mut orig_count = 0;
    
    // 解析 "*** 1,5 ****"
    if first_line.starts_with("*** ") {
        let range_str = first_line.trim_start_matches("*** ").trim_end_matches(" ****");
        let mut parts = range_str.split(',');
        orig_start = parts.next().unwrap_or("1").parse().unwrap_or(1);
        orig_count = parts.next().unwrap_or("1").parse().unwrap_or(1);
    }
    
    // 读取原始部分的行
    let mut orig_lines = Vec::new();
    let mut i = start_idx + 1;
    while i < lines.len() {
        let line = &lines[i];
        if line.starts_with("---") {
            break;
        }
        
        let c = line.chars().next().unwrap_or(' ');
        match c {
            ' ' => orig_lines.push((LineKind::Context, line[2..].to_string())),
            '!' => orig_lines.push((LineKind::Remove, line[2..].to_string())),
            '-' => orig_lines.push((LineKind::Remove, line[2..].to_string())),
            '+' => {}, // + 只出现在新部分
            _ => {}
        }
        i += 1;
    }
    
    // 解析 "--- 1,5 ----"
    let mut new_start = 0;
    let mut new_count = 0;
    if i < lines.len() && lines[i].starts_with("---") {
        let range_str = lines[i].trim_start_matches("--- ").trim_end_matches(" ----");
        let mut parts = range_str.split(',');
        new_start = parts.next().unwrap_or("1").parse().unwrap_or(1);
        new_count = parts.next().unwrap_or("1").parse().unwrap_or(1);
        i += 1;
    }
    
    // 读取新部分的行
    let mut new_lines = Vec::new();
    while i < lines.len() {
        let line = &lines[i];
        if line.starts_with("***") || line.is_empty() {
            break;
        }
        
        let c = line.chars().next().unwrap_or(' ');
        match c {
            ' ' => new_lines.push((LineKind::Context, line[2..].to_string())),
            '!' => new_lines.push((LineKind::Add, line[2..].to_string())),
            '+' => new_lines.push((LineKind::Add, line[2..].to_string())),
            '-' => {}, // - 只出现在原始部分
            _ => {}
        }
        i += 1;
    }
    
    // 合并原始和新的行到统一格式
    let mut hunk_lines = Vec::new();
    let mut orig_idx = 0;
    let mut new_idx = 0;
    
    while orig_idx < orig_lines.len() || new_idx < new_lines.len() {
        if orig_idx < orig_lines.len() && new_idx < new_lines.len() {
            let (orig_kind, orig_content) = &orig_lines[orig_idx];
            let (new_kind, new_content) = &new_lines[new_idx];
            
            if *orig_kind == LineKind::Context && *new_kind == LineKind::Context {
                // 两边都是上下文，应该相同
                hunk_lines.push(HunkLine { kind: LineKind::Context, content: orig_content.clone() });
                orig_idx += 1;
                new_idx += 1;
            } else if *orig_kind == LineKind::Remove {
                hunk_lines.push(HunkLine { kind: LineKind::Remove, content: orig_content.clone() });
                orig_idx += 1;
            } else if *new_kind == LineKind::Add {
                hunk_lines.push(HunkLine { kind: LineKind::Add, content: new_content.clone() });
                new_idx += 1;
            } else {
                // 默认情况
                orig_idx += 1;
                new_idx += 1;
            }
        } else if orig_idx < orig_lines.len() {
            let (kind, content) = &orig_lines[orig_idx];
            hunk_lines.push(HunkLine { kind: *kind, content: content.clone() });
            orig_idx += 1;
        } else if new_idx < new_lines.len() {
            let (kind, content) = &new_lines[new_idx];
            hunk_lines.push(HunkLine { kind: *kind, content: content.clone() });
            new_idx += 1;
        }
    }
    
    let consumed = i - start_idx + 1;

    Ok((PatchHunk {
        orig_start,
        orig_count,
        new_start,
        new_count,
        lines: hunk_lines,
        func: None,
    }, consumed))
}

/// 从行向量解析normal diff的hunk块，返回(hunk, consumed_lines)
fn parse_normal_hunk_from_vec(lines: &[String], start_idx: usize) -> Result<(PatchHunk, usize), String> {
    if start_idx >= lines.len() {
        return Err("Invalid hunk start index".to_string());
    }
    
    let first_line = &lines[start_idx];
    
    // Normal diff 格式:
    // 3c3         <- 改变第3行到第3行
    // < line 3    <- 原始行
    // ---
    // > new line  <- 新行
    //
    // 或:
    // 2,4d1       <- 删除第2-4行
    // < line 2
    // < line 3  
    // < line 4
    //
    // 或:
    // 1a2,3       <- 在第1行后添加
    // > line 2
    // > line 3
    
    let mut orig_start = 0;
    let mut orig_count = 0;
    let mut new_start = 0;
    let mut new_count = 0;
    let mut op = ' ';
    
    // 解析命令行 (e.g., "3c3", "2,4d1", "1a2,3")
    if let Some(c_pos) = first_line.find('c') {
        op = 'c';
        let (left, right) = first_line.split_at(c_pos);
        let right = &right[1..]; // skip 'c'
        
        parse_range(left, &mut orig_start, &mut orig_count);
        parse_range(right, &mut new_start, &mut new_count);
    } else if let Some(d_pos) = first_line.find('d') {
        op = 'd';
        let (left, right) = first_line.split_at(d_pos);
        let right = &right[1..]; // skip 'd'
        
        parse_range(left, &mut orig_start, &mut orig_count);
        parse_range(right, &mut new_start, &mut new_count);
    } else if let Some(a_pos) = first_line.find('a') {
        op = 'a';
        let (left, right) = first_line.split_at(a_pos);
        let right = &right[1..]; // skip 'a'
        
        parse_range(left, &mut orig_start, &mut orig_count);
        parse_range(right, &mut new_start, &mut new_count);
    }
    
    let mut hunk_lines = Vec::new();
    let mut i = start_idx + 1;
    
    // 读取原始行 (以 '<' 开头)
    while i < lines.len() && lines[i].starts_with("< ") {
        let content = lines[i][2..].to_string();
        hunk_lines.push(HunkLine { kind: LineKind::Remove, content });
        i += 1;
    }
    
    // 跳过 "---" 分隔符
    if i < lines.len() && lines[i].starts_with("---") {
        i += 1;
    }
    
    // 读取新行 (以 '>' 开头)
    while i < lines.len() && lines[i].starts_with("> ") {
        let content = lines[i][2..].to_string();
        hunk_lines.push(HunkLine { kind: LineKind::Add, content });
        i += 1;
    }
    
    let consumed = i - start_idx;

    Ok((PatchHunk {
        orig_start,
        orig_count,
        new_start,
        new_count,
        lines: hunk_lines,
        func: None,
    }, consumed))
}

/// 解析范围字符串 (e.g., "3" -> (3, 1), "2,4" -> (2, 3))
fn parse_range(range: &str, start: &mut usize, count: &mut usize) {
    let range = range.trim();
    if let Some(comma_pos) = range.find(',') {
        let (s, e) = range.split_at(comma_pos);
        let e = &e[1..]; // skip ','
        let start_val: usize = s.parse().unwrap_or(1);
        let end_val: usize = e.parse().unwrap_or(start_val);
        *start = start_val;
        *count = if end_val >= start_val { end_val - start_val + 1 } else { 1 };
    } else {
        *start = range.parse().unwrap_or(1);
        *count = 1;
    }
}