use std::env;
use std::process;

mod common;
mod inp;
mod pch;
mod util;
mod bestmatch;
mod merge;
mod safe;

fn main() {
    // 显示版本信息
    println!("GNU patch (Rust重写版) v0.1.0");
    println!("版权所有 (c) 2025 Free Software Foundation, Inc.");
    println!("用法: patch [选项] 原文件 补丁文件\n");

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("参数错误，需指定原文件和补丁文件");
        process::exit(1);
    }

    let orig_file = &args[1];
    let patch_file = &args[2];

    // 1. 读取输入文件
    let input = match inp::InputFile::from_file(orig_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("读取输入文件失败: {}", e);
            process::exit(2);
        }
    };

    // 2. 解析补丁文件
    let patch = match pch::Patch::from_file(patch_file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("解析补丁失败: {}", e);
            process::exit(3);
        }
    };

    // 3. 打开输出文件
    let output_path = format!("{}.patched", orig_file);
    let output_file = match std::fs::File::create(&output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("创建输出文件失败: {}", e);
            process::exit(4);
        }
    };
    let mut outstate = common::OutState {
        file: output_file,
        after_newline: true,
        zero_output: true,
    };

    // 4. 应用补丁
    let results = merge::merge_patch(&input, &patch, &mut outstate);

    // 5. 显示结果
    for (i, result) in results.iter().enumerate() {
        match result {
            merge::MergeResult::Clean => {
                println!("Hunk #{} 合并成功", i + 1);
            }
            merge::MergeResult::Conflict => {
                println!("Hunk #{} 存在冲突，需人工处理", i + 1);
            }
            merge::MergeResult::AlreadyApplied => {
                println!("Hunk #{} 已经应用过", i + 1);
            }
        }
    }

    println!("输出已保存到: {}", output_path);
}