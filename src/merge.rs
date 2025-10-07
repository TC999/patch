use crate::inp::{InputFile, ILine};
use crate::pch::{Patch, PatchHunk, LineKind};
use crate::common::OutState;

/// 合并结果枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeResult {
    Clean,        // 完全干净合并
    Conflict,     // 有冲突
    AlreadyApplied, // 已经应用
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
                }
                input_idx += 1;
            }
            LineKind::Remove => {
                // 输入文件必须有此行，否则冲突
                if let Some(iline) = input.ifetch(input_idx) {
                    if iline.ptr != hunk_line.content.trim_end_matches('\n') {
                        conflict = true;
                    }
                }
                input_idx += 1;
            }
            LineKind::Add => {
                // 直接插入新行到输出
                // outstate.write_line(hunk_line.content.as_str()); // 假设有此方法
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

    for hunk in &patch.hunks {
        // 这里只是简单定位，实际应调用最佳匹配算法
        let where_to_apply = hunk.orig_start;
        let res = merge_hunk(input, hunk, outstate, where_to_apply);
        results.push(res);
    }

    results
}

// 假设 OutState 有写入接口
impl OutState {
    pub fn write_line(&mut self, line: &str) {
        use std::io::Write;
        writeln!(self.file, "{}", line).unwrap();
        self.zero_output = false;
        self.after_newline = line.ends_with('\n');
    }
}