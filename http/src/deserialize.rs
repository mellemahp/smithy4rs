//! HTTP request and response deserialization.
//!
//! Routes HTTP message parts (URI labels, query params, headers, status code, body)
//! back to struct members based on their HTTP binding locations.

use std::borrow::Cow;

use smithy4rs_core::{
    schema::Schema,
    serde::{
        codec::Codec,
        de::{self, DeserializeWithSchema, Deserializer as _},
        never::Never,
    },
};

use crate::{
    binding::{MemberBinding, MemberBindings, OperationBinding},
    error::HttpBindingError,
    header::HeaderDeserializer,
    label::LabelDeserializer,
    query::QueryDeserializer,
    uri::{UriPattern, UriSegment},
};

/// Deserializer that reads an HTTP status code as an integer.
struct StatusCodeDeserializer(u16);

impl<'de> de::Deserializer<'de> for StatusCodeDeserializer {
    type Error = HttpBindingError;
    type StructReader = Never<Self::Error>;
    type ListReader = Never<Self::Error>;
    type MapReader = Never<Self::Error>;

    fn read_integer(self, _schema: &Schema) -> Result<i32, Self::Error> {
        Ok(i32::from(self.0))
    }
}

/// Deserializer that reads headers with a given prefix into a map.
struct PrefixHeadersDeserializer<'a> {
    headers: &'a http::HeaderMap,
    prefix: &'a str,
}

impl<'de, 'a> de::Deserializer<'de> for PrefixHeadersDeserializer<'a> {
    type Error = HttpBindingError;
    type StructReader = Never<Self::Error>;
    type ListReader = Never<Self::Error>;
    type MapReader = PrefixHeadersMapReader<'a>;

    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
        Ok(PrefixHeadersMapReader {
            iter: self.headers.iter(),
            prefix: self.prefix,
            current_value: None,
        })
    }
}

/// Lazy map reader for prefix headers.
///
/// Iterates the `HeaderMap` on demand instead of collecting matching entries
/// upfront, avoiding a `Vec` allocation and eager string cloning.
struct PrefixHeadersMapReader<'a> {
    iter: http::header::Iter<'a, http::header::HeaderValue>,
    prefix: &'a str,
    // The value for the current entry, set by `read_key` and consumed by `read_value`.
    current_value: Option<&'a http::header::HeaderValue>,
}

impl<'de, 'a> de::MapReader<'de> for PrefixHeadersMapReader<'a> {
    type Error = HttpBindingError;

    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        for (name, value) in self.iter.by_ref() {
            if let Some(suffix) = name.as_str().strip_prefix(self.prefix) {
                self.current_value = Some(value);
                return Ok(Some(suffix.to_string()));
            }
        }
        Ok(None)
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        let value = self
            .current_value
            .take()
            .ok_or_else(|| HttpBindingError::new("read_value called without read_key"))?;
        let value_str = value
            .to_str()
            .map_err(|_| HttpBindingError::new("invalid header value encoding"))?;
        V::deserialize_with_schema(schema, HeaderDeserializer::new(value_str))
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.current_value = None;
        Ok(())
    }

    fn size_hint(&self) -> Option<usize> {
        None
    }
}

/// Deserializes an `http::Response<&[u8]>` into a value by routing response
/// parts to their HTTP binding locations and delegating body content to a
/// [`Codec`].
pub struct HttpResponseDeserializer<'a, C: Codec> {
    codec: &'a C,
}

impl<'a, C: Codec> HttpResponseDeserializer<'a, C> {
    /// Create a new response deserializer backed by the given codec.
    pub fn new(codec: &'a C) -> Self {
        Self { codec }
    }

    /// Deserialize an HTTP response into a value using a pre-built operation binding.
    ///
    /// # Errors
    /// Returns [`HttpBindingError`] if deserialization fails.
    pub fn deserialize<'de, T>(
        &self,
        response: &'de http::Response<&'de [u8]>,
        output_schema: &Schema,
        op_binding: &OperationBinding<'_>,
    ) -> Result<T, HttpBindingError>
    where
        T: DeserializeWithSchema<'de>,
    {
        let deserializer = HttpDeserializer {
            codec: self.codec,
            body: response.body(),
            status_code: response.status().as_u16(),
            headers: response.headers(),
            bindings: &op_binding.response,
            output_schema,
        };

        T::deserialize_with_schema(output_schema, deserializer)
    }
}

