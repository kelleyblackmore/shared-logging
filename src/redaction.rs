//! Redaction helpers for PII, secrets, and tokens.

use regex::Regex;
use std::sync::LazyLock;

/// Redaction marker for sensitive data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Redaction {
    /// Redact completely (show only [REDACTED])
    Full,
    /// Show partial data (e.g., last 4 digits)
    Partial,
    /// Hash the value
    Hash,
}

impl Redaction {
    /// Apply redaction to a value.
    pub fn apply(&self, value: &str) -> String {
        match self {
            Redaction::Full => "[REDACTED]".to_string(),
            Redaction::Partial => {
                let char_count = value.chars().count();
                if char_count <= 4 {
                    "[REDACTED]".to_string()
                } else {
                    let tail: String = value.chars().skip(char_count - 4).collect();
                    format!("****{}", tail)
                }
            }
            Redaction::Hash => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                format!("hash:{}", hasher.finish())
            }
        }
    }
}

/// Redactor for detecting and redacting sensitive data.
pub struct Redactor {
    /// Patterns for detecting PII
    pii_patterns: Vec<Regex>,
    /// Patterns for detecting secrets/tokens
    secret_patterns: Vec<Regex>,
}

impl Redactor {
    /// Create a new redactor with default patterns.
    pub fn new() -> Self {
        Self {
            pii_patterns: Self::default_pii_patterns(),
            secret_patterns: Self::default_secret_patterns(),
        }
    }

    /// Create a redactor with custom patterns.
    pub fn with_patterns(
        pii_patterns: Vec<Regex>,
        secret_patterns: Vec<Regex>,
    ) -> Self {
        Self {
            pii_patterns,
            secret_patterns,
        }
    }

    /// Default PII detection patterns.
    fn default_pii_patterns() -> Vec<Regex> {
        vec![
            // Email addresses
            Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(),
            // Credit card numbers (basic pattern)
            Regex::new(r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b").unwrap(),
            // SSN (US)
            Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
            // Phone numbers (US format)
            Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap(),
        ]
    }

    /// Default secret/token detection patterns.
    fn default_secret_patterns() -> Vec<Regex> {
        vec![
            // API keys (common patterns)
            Regex::new(r"\b[A-Za-z0-9]{32,}\b").unwrap(),
            // Bearer tokens
            Regex::new(r"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*").unwrap(),
            // JWT tokens
            Regex::new(r"\beyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\b").unwrap(),
            // AWS access keys
            Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(),
            // Private keys (basic detection)
            Regex::new(r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----").unwrap(),
        ]
    }

    /// Check if a value contains PII.
    pub fn contains_pii(&self, value: &str) -> bool {
        self.pii_patterns.iter().any(|pattern| pattern.is_match(value))
    }

    /// Check if a value contains secrets/tokens.
    pub fn contains_secret(&self, value: &str) -> bool {
        self.secret_patterns.iter().any(|pattern| pattern.is_match(value))
    }

    /// Redact sensitive data in a string.
    pub fn redact(&self, value: &str, redaction: Redaction) -> String {
        let mut result = value.to_string();

        // Redact secrets first (most sensitive)
        for pattern in &self.secret_patterns {
            result = pattern
                .replace_all(&result, |caps: &regex::Captures<'_>| {
                    redaction.apply(&caps[0])
                })
                .to_string();
        }

        // Redact PII
        for pattern in &self.pii_patterns {
            result = pattern
                .replace_all(&result, |caps: &regex::Captures<'_>| {
                    redaction.apply(&caps[0])
                })
                .to_string();
        }

        result
    }

    /// Redact a field value based on its name.
    pub fn redact_field(&self, field_name: &str, value: &str) -> String {
        // Check field name for common sensitive field names
        let lower_name = field_name.to_lowercase();
        if lower_name.contains("password")
            || lower_name.contains("secret")
            || lower_name.contains("token")
            || lower_name.contains("key")
            || lower_name.contains("api_key")
            || lower_name.contains("access_token")
            || lower_name.contains("refresh_token")
        {
            return Redaction::Full.apply(value);
        }

        // Check if value contains secrets
        if self.contains_secret(value) {
            return Redaction::Full.apply(value);
        }

        // Check if value contains PII
        if self.contains_pii(value) {
            return Redaction::Partial.apply(value);
        }

        value.to_string()
    }
}

impl Default for Redactor {
    fn default() -> Self {
        Self::new()
    }
}

/// Global redactor instance.
static GLOBAL_REDACTOR: LazyLock<Redactor> = LazyLock::new(Redactor::new);

/// Redact a value using the global redactor.
pub fn redact(value: &str, redaction: Redaction) -> String {
    GLOBAL_REDACTOR.redact(value, redaction)
}

/// Redact a field value using the global redactor.
pub fn redact_field(field_name: &str, value: &str) -> String {
    GLOBAL_REDACTOR.redact_field(field_name, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_full() {
        assert_eq!(Redaction::Full.apply("secret123"), "[REDACTED]");
    }

    #[test]
    fn test_redaction_partial() {
        let result = Redaction::Partial.apply("1234567890");
        assert!(result.ends_with("7890"));
        assert!(result.starts_with("****"));
    }

    #[test]
    fn test_redaction_partial_multibyte() {
        // Must not panic when the last 4 bytes split a multi-byte character
        assert_eq!(Redaction::Partial.apply("пароль-ключ"), "****ключ");
        assert_eq!(Redaction::Partial.apply("ключ"), "[REDACTED]");
    }

    #[test]
    fn test_redactor_email() {
        let redactor = Redactor::new();
        assert!(redactor.contains_pii("test@example.com"));
        let redacted = redactor.redact("Contact: test@example.com", Redaction::Partial);
        assert!(redacted.contains("****"));
    }

    #[test]
    fn test_redactor_secret() {
        let redactor = Redactor::new();
        assert!(redactor.contains_secret("Bearer abc123token456"));
        let redacted = redactor.redact("Authorization: Bearer abc123token456", Redaction::Full);
        assert_eq!(redacted, "Authorization: [REDACTED]");
    }

    #[test]
    fn test_redact_field() {
        let redactor = Redactor::new();
        assert_eq!(
            redactor.redact_field("password", "secret123"),
            "[REDACTED]"
        );
        assert_eq!(
            redactor.redact_field("email", "test@example.com"),
            "****.com"
        );
    }
}

