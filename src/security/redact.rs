/// Redact common secret patterns before logging user-controlled strings.
pub fn redact_secrets(input: &str) -> String {
    let mut out = input.to_string();

    // Bearer tokens
    if let Some(idx) = out.find("Bearer ") {
        let rest = &out[idx + 7..];
        let token_len = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
        if token_len > 0 {
            out.replace_range(idx..idx + 7 + token_len, "Bearer ***");
        }
    }

    // sk- prefixed API keys (OpenAI-style)
    for prefix in ["sk-", "sk_live_", "sk_test_"] {
        if let Some(start) = out.find(prefix) {
            let tail = &out[start..];
            let len = tail
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .count();
            if len > prefix.len() + 4 {
                out.replace_range(start..start + len, &format!("{prefix}***"));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_bearer_token() {
        // Use obviously fake token material — not a real secret pattern.
        let s = "Authorization: Bearer FAKE_TOKEN_FOR_UNIT_TEST_ONLY";
        let redacted = redact_secrets(s);
        assert!(redacted.contains("Bearer ***"));
        assert!(!redacted.contains("FAKE_TOKEN_FOR_UNIT_TEST_ONLY"));
    }
}
