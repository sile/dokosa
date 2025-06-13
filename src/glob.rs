use std::path::Path;

#[derive(Debug, Clone)]
pub struct GlobPathPattern {
    pattern: Vec<char>,
}

impl GlobPathPattern {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.chars().collect(),
        }
    }

    pub fn matches<P: AsRef<Path>>(&self, path: P) -> bool {
        let Some(s) = path.as_ref().as_os_str().to_str() else {
            return false;
        };
        todo!()
    }
}

pub fn glob_matches(s: &str, pattern: &str) -> bool {
    // TODO: Use Peekable<Chars> instead
    let s_chars: Vec<char> = s.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    fn matches_helper(s: &[char], pattern: &[char]) -> bool {
        match (s.is_empty(), pattern.is_empty()) {
            // Both empty - match
            (true, true) => true,
            // String empty but pattern has non-* characters - no match
            (true, false) => pattern.iter().all(|&c| c == '*'),
            // Pattern empty but string has characters - no match
            (false, true) => false,
            // Both have characters
            (false, false) => {
                match pattern[0] {
                    '*' => {
                        // Try matching * with empty string (skip the *)
                        matches_helper(s, &pattern[1..]) ||
                        // Try matching * with one or more characters
                        matches_helper(&s[1..], pattern)
                    }
                    c => {
                        // Regular character must match exactly
                        s[0] == c && matches_helper(&s[1..], &pattern[1..])
                    }
                }
            }
        }
    }

    matches_helper(&s_chars, &pattern_chars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matches() {
        // Exact matches
        assert!(glob_matches("hello", "hello"));
        assert!(!glob_matches("hello", "world"));

        // Multi-character wildcard
        assert!(glob_matches("hello", "*"));
        assert!(glob_matches("hello", "h*"));
        assert!(glob_matches("hello", "*o"));
        assert!(glob_matches("hello", "h*o"));
        assert!(glob_matches("hello", "*ell*"));
        assert!(glob_matches("hello", "he*lo"));
        assert!(!glob_matches("hello", "*x*"));
        assert!(!glob_matches("hello", "h*x"));

        // Multiple stars
        assert!(glob_matches("hello", "h**o"));
        assert!(glob_matches("hello", "*h*e*l*l*o*"));
        assert!(glob_matches("abc", "***"));

        // Edge cases
        assert!(glob_matches("", ""));
        assert!(glob_matches("", "*"));
        assert!(glob_matches("", "***"));
        assert!(!glob_matches("a", ""));
        assert!(!glob_matches("hello", "hi*"));
    }
}
