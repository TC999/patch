use std::fs::File;
use std::sync::atomic::{AtomicBool, AtomicI32};

pub type Idx = usize;

#[derive(Debug, Clone)]
pub struct OutFile {
    pub name: String,
    pub exists: Option<String>,
    pub alloc: Option<String>,
    pub temporary: bool,
}

pub static DEBUG: AtomicBool = AtomicBool::new(true);

pub static FORCE: AtomicBool = AtomicBool::new(false);
pub static BATCH: AtomicBool = AtomicBool::new(false);

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

pub static DIFF_TYPE: AtomicI32 = AtomicI32::new(DiffType::NoDiff as i32);

pub static DRY_RUN: AtomicBool = AtomicBool::new(false);

pub enum Verbosity {
    Default,
    Silent,
    Verbose,
}

pub struct OutState {
    pub file: File,
    pub after_newline: bool,
    pub zero_output: bool,
}