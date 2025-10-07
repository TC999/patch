/// Myers O(ND)差分算法的最佳匹配实现（精简版）
/// 用于在两个向量（如文本行）之间寻找最佳匹配
///
/// 参数说明：
/// - a: 原始序列（如输入文件的行）
/// - b: 目标序列（如patch中的行）
/// - equal: 比较a/b元素是否相等的函数
/// - min: 最少需要匹配的元素数
/// - max: 允许的最大变更（插入+删除）
/// 返回: 所需的最小变更数，如果无法在max之内完成则返回max+1
pub fn bestmatch<T, F>(
    a: &[T],
    b: &[T],
    equal: F,
    min: usize,
    max: usize,
) -> (usize, usize)
where
    F: Fn(&T, &T) -> bool,
{
    // Myers算法，返回(最小变更数, 匹配到的b前缀长度)
    let n = a.len();
    let m = b.len();
    let dmax = max;
    let mut v = vec![0isize; 2 * dmax + 3];
    let fmid = n as isize - m as isize;

    let offset = (dmax + 1) as isize - fmid;
    let fd = &mut v[offset as usize..];

    let mut ymax = -1isize;
    let mut c = 0;

    // 先处理完全匹配的前缀
    let mut xoff = 0;
    let mut yoff = 0;
    while xoff < n && yoff < m && equal(&a[xoff], &b[yoff]) {
        xoff += 1;
        yoff += 1;
    }
    let fmid_plus_2_min = fmid + 2 * min as isize;

    if xoff == n && (yoff as isize) >= min as isize && (xoff + yoff) as isize >= fmid_plus_2_min {
        return (0, yoff);
    }

    fd[fmid as usize] = xoff as isize;

    for c in 1..=max {
        let mut fmin = fmid - c as isize;
        let mut fmax = fmid + c as isize;

        // 尝试所有对角线
        for d in (fmin..=fmax).rev().step_by(2) {
            let k = d as usize;
            let mut x;
            if fd.get(k.wrapping_sub(1)).unwrap_or(&-1) < fd.get(k.wrapping_add(1)).unwrap_or(&-1) {
                x = *fd.get(k.wrapping_add(1)).unwrap_or(&-1);
            } else {
                x = fd.get(k.wrapping_sub(1)).unwrap_or(&-1) + 1;
            }
            let mut y = x - d;
            // 沿对角线尝试延长匹配
            while (x as usize) < n && (y as usize) < m && equal(&a[x as usize], &b[y as usize]) {
                x += 1;
                y += 1;
            }
            fd[k] = x;
            if x as usize == n && (y as usize) >= min && x + y - (c as isize) >= fmid_plus_2_min {
                if ymax < y {
                    ymax = y;
                }
                if y as usize == m {
                    return (c, ymax as usize);
                }
            }
        }
        if ymax != -1 {
            return (c, ymax as usize);
        }
    }
    (max + 1, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bestmatch_simple() {
        let a = vec!["a", "b", "c", "d"];
        let b = vec!["a", "x", "c", "d"];
        let (changes, matched) = bestmatch(&a, &b, |x, y| x == y, 2, 2);
        assert_eq!(changes, 1);
        assert_eq!(matched, 4);
    }

    #[test]
    fn test_bestmatch_exact_match() {
        let a = vec!["a", "b", "c"];
        let b = vec!["a", "b", "c"];
        let (changes, matched) = bestmatch(&a, &b, |x, y| x == y, 3, 3);
        assert_eq!(changes, 0);
        assert_eq!(matched, 3);
    }

    #[test]
    fn test_bestmatch_no_match() {
        let a = vec!["a", "b", "c"];
        let b = vec!["x", "y", "z"];
        let (changes, matched) = bestmatch(&a, &b, |x, y| x == y, 1, 2);
        assert_eq!(changes, 3);
        assert_eq!(matched, 0);
    }
}