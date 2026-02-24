use crate::error::HttpBindingError;

/// A single segment in a URI pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UriSegment {
    /// A literal path segment (e.g. `buckets`).
    Literal(String),
    /// A label path segment (e.g. `{bucket}`).
    Label(String),
    /// A greedy label that captures the rest of the path (e.g. `{key+}`).
    GreedyLabel(String),
}

/// A parsed Smithy URI pattern.
///
/// Supports patterns like `/buckets/{bucket}/objects/{key+}?versionId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriPattern {
    segments: Vec<UriSegment>,
    query_literals: Vec<(String, String)>,
}

impl UriPattern {
    /// Parse a Smithy URI pattern string.
    ///
    /// # Errors
    /// Returns [`HttpBindingError`] if the pattern is malformed.
    pub fn parse(pattern: &str) -> Result<Self, HttpBindingError> {
        let (path, query) = pattern.split_once('?').unwrap_or((pattern, ""));

        let segments = Self::parse_segments(path)?;
        let query_literals = Self::parse_query(query);

        Ok(Self {
            segments,
            query_literals,
        })
    }

    /// Get the path segments.
    #[inline]
    #[must_use]
    pub fn segments(&self) -> &[UriSegment] {
        &self.segments
    }

    /// Get the query literal key-value pairs.
    #[inline]
    #[must_use]
    pub fn query_literals(&self) -> &[(String, String)] {
        &self.query_literals
    }

    fn parse_segments(path: &str) -> Result<Vec<UriSegment>, HttpBindingError> {
        let mut segments = Vec::new();
        for part in path.split('/') {
            if part.is_empty() {
                continue;
            }
            if let Some(inner) = part.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                if let Some(label) = inner.strip_suffix('+') {
                    if label.is_empty() {
                        return Err(HttpBindingError::new("empty greedy label name"));
                    }
                    segments.push(UriSegment::GreedyLabel(label.to_string()));
                } else {
                    if inner.is_empty() {
                        return Err(HttpBindingError::new("empty label name"));
                    }
                    segments.push(UriSegment::Label(inner.to_string()));
                }
            } else {
                segments.push(UriSegment::Literal(part.to_string()));
            }
        }
        Ok(segments)
    }

    fn parse_query(query: &str) -> Vec<(String, String)> {
        if query.is_empty() {
            return Vec::new();
        }
        let mut pairs = Vec::new();
        for pair_str in query.split('&') {
            if let Some((key, value)) = pair_str.split_once('=') {
                pairs.push((key.to_string(), value.to_string()));
            } else if !pair_str.is_empty() {
                pairs.push((pair_str.to_string(), String::new()));
            }
        }
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_path() {
        let pattern = UriPattern::parse("/buckets").unwrap();
        assert_eq!(
            pattern.segments(),
            &[UriSegment::Literal("buckets".to_string())]
        );
        assert!(pattern.query_literals().is_empty());
    }

    #[test]
    fn parse_labels() {
        let pattern = UriPattern::parse("/buckets/{bucket}/objects/{key+}").unwrap();
        assert_eq!(
            pattern.segments(),
            &[
                UriSegment::Literal("buckets".to_string()),
                UriSegment::Label("bucket".to_string()),
                UriSegment::Literal("objects".to_string()),
                UriSegment::GreedyLabel("key".to_string()),
            ]
        );
    }

    #[test]
    fn parse_query_string() {
        let pattern = UriPattern::parse("/buckets/{bucket}?versionId=123&marker").unwrap();
        assert_eq!(
            pattern.segments(),
            &[
                UriSegment::Literal("buckets".to_string()),
                UriSegment::Label("bucket".to_string()),
            ]
        );
        assert_eq!(
            pattern.query_literals(),
            &[
                ("versionId".to_string(), "123".to_string()),
                ("marker".to_string(), String::new()),
            ]
        );
    }

    #[test]
    fn parse_root_path() {
        let pattern = UriPattern::parse("/").unwrap();
        assert!(pattern.segments().is_empty());
        assert!(pattern.query_literals().is_empty());
    }

    #[test]
    fn empty_label_name_is_error() {
        assert!(UriPattern::parse("/{}").is_err());
    }

    #[test]
    fn empty_greedy_label_name_is_error() {
        assert!(UriPattern::parse("/{+}").is_err());
    }

    #[test]
    fn parse_multiple_literal_segments() {
        let pattern = UriPattern::parse("/a/b/c").unwrap();
        assert_eq!(
            pattern.segments(),
            &[
                UriSegment::Literal("a".to_string()),
                UriSegment::Literal("b".to_string()),
                UriSegment::Literal("c".to_string()),
            ]
        );
    }

    #[test]
    fn parse_label_only() {
        let pattern = UriPattern::parse("/{id}").unwrap();
        assert_eq!(pattern.segments(), &[UriSegment::Label("id".to_string())]);
    }

    #[test]
    fn parse_mixed_labels_and_literals() {
        let pattern = UriPattern::parse("/a/{b}/c/{d+}").unwrap();
        assert_eq!(
            pattern.segments(),
            &[
                UriSegment::Literal("a".to_string()),
                UriSegment::Label("b".to_string()),
                UriSegment::Literal("c".to_string()),
                UriSegment::GreedyLabel("d".to_string()),
            ]
        );
    }

    #[test]
    fn parse_multiple_query_params() {
        let pattern = UriPattern::parse("/?a=1&b=2&c").unwrap();
        assert!(pattern.segments().is_empty());
        assert_eq!(
            pattern.query_literals(),
            &[
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
                ("c".to_string(), String::new()),
            ]
        );
    }

    #[test]
    fn parse_query_with_empty_value() {
        let pattern = UriPattern::parse("/items?key=").unwrap();
        assert_eq!(
            pattern.query_literals(),
            &[("key".to_string(), String::new())]
        );
    }

    #[test]
    fn parse_empty_string() {
        let pattern = UriPattern::parse("").unwrap();
        assert!(pattern.segments().is_empty());
        assert!(pattern.query_literals().is_empty());
    }

    #[test]
    fn parse_trailing_slash() {
        // Trailing slash produces an empty segment that gets filtered
        let pattern = UriPattern::parse("/items/").unwrap();
        assert_eq!(
            pattern.segments(),
            &[UriSegment::Literal("items".to_string())]
        );
    }

    #[test]
    fn parse_greedy_label_only() {
        let pattern = UriPattern::parse("/{path+}").unwrap();
        assert_eq!(
            pattern.segments(),
            &[UriSegment::GreedyLabel("path".to_string())]
        );
    }
}