/// Top-level deserializer that routes struct members from HTTP binding locations.
struct HttpDeserializer<'a, 'b, 'de, C: Codec> {
    codec: &'a C,
    body: &'de [u8],
    status_code: u16,
    headers: &'de http::HeaderMap,
    bindings: &'b MemberBindings<'b>,
    output_schema: &'b Schema,
}

impl<'a, 'b, 'de, C: Codec> de::Deserializer<'de> for HttpDeserializer<'a, 'b, 'de, C> {
    type Error = HttpBindingError;
    type StructReader = HttpStructReader<'a, 'b, 'de, C>;
    type ListReader = Never<Self::Error>;
    type MapReader = Never<Self::Error>;

    fn read_struct(self, _schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        let has_value =
            compute_response_has_value(self.bindings.as_slice(), self.headers, self.body);
        Ok(HttpStructReader {
            codec: self.codec,
            body: self.body,
            status_code: self.status_code,
            headers: self.headers,
            remaining: self.bindings.as_slice(),
            current_binding: None,
            has_value,
            total_members: self.bindings.len(),
            output_schema: self.output_schema,
            body_reader: None,
        })
    }
}

/// Pre-compute which response bindings have values present.
fn compute_response_has_value(
    bindings: &[MemberBinding<'_>],
    headers: &http::HeaderMap,
    body: &[u8],
) -> Vec<bool> {
    bindings
        .iter()
        .map(|binding| match binding {
            MemberBinding::StatusCode { .. } => true,
            MemberBinding::Header { name, .. } => headers.contains_key(name),
            MemberBinding::PrefixHeaders { .. } => true,
            MemberBinding::Payload { .. } | MemberBinding::Body { .. } => !body.is_empty(),
            MemberBinding::Label { .. }
            | MemberBinding::QueryParam { .. }
            | MemberBinding::QueryParams { .. } => false,
        })
        .collect()
}

/// The codec's struct reader type, used for lazy body deserialization.
type BodyStructReader<'de, C> =
    <<C as Codec>::Deserializer<'de> as de::Deserializer<'de>>::StructReader;

/// Struct reader that routes member reads to HTTP binding locations.
struct HttpStructReader<'a, 'b, 'de, C: Codec> {
    codec: &'a C,
    body: &'de [u8],
    status_code: u16,
    headers: &'de http::HeaderMap,
    // Remaining bindings slice, advanced with each member.
    remaining: &'b [MemberBinding<'b>],
    // The binding found by `read_member`, used by `read_value`.
    current_binding: Option<&'b MemberBinding<'b>>,
    // Pre-computed presence for each binding position.
    has_value: Vec<bool>,
    // Total number of members (for computing position from remaining length).
    total_members: usize,
    output_schema: &'b Schema,
    // Lazy body struct reader — created on first Body member, reused for subsequent ones.
    body_reader: Option<BodyStructReader<'de, C>>,
}

