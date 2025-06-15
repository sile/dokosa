use std::path::Path;

#[derive(Debug, Default)]
pub struct GlobPathFilter {
    pub include_files: Vec<GlobPathPattern>,
    pub exclude_files: Vec<GlobPathPattern>,
}

impl GlobPathFilter {
    pub fn should_include<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();

        // Check if path matches any exclude pattern
        if self
            .exclude_files
            .iter()
            .any(|pattern| pattern.matches(path))
        {
            return false;
        }

        // If no include patterns are specified, include all (that aren't excluded)
        if self.include_files.is_empty() {
            return true;
        }

        // Check if path matches at least one include pattern
        self.include_files
            .iter()
            .any(|pattern| pattern.matches(path))
    }
}

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

impl std::fmt::Display for GlobPathPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.matches_bos {
            write!(f, "*")?;
        }
        for (i, token) in self.tokens.iter().enumerate() {
            if i > 0 {
                write!(f, "*{token}")?;
            } else {
                write!(f, "{token}")?;
            }
        }
        if !self.matches_eos {
            write!(f, "*")?;
        }

        Ok(())
    }
}

impl std::str::FromStr for GlobPathPattern {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl nojson::DisplayJson for GlobPathPattern {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.string(self)
    }
}

impl<'text> nojson::FromRawJsonValue<'text> for GlobPathPattern {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        value.to_unquoted_string_str().map(|s| Self::new(&*s))
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
