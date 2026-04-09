//! Port of Java `AgvPlcNamePrefixHelper.stripPrefixAndParseInt` for OpenTCS ↔ protocol name ids.

/// Mirrors `AgvPlcNamePrefixHelper.stripPrefixAndParseInt`: strip optional prefix, parse int, else
/// trailing digit run (same order as Java).
pub fn strip_prefix_and_parse_int(value: &str, prefix: &str) -> i32 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return 0;
    }
    let after_prefix = if !prefix.is_empty() && trimmed.starts_with(prefix) {
        &trimmed[prefix.len()..]
    } else {
        trimmed
    };
    if let Ok(n) = after_prefix.parse::<i32>() {
        return n;
    }
    if let Some(digits) = trailing_digits_at_end(after_prefix) {
        if let Ok(n) = digits.parse::<i32>() {
            return n;
        }
    }
    0
}

fn trailing_digits_at_end(s: &str) -> Option<&str> {
    let s = s.trim_end();
    let mut start_idx = None;
    for (i, c) in s.char_indices().rev() {
        if c.is_ascii_digit() {
            start_idx = Some(i);
        } else {
            break;
        }
    }
    let i = start_idx?;
    Some(&s[i..])
}

#[cfg(test)]
mod tests {
    use super::strip_prefix_and_parse_int;

    #[test]
    fn matches_java_point_examples() {
        assert_eq!(strip_prefix_and_parse_int("Point_3", "Point_"), 3);
        assert_eq!(strip_prefix_and_parse_int("3", "Point_"), 3);
    }

    #[test]
    fn matches_java_path_examples() {
        assert_eq!(strip_prefix_and_parse_int("Path_1", "Path_"), 1);
        assert_eq!(strip_prefix_and_parse_int("12", "Path_"), 12);
    }

    #[test]
    fn trailing_digits_fallback_like_java() {
        assert_eq!(strip_prefix_and_parse_int("Point_99", ""), 99);
    }
}