impl<'a, 'b, 'de, C: Codec> de::StructReader<'de> for HttpStructReader<'a, 'b, 'de, C> {
    type Error = HttpBindingError;

    fn read_member<'s>(&mut self, schema: &'s Schema) -> Result<Option<&'s Schema>, Self::Error> {
        while let Some((binding, rest)) = self.remaining.split_first() {
            self.remaining = rest;
            let idx = self.total_members - self.remaining.len() - 1;

            if self.has_value[idx] {
                self.current_binding = Some(binding);
                let (_, member_schema) = schema
                    .members()
                    .get_index(idx)
                    .ok_or_else(|| HttpBindingError::new("member index out of bounds"))?;
                return Ok(Some(member_schema));
            }
        }

        Ok(None)
    }

    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<T, Self::Error> {
        let binding = self
            .current_binding
            .take()
            .ok_or_else(|| HttpBindingError::new("read_value called without read_member"))?;

        match binding {
            MemberBinding::StatusCode { .. } => {
                T::deserialize_with_schema(schema, StatusCodeDeserializer(self.status_code))
            }
            MemberBinding::Header { name, .. } => {
                let value = self
                    .headers
                    .get(name)
                    .ok_or_else(|| HttpBindingError::new(format!("missing header: {name}")))?
                    .to_str()
                    .map_err(|_| HttpBindingError::new(format!("invalid header value: {name}")))?;
                T::deserialize_with_schema(schema, HeaderDeserializer::new(value))
            }
            MemberBinding::PrefixHeaders { prefix, .. } => T::deserialize_with_schema(
                schema,
                PrefixHeadersDeserializer {
                    headers: self.headers,
                    prefix: prefix.as_str(),
                },
            ),
            MemberBinding::Payload { .. } => {
                T::deserialize_with_schema(schema, self.codec.deserializer(self.body))
                    .map_err(|e| HttpBindingError::new(e.to_string()))
            }
            MemberBinding::Body { .. } => {
                let parent_schema = self.output_schema;
                if self.body_reader.is_none() {
                    let reader = self
                        .codec
                        .deserializer(self.body)
                        .read_struct(parent_schema)
                        .map_err(|e| HttpBindingError::new(e.to_string()))?;
                    self.body_reader = Some(reader);
                }
                let body_reader = self.body_reader.as_mut().unwrap();
                body_reader
                    .read_member(parent_schema)
                    .map_err(|e| HttpBindingError::new(e.to_string()))?;
                body_reader
                    .read_value(schema)
                    .map_err(|e| HttpBindingError::new(e.to_string()))
            }
            _ => Err(HttpBindingError::new("request-only binding in response")),
        }
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.current_binding = None;
        Ok(())
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.total_members)
    }
}

/// Deserializes an `http::Request<&[u8]>` into a value by routing request
/// parts to their HTTP binding locations and delegating body content to a
/// [`Codec`].
pub struct HttpRequestDeserializer<'a, C: Codec> {
    codec: &'a C,
}

impl<'a, C: Codec> HttpRequestDeserializer<'a, C> {
    /// Create a new request deserializer backed by the given codec.
    pub fn new(codec: &'a C) -> Self {
        Self { codec }
    }

    /// Deserialize an HTTP request into a value using a pre-built operation binding.
    ///
    /// # Errors
    /// Returns [`HttpBindingError`] if deserialization fails.
    pub fn deserialize<'de, T>(
        &self,
        request: &'de http::Request<&'de [u8]>,
        input_schema: &Schema,
        op_binding: &OperationBinding<'_>,
    ) -> Result<T, HttpBindingError>
    where
        T: DeserializeWithSchema<'de>,
    {
        let labels = extract_uri_labels(request.uri().path(), &op_binding.uri_pattern)?;
        let query_params = parse_query(request.uri().query().unwrap_or(""))?;

        let deserializer = RequestHttpDeserializer {
            codec: self.codec,
            body: request.body(),
            headers: request.headers(),
            labels,
            query_params,
            bindings: &op_binding.request,
            input_schema,
        };

        T::deserialize_with_schema(input_schema, deserializer)
    }
}

// --- Percent decoding ---

