/// 帮助文本，保持与 GNU patch 兼容
pub const HELP_TEXT: &str = r#"
用法: patch [选项]... [原文件 [补丁文件]]

输入选项:

  -p NUM  --strip=NUM         去除文件名前 NUM 个组件
  -F LINES  --fuzz=LINES      设置模糊匹配行数
  -l  --ignore-whitespace     忽略空白字符的变化

  -c  --context               按上下文差异格式解析补丁
  -e  --ed                    按 ed 脚本格式解析补丁
  -n  --normal                按普通差异格式解析补丁
  -u  --unified               按统一差异格式解析补丁

  -N  --forward               忽略反向或已应用的补丁
  -R  --reverse               交换原文件和新文件位置
  -i PATCHFILE  --input=PATCHFILE  指定补丁文件

输出选项:

  -o FILE  --output=FILE      输出到指定文件
  -r FILE  --reject-file=FILE 输出未能应用的补丁到 FILE

  -D NAME  --ifdef=NAME       用 ifdef/ifndef 方式输出冲突
  --merge                     使用冲突标记而不是 reject 文件输出
  -E  --remove-empty-files    删除应用后为空的文件

  -Z  --set-utc               用 UTC 时间设置文件时间戳
  -T  --set-time              用本地时间设置文件时间戳

  --quoting-style=WORD        文件名引用风格（literal, shell, c, escape）
                             默认从环境变量 QUOTING_STYLE 获取，未设置则为 shell

备份和版本控制选项:

  -b  --backup                备份原文件
  --backup-if-mismatch        仅在补丁不完全匹配时备份
  --no-backup-if-mismatch     仅在另外指定时备份不匹配

  -V STYLE  --version-control=STYLE  备份方式（simple, numbered, existing）
  -B PREFIX  --prefix=PREFIX         备份文件名前缀
  -Y PREFIX  --basename-prefix=PREFIX  备份文件基名前缀
  -z SUFFIX  --suffix=SUFFIX         备份文件名后缀

  -g NUM  --get=NUM           RCS/SCCS 文件获取模式（正值自动，负值询问）

其他选项:

  -t  --batch                 批处理，跳过疑问
  -f  --force                 强制，忽略疑问和反向
  -s  --quiet --silent        安静模式，出错才显示
  --verbose                   显示详细信息
  --dry-run                   仅显示将做什么，不实际修改文件
  --posix                     严格遵循 POSIX 标准

  -d DIR  --directory=DIR     首先切换到指定目录
  --reject-format=FORMAT      reject 文件格式（context/unified）
  --binary                    以二进制方式读写数据
  --read-only=BEHAVIOR        只读文件处理方式: ignore/warn/fail

  -v  --version               显示版本信息
  --help                      显示本帮助

Bug报告请发送至 <bug-gnu-patch@gnu.org>
"#;