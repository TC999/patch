use crate::inp::InputFile;
use crate::pch::{Patch, PatchHunk, LineKind};
use crate::common::OutState;

/// 合并结果枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeResult {
    Clean,        // 完全干净合并
    Conflict,     // 有冲突
    AlreadyApplied, // 已经应用
}

/// 查找 hunk 的最佳应用位置
fn find_best_match(input: &InputFile, hunk: &PatchHunk, _fuzz: usize) -> Option<usize> {
    // 提取 hunk 中的原始行（context + remove）
    let hunk_orig_lines: Vec<String> = hunk.lines.iter()
        .filter(|l| l.kind == LineKind::Context || l.kind == LineKind::Remove)
        .map(|l| l.content.trim_end_matches('\n').to_string())
        .collect();
    
    if hunk_orig_lines.is_empty() {
        return Some(hunk.orig_start);
    }
    
    // 首先尝试精确匹配预期位置
    let expected_start = hunk.orig_start;
    let mut matches = true;
    
    for (i, hunk_line) in hunk_orig_lines.iter().enumerate() {
        if let Some(input_line) = input.ifetch(expected_start + i) {
            if input_line.ptr != hunk_line {
                matches = false;
                break;
            }
        } else {
            matches = false;
            break;
        }
    }
    
    if matches {
        return Some(expected_start);
    }
    
    // 如果精确匹配失败，尝试在附近搜索（简单的线性搜索）
    let search_range = 100; // 在前后100行内搜索
    let min_start = if expected_start > search_range { expected_start - search_range } else { 1 };
    let max_start = (expected_start + search_range).min(input.num_lines());
    
    for try_start in min_start..=max_start {
        let mut found = true;
        for (i, hunk_line) in hunk_orig_lines.iter().enumerate() {
            if let Some(input_line) = input.ifetch(try_start + i) {
                if input_line.ptr != hunk_line {
                    found = false;
                    break;
                }
            } else {
                found = false;
                break;
            }
        }
        
        if found {
            return Some(try_start);
        }
    }
    
    // 如果找不到完全匹配，返回原始位置
    Some(expected_start)
}

/// 合并单个 hunk 到输入文件
pub fn merge_hunk(
    input: &InputFile,
    patch_hunk: &PatchHunk,
    outstate: &mut OutState,
    start_at: usize, // 合并起始行（1-based）
) -> MergeResult {
    let mut input_idx = start_at;
    let mut hunk_idx = 0;
    let mut conflict = false;

    // 简单的三路合并主循环
    while hunk_idx < patch_hunk.lines.len() {
        let hunk_line = &patch_hunk.lines[hunk_idx];

        match hunk_line.kind {
            LineKind::Context => {
                // 必须和输入文件一致，否则可能冲突
                if let Some(iline) = input.ifetch(input_idx) {
                    if iline.ptr != hunk_line.content.trim_end_matches('\n') {
                        conflict = true;
                        // 这里可以收集冲突信息
                    }
                    // 输出上下文行
                    outstate.write_line(iline.ptr);
                }
                input_idx += 1;
            }
            LineKind::Remove => {
                // 输入文件必须有此行，否则冲突
                if let Some(iline) = input.ifetch(input_idx) {
                    if iline.ptr != hunk_line.content.trim_end_matches('\n') {
                        conflict = true;
                    }
                    // Remove 行不输出到结果文件
                }
                input_idx += 1;
            }
            LineKind::Add => {
                // 直接插入新行到输出
                outstate.write_line(hunk_line.content.trim_end_matches('\n'));
            }
        }
        hunk_idx += 1;
    }

    if conflict {
        MergeResult::Conflict
    } else {
        MergeResult::Clean
    }
}

/// 合并整个 patch 到输入文件
pub fn merge_patch(
    input: &InputFile,
    patch: &Patch,
    outstate: &mut OutState,
) -> Vec<MergeResult> {
    let mut results = Vec::new();
    let mut current_line = 1; // 1-based index
    
    for hunk in &patch.hunks {
        // 使用最佳匹配算法定位 hunk 应用位置
        let fuzz = 2; // 允许2行的差异
        let where_to_apply = find_best_match(input, hunk, fuzz)
            .unwrap_or(hunk.orig_start);
        
        // 输出 hunk 之前的未修改行
        while current_line < where_to_apply {
            if let Some(iline) = input.ifetch(current_line) {
                outstate.write_line(iline.ptr);
            }
            current_line += 1;
        }
        
        let res = merge_hunk(input, hunk, outstate, where_to_apply);
        results.push(res);
        
        // 更新当前行位置：跳过 hunk 中处理的原始行数
        // 计算 hunk 中的原始行数（context + remove）
        let orig_lines = hunk.lines.iter().filter(|l| {
            l.kind == LineKind::Context || l.kind == LineKind::Remove
        }).count();
        current_line = where_to_apply + orig_lines;
    }
    
    // 输出最后一个 hunk 之后的所有剩余行
    while current_line <= input.num_lines() {
        if let Some(iline) = input.ifetch(current_line) {
            outstate.write_line(iline.ptr);
        }
        current_line += 1;
    }

    results
}

// 假设 OutState 有写入接口
impl OutState {
    pub fn write_line(&mut self, line: &str) {
        use std::io::Write;
        // 直接写入内容，不添加额外的换行
        write!(self.file, "{}\n", line).unwrap();
        self.zero_output = false;
        self.after_newline = true;
    }
}