/// Percent-decode a URI component.
///
/// Returns `Cow::Borrowed` if no decoding is needed, avoiding allocation.
/// When `decode_plus` is true, `+` is decoded as a space (query strings).
/// When `decode_plus` is false, `+` is treated as a literal (path segments, per RFC 3986).
///
/// Decoded bytes are accumulated into a `Vec<u8>` and validated as UTF-8 at the end,
/// so multi-byte sequences like `%C3%A9` (é) are handled correctly.
fn percent_decode(s: &str, decode_plus: bool) -> Result<Cow<'_, str>, HttpBindingError> {
    let needs_decode = if decode_plus {
        s.bytes().any(|b| b == b'%' || b == b'+')
    } else {
        s.bytes().any(|b| b == b'%')
    };
    if !needs_decode {
        return Ok(Cow::Borrowed(s));
    }

    let bytes = s.as_bytes();
    let mut buf: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut start = 0;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' {
            // Flush the plain run
            if start < i {
                buf.extend_from_slice(&bytes[start..i]);
            }
            let high = bytes.get(i + 1).copied().and_then(hex_digit);
            let low = bytes.get(i + 2).copied().and_then(hex_digit);
            match (high, low) {
                (Some(h), Some(l)) => buf.push(h << 4 | l),
                _ => {
                    return Err(HttpBindingError::new(format!(
                        "invalid percent-encoding in '{s}'"
                    )));
                }
            }
            i += 3;
            start = i;
        } else if decode_plus && b == b'+' {
            if start < i {
                buf.extend_from_slice(&bytes[start..i]);
            }
            buf.push(b' ');
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }

    // Flush remaining plain run
    if start < bytes.len() {
        buf.extend_from_slice(&bytes[start..]);
    }

    String::from_utf8(buf)
        .map(Cow::Owned)
        .map_err(|_| HttpBindingError::new(format!("invalid UTF-8 after percent-decoding '{s}'")))
}

fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

// --- URI label extraction — positional values in pattern order ---

/// Extract URI label values from a request path, in URI pattern order.
/// Returns `Cow::Borrowed` for non-encoded segments (common case), avoiding allocation.
///
/// TODO(performance): Maybe `skip_validate` flag to skip literal segment decoding/comparison
/// when server-side routing has already matched the path, might not be necessary depending on
/// how we resolve the operation
fn extract_uri_labels<'a>(
    path: &'a str,
    pattern: &UriPattern,
) -> Result<Vec<Cow<'a, str>>, HttpBindingError> {
    let mut labels = Vec::with_capacity(4);
    let mut segments = path.split('/').filter(|s| !s.is_empty());

    for segment in pattern.segments() {
        match segment {
            UriSegment::Literal(lit) => {
                let raw = segments.next().ok_or_else(|| {
                    HttpBindingError::new(format!("URI path too short, expected literal '{lit}'"))
                })?;
                let actual = percent_decode(raw, false)?;
                if actual != *lit {
                    return Err(HttpBindingError::new(format!(
                        "URI mismatch: expected '{lit}', got '{actual}'"
                    )));
                }
            }
            UriSegment::Label(name) => {
                let raw = segments.next().ok_or_else(|| {
                    HttpBindingError::new(format!("URI path too short, expected label '{name}'"))
                })?;
                labels.push(percent_decode(raw, false)?);
            }
            UriSegment::GreedyLabel(name) => {
                let first = segments.next().ok_or_else(|| {
                    HttpBindingError::new(format!(
                        "URI path too short, expected greedy label '{name}'"
                    ))
                })?;
                let mut value = percent_decode(first, false)?.into_owned();
                for raw in segments.by_ref() {
                    value.push('/');
                    value.push_str(&percent_decode(raw, false)?);
                }
                labels.push(Cow::Owned(value));
            }
        }
    }

    Ok(labels)
}

// --- Query string parsing — name-value pairs ---

/// Parsed query parameter pair: (key, value) as `Cow` to avoid allocation for non-encoded values.
type QueryPair<'a> = (Cow<'a, str>, Cow<'a, str>);

/// Parse query string into (key, value) pairs.
/// Returns `Cow::Borrowed` for non-encoded keys/values (common case), avoiding allocation.
fn parse_query(query: &str) -> Result<Vec<QueryPair<'_>>, HttpBindingError> {
    let mut params = Vec::new();

    if query.is_empty() {
        return Ok(params);
    }

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        params.push((percent_decode(key, true)?, percent_decode(value, true)?));
    }

    Ok(params)
}

// --- RequestHttpDeserializer — top-level deserializer for HTTP requests ---

