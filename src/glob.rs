use std::path::Path;

#[derive(Debug, Clone)]
pub struct GlobPathPattern {
    tokens: Vec<String>,
    matches_bos: bool,
    matches_eos: bool,
}

impl GlobPathPattern {
    pub fn new(pattern: &str) -> Self {
        Self {
            matches_bos: !pattern.starts_with('*'),
            matches_eos: !pattern.ends_with('*'),
            tokens: pattern.split('*').map(|s| s.to_owned()).collect(),
        }
    }

    pub fn matches<P: AsRef<Path>>(&self, path: P) -> bool {
        let Some(mut s) = path.as_ref().as_os_str().to_str() else {
            return false;
        };

        let mut tokens = self.tokens.iter();
        if self.matches_bos {
            let Some(s0) = tokens.next().and_then(|t| s.strip_prefix(t)) else {
                return false;
            };
            s = s0;
        }

        for token in tokens {
            let Some(i) = s.find(token) else {
                return false;
            };
            s = &s[i + token.len()..];
        }

        if self.matches_eos { s.is_empty() } else { true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glob_matches(s: &str, pattern: &str) -> bool {
        GlobPathPattern::new(pattern).matches(s)
    }

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