struct RequestHttpDeserializer<'a, 'b, 'de, C: Codec> {
    codec: &'a C,
    body: &'de [u8],
    headers: &'de http::HeaderMap,
    labels: Vec<Cow<'de, str>>,
    query_params: Vec<QueryPair<'de>>,
    bindings: &'b MemberBindings<'b>,
    input_schema: &'b Schema,
}

impl<'a, 'b, 'de, C: Codec> de::Deserializer<'de> for RequestHttpDeserializer<'a, 'b, 'de, C> {
    type Error = HttpBindingError;
    type StructReader = RequestHttpStructReader<'a, 'b, 'de, C>;
    type ListReader = Never<Self::Error>;
    type MapReader = Never<Self::Error>;

    fn read_struct(self, _schema: &Schema) -> Result<Self::StructReader, Self::Error> {
        let total_members = self.bindings.len();
        let query_keys = self.bindings.query_keys();
        let presence = compute_request_presence(
            self.bindings.as_slice(),
            self.headers,
            &self.query_params,
            query_keys,
            self.body,
        );
        Ok(RequestHttpStructReader {
            codec: self.codec,
            body: self.body,
            headers: self.headers,
            labels: self.labels,
            query_params: self.query_params,
            query_keys,
            remaining: self.bindings.as_slice(),
            current_binding: None,
            presence,
            total_members,
            input_schema: self.input_schema,
            body_reader: None,
        })
    }
}

/// Pre-computed presence state for a request member binding.
#[derive(Clone, Copy)]
enum MemberPresence {
    // No value present — member will be skipped.
    Absent,
    // Value present — routed by binding type at `read_value` time.
    Present,
    // Query param present at the given index in the `query_params` vec.
    QueryParamAt(usize),
}

impl MemberPresence {
    #[inline]
    fn is_present(self) -> bool {
        !matches!(self, Self::Absent)
    }
}

/// Pre-compute which request bindings have values present.
/// For `QueryParam` bindings, caches the index into `query_params` to avoid a second linear scan.
fn compute_request_presence(
    bindings: &[MemberBinding<'_>],
    headers: &http::HeaderMap,
    query_params: &[QueryPair<'_>],
    query_keys: &[&str],
    body: &[u8],
) -> Vec<MemberPresence> {
    bindings
        .iter()
        .map(|binding| match binding {
            // Labels are always present — extract_uri_labels would have failed otherwise.
            MemberBinding::Label { .. } => MemberPresence::Present,
            MemberBinding::QueryParam { key, .. } => query_params
                .iter()
                .position(|(k, _)| k.as_ref() == key.as_str())
                .map_or(MemberPresence::Absent, MemberPresence::QueryParamAt),
            MemberBinding::QueryParams { .. } => {
                if query_params
                    .iter()
                    .any(|(k, _)| !query_keys.contains(&k.as_ref()))
                {
                    MemberPresence::Present
                } else {
                    MemberPresence::Absent
                }
            }
            MemberBinding::Header { name, .. } => {
                if headers.contains_key(name) {
                    MemberPresence::Present
                } else {
                    MemberPresence::Absent
                }
            }
            MemberBinding::PrefixHeaders { .. } => MemberPresence::Present,
            MemberBinding::Payload { .. } | MemberBinding::Body { .. } => {
                if body.is_empty() {
                    MemberPresence::Absent
                } else {
                    MemberPresence::Present
                }
            }
            MemberBinding::StatusCode { .. } => MemberPresence::Absent,
        })
        .collect()
}

/// Struct reader that routes member reads to HTTP binding locations for requests.
struct RequestHttpStructReader<'a, 'b, 'de, C: Codec> {
    codec: &'a C,
    body: &'de [u8],
    headers: &'de http::HeaderMap,
    labels: Vec<Cow<'de, str>>,
    query_params: Vec<QueryPair<'de>>,
    // Keys of `@httpQuery`-bound members (for filtering `@httpQueryParams`).
    query_keys: &'b [&'b str],
    // Remaining bindings slice, advanced with each member.
    remaining: &'b [MemberBinding<'b>],
    // The binding found by `read_member`, used by `read_value`.
    current_binding: Option<&'b MemberBinding<'b>>,
    // Pre-computed presence/index for each binding position.
    presence: Vec<MemberPresence>,
    // Total number of members (for computing position from remaining length).
    total_members: usize,
    input_schema: &'b Schema,
    // Lazy body struct reader — created on first Body member, reused for subsequent ones.
    body_reader: Option<BodyStructReader<'de, C>>,
}

impl<'a, 'b, 'de, C: Codec> de::StructReader<'de> for RequestHttpStructReader<'a, 'b, 'de, C> {
    type Error = HttpBindingError;

    fn read_member<'s>(&mut self, schema: &'s Schema) -> Result<Option<&'s Schema>, Self::Error> {
        while let Some((binding, rest)) = self.remaining.split_first() {
            self.remaining = rest;
            let idx = self.total_members - self.remaining.len() - 1;

            if self.presence[idx].is_present() {
                self.current_binding = Some(binding);
                let (_, member_schema) = schema
                    .members()
                    .get_index(idx)
                    .ok_or_else(|| HttpBindingError::new("member index out of bounds"))?;
                return Ok(Some(member_schema));
            }
        }

        Ok(None)
    }

    fn read_value<T: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<T, Self::Error> {
        let binding = self
            .current_binding
            .take()
            .ok_or_else(|| HttpBindingError::new("read_value called without read_member"))?;

        // Retrieve the current member's presence (for QueryParamAt index).
        let idx = self.total_members - self.remaining.len() - 1;
        let presence = self.presence[idx];

        match binding {
            MemberBinding::Label { pattern_index, .. } => {
                let value = self
                    .labels
                    .get(*pattern_index)
                    .ok_or_else(|| HttpBindingError::new("missing label value"))?;
                T::deserialize_with_schema(schema, LabelDeserializer::new(value.as_ref()))
            }
            MemberBinding::QueryParam { key, .. } => {
                let value = match presence {
                    MemberPresence::QueryParamAt(qi) => self.query_params[qi].1.as_ref(),
                    _ => {
                        return Err(HttpBindingError::new(format!("missing query param: {key}")));
                    }
                };
                T::deserialize_with_schema(schema, QueryDeserializer::new(value))
            }
            MemberBinding::QueryParams { .. } => T::deserialize_with_schema(
                schema,
                QueryParamsDeserializer {
                    params: &self.query_params,
                    bound_keys: self.query_keys,
                },
            ),
            MemberBinding::Header { name, .. } => {
                let value = self
                    .headers
                    .get(name)
                    .ok_or_else(|| HttpBindingError::new(format!("missing header: {name}")))?
                    .to_str()
                    .map_err(|_| HttpBindingError::new(format!("invalid header value: {name}")))?;
                T::deserialize_with_schema(schema, HeaderDeserializer::new(value))
            }
            MemberBinding::PrefixHeaders { prefix, .. } => T::deserialize_with_schema(
                schema,
                PrefixHeadersDeserializer {
                    headers: self.headers,
                    prefix: prefix.as_str(),
                },
            ),
            MemberBinding::Payload { .. } => {
                T::deserialize_with_schema(schema, self.codec.deserializer(self.body))
                    .map_err(|e| HttpBindingError::new(e.to_string()))
            }
            MemberBinding::Body { .. } => {
                let parent_schema = self.input_schema;
                if self.body_reader.is_none() {
                    let reader = self
                        .codec
                        .deserializer(self.body)
                        .read_struct(parent_schema)
                        .map_err(|e| HttpBindingError::new(e.to_string()))?;
                    self.body_reader = Some(reader);
                }
                let body_reader = self.body_reader.as_mut().unwrap();
                body_reader
                    .read_member(parent_schema)
                    .map_err(|e| HttpBindingError::new(e.to_string()))?;
                body_reader
                    .read_value(schema)
                    .map_err(|e| HttpBindingError::new(e.to_string()))
            }
            MemberBinding::StatusCode { .. } => {
                Err(HttpBindingError::new("response-only binding in request"))
            }
        }
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.current_binding = None;
        Ok(())
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.total_members)
    }
}

// --- QueryParamsDeserializer — deserializes all query params into a map ---

struct QueryParamsDeserializer<'a, 'b, 'de> {
    params: &'a [QueryPair<'de>],
    // Keys already bound by `@httpQuery` — skipped per Smithy spec.
    bound_keys: &'b [&'b str],
}

impl<'de, 'a, 'b> de::Deserializer<'de> for QueryParamsDeserializer<'a, 'b, 'de> {
    type Error = HttpBindingError;
    type StructReader = Never<Self::Error>;
    type ListReader = Never<Self::Error>;
    type MapReader = QueryParamsMapReader<'a, 'b, 'de>;

    fn read_map(self, _schema: &Schema) -> Result<Self::MapReader, Self::Error> {
        Ok(QueryParamsMapReader {
            params: self.params,
            bound_keys: self.bound_keys,
            index: 0,
        })
    }
}

struct QueryParamsMapReader<'a, 'b, 'de> {
    params: &'a [QueryPair<'de>],
    bound_keys: &'b [&'b str],
    index: usize,
}

impl<'de, 'a, 'b> de::MapReader<'de> for QueryParamsMapReader<'a, 'b, 'de> {
    type Error = HttpBindingError;

    // TODO(performance): benchmark slice vs hashset for varying sizes
    fn read_key(&mut self) -> Result<Option<String>, Self::Error> {
        while self.index < self.params.len() {
            let (key, _) = &self.params[self.index];
            if !self.bound_keys.contains(&key.as_ref()) {
                return Ok(Some(key.as_ref().to_string()));
            }
            self.index += 1;
        }
        Ok(None)
    }

    fn read_value<V: DeserializeWithSchema<'de>>(
        &mut self,
        schema: &Schema,
    ) -> Result<V, Self::Error> {
        let (_, value) = &self.params[self.index];
        self.index += 1;
        V::deserialize_with_schema(schema, QueryDeserializer::new(value.as_ref()))
    }

    fn skip_value(&mut self) -> Result<(), Self::Error> {
        self.index += 1;
        Ok(())
    }

    fn size_hint(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- hex_digit ---

    #[test]
    fn hex_digit_digits() {
        for (byte, expected) in (b'0'..=b'9').zip(0u8..=9) {
            assert_eq!(hex_digit(byte), Some(expected));
        }
    }

    #[test]
    fn hex_digit_lowercase() {
        for (byte, expected) in (b'a'..=b'f').zip(10u8..=15) {
            assert_eq!(hex_digit(byte), Some(expected));
        }
    }

    #[test]
    fn hex_digit_uppercase() {
        for (byte, expected) in (b'A'..=b'F').zip(10u8..=15) {
            assert_eq!(hex_digit(byte), Some(expected));
        }
    }

    #[test]
    fn hex_digit_invalid() {
        assert_eq!(hex_digit(b'G'), None);
        assert_eq!(hex_digit(b'z'), None);
        assert_eq!(hex_digit(b' '), None);
        assert_eq!(hex_digit(b'%'), None);
    }

    // --- percent_decode ---

    #[test]
    fn percent_decode_no_encoding() {
        let result = percent_decode("hello", false).unwrap();
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "hello");
    }

    #[test]
    fn percent_decode_space() {
        assert_eq!(percent_decode("%20", false).unwrap(), " ");
    }

    #[test]
    fn percent_decode_multibyte_utf8_e_acute() {
        // é = U+00E9 = 0xC3 0xA9
        assert_eq!(percent_decode("%C3%A9", false).unwrap(), "é");
    }

    #[test]
    fn percent_decode_multibyte_utf8_checkmark() {
        // ✓ = U+2713 = 0xE2 0x9C 0x93
        assert_eq!(percent_decode("%E2%9C%93", false).unwrap(), "✓");
    }

    #[test]
    fn percent_decode_plus_as_space() {
        assert_eq!(percent_decode("hello+world", true).unwrap(), "hello world");
    }

    #[test]
    fn percent_decode_plus_literal() {
        let result = percent_decode("hello+world", false).unwrap();
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "hello+world");
    }

    #[test]
    fn percent_decode_mixed() {
        assert_eq!(
            percent_decode("hello%20world+foo", true).unwrap(),
            "hello world foo"
        );
        assert_eq!(
            percent_decode("hello%20world+foo", false).unwrap(),
            "hello world+foo"
        );
    }

    #[test]
    fn percent_decode_invalid_hex() {
        assert!(percent_decode("%GG", false).is_err());
    }

    #[test]
    fn percent_decode_truncated() {
        assert!(percent_decode("%2", false).is_err());
        assert!(percent_decode("abc%", false).is_err());
    }

    #[test]
    fn percent_decode_invalid_utf8() {
        // 0xFF 0xFE is not valid UTF-8
        assert!(percent_decode("%FF%FE", false).is_err());
    }

    // --- extract_uri_labels ---

    #[test]
    fn extract_single_label() {
        let pattern = UriPattern::parse("/buckets/{bucket}").unwrap();
        let labels = extract_uri_labels("/buckets/my-bucket", &pattern).unwrap();
        assert_eq!(labels, vec!["my-bucket"]);
    }

    #[test]
    fn extract_multiple_labels() {
        let pattern = UriPattern::parse("/buckets/{bucket}/objects/{key}").unwrap();
        let labels = extract_uri_labels("/buckets/b1/objects/k1", &pattern).unwrap();
        assert_eq!(labels, vec!["b1", "k1"]);
    }

    #[test]
    fn extract_greedy_label() {
        let pattern = UriPattern::parse("/buckets/{bucket}/objects/{key+}").unwrap();
        let labels = extract_uri_labels("/buckets/b1/objects/a/b/c", &pattern).unwrap();
        assert_eq!(labels, vec!["b1", "a/b/c"]);
    }

    #[test]
    fn extract_percent_encoded_label() {
        let pattern = UriPattern::parse("/items/{id}").unwrap();
        let labels = extract_uri_labels("/items/hello%20world", &pattern).unwrap();
        assert_eq!(labels, vec!["hello world"]);
    }

    #[test]
    fn extract_path_too_short() {
        let pattern = UriPattern::parse("/buckets/{bucket}/objects/{key}").unwrap();
        assert!(extract_uri_labels("/buckets/b1", &pattern).is_err());
    }

    #[test]
    fn extract_literal_mismatch() {
        let pattern = UriPattern::parse("/buckets/{bucket}").unwrap();
        assert!(extract_uri_labels("/containers/b1", &pattern).is_err());
    }

    // --- parse_query ---

    #[test]
    fn parse_query_empty() {
        assert!(parse_query("").unwrap().is_empty());
    }

    #[test]
    fn parse_query_single_pair() {
        let pairs = parse_query("key=value").unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.as_ref(), "key");
        assert_eq!(pairs[0].1.as_ref(), "value");
    }

    #[test]
    fn parse_query_multiple_pairs() {
        let pairs = parse_query("a=1&b=2&c=3").unwrap();
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0].0.as_ref(), "a");
        assert_eq!(pairs[2].0.as_ref(), "c");
    }

    #[test]
    fn parse_query_key_without_value() {
        let pairs = parse_query("flag").unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.as_ref(), "flag");
        assert_eq!(pairs[0].1.as_ref(), "");
    }

    #[test]
    fn parse_query_percent_encoded() {
        let pairs = parse_query("name=hello+world&key=%C3%A9").unwrap();
        assert_eq!(pairs[0].1.as_ref(), "hello world"); // + decoded as space
        assert_eq!(pairs[1].1.as_ref(), "é");
    }

    #[test]
    fn parse_query_empty_pairs_skipped() {
        let pairs = parse_query("a=1&&b=2&").unwrap();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0.as_ref(), "a");
        assert_eq!(pairs[1].0.as_ref(), "b");
    }
}
