//! HTTP request and response serialization.
//!
//! Routes struct members to their HTTP binding locations (URI labels, query
//! parameters, headers, status code, or body) and assembles HTTP messages.

use smithy4rs_core::{
    BigDecimal, BigInt, ByteBuffer, Instant,
    schema::{Document, Schema},
    serde::{
        codec::Codec,
        never::Never,
        se::{self, SerializeWithSchema},
    },
};

use crate::{
    binding::{MemberBinding, MemberBindings, OperationBinding},
    error::HttpBindingError,
    header::{HeaderSerializer, PrefixHeadersSerializer},
    label::LabelSerializer,
    query::{QueryParamsSerializer, QuerySerializer},
    uri::{UriPattern, UriSegment},
};

/// Serializer that captures an integer value as an HTTP status code.
///
/// Only `write_integer` is implemented; all other methods use defaults
/// (which return "not supported" errors).
struct StatusCodeSerializer<'a> {
    out: &'a mut Option<u16>,
}

impl se::Serializer for StatusCodeSerializer<'_> {
    type Error = HttpBindingError;
    type Ok = ();
    type ListWriter = Never<Self::Error>;
    type MapWriter = Never<Self::Error>;
    type StructWriter = Never<Self::Error>;

    fn write_integer(self, _schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error> {
        let code = u16::try_from(value)
            .map_err(|_| HttpBindingError::new(format!("status code {value} out of u16 range")))?;
        if !(100..=599).contains(&code) {
            return Err(HttpBindingError::new(format!(
                "status code {code} out of valid range [100, 599]"
            )));
        }
        *self.out = Some(code);
        Ok(())
    }

    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Collected HTTP request parts (labels, headers, query params).
struct RequestParts<'b> {
    // Label values indexed by URI pattern position.
    labels: Vec<Option<String>>,
    // Headers as (pre-parsed name, value) pairs.
    headers: Vec<(&'b http::header::HeaderName, String)>,
    // Query params as (key, value) pairs.
    query_params: Vec<(&'b str, String)>,
    // Headers from `@httpPrefixHeaders` — names are computed at runtime.
    prefix_headers: Vec<(String, String)>,
    // Query params from `@httpQueryParams` — keys are computed at runtime.
    dynamic_query_params: Vec<(String, String)>,
    // Reusable scratch buffer for serializing scalar values.
    scratch: String,
}

impl<'b> RequestParts<'b> {
    fn new(label_count: usize) -> Self {
        Self {
            labels: vec![None; label_count],
            headers: Vec::with_capacity(8),
            query_params: Vec::with_capacity(8),
            prefix_headers: Vec::new(),
            dynamic_query_params: Vec::new(),
            scratch: String::with_capacity(64),
        }
    }
}

/// A [`se::Serializer`] that intercepts struct serialization to route members
/// to their HTTP binding locations.
///
/// For non-struct values (e.g. `@httpPayload` on a scalar), the call is
/// forwarded directly to the inner codec serializer.
struct RequestHttpSerializer<'a, 'b, S: se::Serializer> {
    codec_serializer: Option<S>,
    parts: &'a mut RequestParts<'b>,
    bindings: &'b MemberBindings<'b>,
}

impl<'a, 'b, S: se::Serializer<Ok = ()>> se::Serializer for RequestHttpSerializer<'a, 'b, S> {
    type Error = HttpBindingError;
    type Ok = ();
    type ListWriter = Never<Self::Error>;
    type MapWriter = Never<Self::Error>;
    type StructWriter = RequestHttpStructWriter<'a, 'b, S>;

    fn write_struct(self, schema: &Schema, _len: usize) -> Result<Self::StructWriter, Self::Error> {
        Ok(RequestHttpStructWriter {
            codec_serializer: self.codec_serializer,
            body_struct: None,
            parts: self.parts,
            remaining: self.bindings.as_slice(),
            parent_schema: schema.clone(),
            body_member_count: self.bindings.body_members().len(),
        })
    }

    fn write_list(self, _schema: &Schema, _len: usize) -> Result<Self::ListWriter, Self::Error> {
        Err(HttpBindingError::new(
            "HTTP operation input must be a struct",
        ))
    }

    fn write_map(self, _schema: &Schema, _len: usize) -> Result<Self::MapWriter, Self::Error> {
        Err(HttpBindingError::new(
            "HTTP operation input must be a struct",
        ))
    }

    fn write_boolean(self, schema: &Schema, value: bool) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_boolean(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_byte(self, schema: &Schema, value: i8) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_byte(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_short(self, schema: &Schema, value: i16) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_short(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_integer(self, schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_integer(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_long(self, schema: &Schema, value: i64) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_long(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_float(self, schema: &Schema, value: f32) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_float(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_double(self, schema: &Schema, value: f64) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_double(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_big_integer(self, schema: &Schema, value: &BigInt) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_big_integer(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_big_decimal(
        self,
        schema: &Schema,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_big_decimal(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_string(self, schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_string(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_blob(self, schema: &Schema, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_blob(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_timestamp(self, schema: &Schema, value: &Instant) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_timestamp(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_document(
        self,
        schema: &Schema,
        value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_document(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_null(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_null(schema)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// [`se::StructWriter`] that routes each member to the appropriate HTTP
/// binding location based on the [`MemberBindings`].
struct RequestHttpStructWriter<'a, 'b, S: se::Serializer> {
    // The codec serializer, consumed when body or payload members are encountered.
    codec_serializer: Option<S>,
    // Lazy-initialized body struct serializer (for `MemberBinding::Body` members).
    body_struct: Option<S::StructWriter>,
    parts: &'a mut RequestParts<'b>,
    // Remaining bindings slice, advanced with each member.
    remaining: &'b [MemberBinding<'b>],
    parent_schema: Schema,
    body_member_count: usize,
}

impl<'a, 'b, S: se::Serializer<Ok = ()>> RequestHttpStructWriter<'a, 'b, S> {
    /// Lazily initialize the body struct serializer if not already created.
    fn ensure_body_struct(&mut self) -> Result<&mut S::StructWriter, HttpBindingError> {
        if self.body_struct.is_none() {
            let ser = self
                .codec_serializer
                .take()
                .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?;
            let body_struct = ser
                .write_struct(&self.parent_schema, self.body_member_count)
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
            self.body_struct = Some(body_struct);
        }
        Ok(self.body_struct.as_mut().expect("body_struct was just set"))
    }

    /// Advance the remaining bindings slice and return the next binding.
    fn next_binding(&mut self) -> Result<&'b MemberBinding<'b>, HttpBindingError> {
        let (binding, rest) = self
            .remaining
            .split_first()
            .ok_or_else(|| HttpBindingError::new("more members written than bindings declared"))?;
        self.remaining = rest;
        Ok(binding)
    }
}

impl<'a, 'b, S: se::Serializer<Ok = ()>> se::StructWriter for RequestHttpStructWriter<'a, 'b, S> {
    type Error = HttpBindingError;
    type Ok = ();

    fn write_member<T>(&mut self, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let binding = self.next_binding()?;

        match binding {
            MemberBinding::Label { pattern_index, .. } => {
                self.parts.scratch.clear();
                value.serialize_with_schema(
                    member_schema,
                    LabelSerializer::new(&mut self.parts.scratch),
                )?;
                self.parts.labels[*pattern_index] = Some(self.parts.scratch.clone());
            }
            MemberBinding::Header { name, .. } => {
                self.parts.scratch.clear();
                value.serialize_with_schema(
                    member_schema,
                    HeaderSerializer::new(&mut self.parts.scratch),
                )?;
                if !self.parts.scratch.is_empty() {
                    self.parts.headers.push((name, self.parts.scratch.clone()));
                }
            }
            MemberBinding::QueryParam { key, .. } => {
                self.parts.scratch.clear();
                value.serialize_with_schema(
                    member_schema,
                    QuerySerializer::new(&mut self.parts.scratch),
                )?;
                if !self.parts.scratch.is_empty() {
                    self.parts
                        .query_params
                        .push((key.as_str(), self.parts.scratch.clone()));
                }
            }
            MemberBinding::Body { .. } => {
                let body = self.ensure_body_struct()?;
                body.write_member(member_schema, value)
                    .map_err(|e| HttpBindingError::new(e.to_string()))?;
            }
            MemberBinding::Payload { .. } => {
                let ser = self
                    .codec_serializer
                    .take()
                    .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?;
                value
                    .serialize_with_schema(member_schema, ser)
                    .map_err(|e| HttpBindingError::new(e.to_string()))?;
            }
            MemberBinding::PrefixHeaders { prefix, .. } => {
                value.serialize_with_schema(
                    member_schema,
                    PrefixHeadersSerializer::new(prefix, &mut self.parts.prefix_headers),
                )?;
            }
            MemberBinding::QueryParams { .. } => {
                value.serialize_with_schema(
                    member_schema,
                    QueryParamsSerializer::new(&mut self.parts.dynamic_query_params),
                )?;
            }
            // StatusCode not handled in request serialization
            MemberBinding::StatusCode { .. } => {}
        }
        Ok(())
    }

    fn skip_member(&mut self, _schema: &Schema) -> Result<(), Self::Error> {
        self.next_binding()?;
        Ok(())
    }

    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        if let Some(body_struct) = self.body_struct {
            body_struct
                .end(schema)
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
        }
        Ok(())
    }
}

// --- build_uri — interpolates labels and appends query params ---

fn build_uri(pattern: &UriPattern, parts: &RequestParts<'_>) -> Result<String, HttpBindingError> {
    let capacity = estimate_uri_capacity(pattern, parts);
    let mut path = String::with_capacity(capacity);

    let mut label_iter = parts.labels.iter();
    for segment in pattern.segments() {
        path.push('/');
        match segment {
            UriSegment::Literal(lit) => path.push_str(lit),
            UriSegment::Label(name) => {
                let value = label_iter
                    .next()
                    .and_then(|v| v.as_deref())
                    .ok_or_else(|| HttpBindingError::new(format!("missing label: {name}")))?;
                percent_encode_segment(&mut path, value);
            }
            UriSegment::GreedyLabel(name) => {
                let value = label_iter
                    .next()
                    .and_then(|v| v.as_deref())
                    .ok_or_else(|| {
                        HttpBindingError::new(format!("missing greedy label: {name}"))
                    })?;
                path.push_str(value);
            }
        }
    }

    if path.is_empty() {
        path.push('/');
    }

    // Chain query parameters: pattern literals + bound params + dynamic params
    let query_pairs = pattern
        .query_literals()
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .chain(parts.query_params.iter().map(|(k, v)| (*k, v.as_str())))
        .chain(
            parts
                .dynamic_query_params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str())),
        );

    let mut first = true;
    for (key, value) in query_pairs {
        if first {
            path.push('?');
            first = false;
        } else {
            path.push('&');
        }
        percent_encode_segment(&mut path, key);
        if !value.is_empty() {
            path.push('=');
            percent_encode_segment(&mut path, value);
        }
    }

    Ok(path)
}

/// Estimate URI capacity from pattern structure and collected values.
fn estimate_uri_capacity(pattern: &UriPattern, parts: &RequestParts<'_>) -> usize {
    let mut cap = 0;
    let mut label_iter = parts.labels.iter();
    for seg in pattern.segments() {
        cap += 1; // '/'
        match seg {
            UriSegment::Literal(lit) => cap += lit.len(),
            UriSegment::Label(_) | UriSegment::GreedyLabel(_) => {
                cap += label_iter
                    .next()
                    .and_then(|v| v.as_ref())
                    .map_or(16, |v| v.len());
            }
        }
    }
    if cap == 0 {
        cap = 1; // "/"
    }
    // Query params
    for (k, v) in pattern
        .query_literals()
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .chain(parts.query_params.iter().map(|(k, v)| (*k, v.as_str())))
        .chain(
            parts
                .dynamic_query_params
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str())),
        )
    {
        cap += 1 + k.len(); // &key or ?key
        if !v.is_empty() {
            cap += 1 + v.len(); // =value
        }
    }
    cap
}

/// Percent-encode a URI path segment (RFC 3986 unreserved characters).
///
/// Batches consecutive unreserved bytes into a single `push_str` (memcpy)
/// instead of pushing one character at a time. Uses a lookup table for
/// unreserved classification and direct hex digit lookup instead of
/// `write!`/`fmt::Formatter` machinery.
fn percent_encode_segment(out: &mut String, value: &str) {
    #[rustfmt::skip]
    static UNRESERVED: [bool; 256] = {
        let mut table = [false; 256];
        let mut i = b'A'; while i <= b'Z' { table[i as usize] = true; i += 1; }
        i = b'a'; while i <= b'z' { table[i as usize] = true; i += 1; }
        i = b'0'; while i <= b'9' { table[i as usize] = true; i += 1; }
        table[b'-' as usize] = true;
        table[b'_' as usize] = true;
        table[b'.' as usize] = true;
        table[b'~' as usize] = true;
        table
    };
    static HEX: [u8; 16] = *b"0123456789ABCDEF";

    let bytes = value.as_bytes();
    let mut start = 0;

    for (i, &b) in bytes.iter().enumerate() {
        if !UNRESERVED[b as usize] {
            // Flush the unreserved run as a single memcpy
            if start < i {
                out.push_str(&value[start..i]);
            }
            out.push('%');
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0F) as usize] as char);
            start = i + 1;
        }
    }

    // Flush remaining (or entire string if nothing needed encoding)
    if start < bytes.len() {
        out.push_str(&value[start..]);
    }
}

/// Serializes a value into an `http::Request<Vec<u8>>` by routing struct
/// members to their HTTP binding locations and delegating body content to a
/// [`Codec`].
pub struct HttpRequestSerializer<'a, C: Codec> {
    codec: &'a C,
}

impl<'a, C: Codec> HttpRequestSerializer<'a, C> {
    /// Create a new request serializer backed by the given codec.
    pub fn new(codec: &'a C) -> Self {
        Self { codec }
    }

    /// Serialize a value into an HTTP request using a pre-built operation binding.
    pub fn serialize<T: SerializeWithSchema>(
        &self,
        value: &T,
        input_schema: &Schema,
        op_binding: &OperationBinding<'_>,
    ) -> Result<http::Request<Vec<u8>>, HttpBindingError> {
        let has_body = !op_binding.request.body_members().is_empty()
            || op_binding.request.payload_member().is_some();
        let mut body_buf = Vec::with_capacity(if has_body { 1024 } else { 0 });
        let mut parts = RequestParts::new(op_binding.request.label_members().len());

        let codec_serializer = self.codec.serializer(&mut body_buf);
        let http_ser = RequestHttpSerializer {
            codec_serializer: Some(codec_serializer),
            parts: &mut parts,
            bindings: &op_binding.request,
        };
        value.serialize_with_schema(input_schema, http_ser)?;

        let uri = build_uri(&op_binding.uri_pattern, &parts)?;

        let uri: http::Uri = uri
            .parse()
            .map_err(|e: http::uri::InvalidUri| HttpBindingError::new(e.to_string()))?;

        let mut request = http::Request::new(body_buf);
        *request.method_mut() = op_binding.method.clone();
        *request.uri_mut() = uri;

        let header_count = parts.headers.len()
            + parts.prefix_headers.len()
            + usize::from(!request.body().is_empty());
        request.headers_mut().reserve(header_count);

        for (name, value) in &parts.headers {
            let hv = http::header::HeaderValue::from_str(value)
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
            request.headers_mut().insert((*name).clone(), hv);
        }
        for (name, value) in &parts.prefix_headers {
            let hn = http::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
            let hv = http::header::HeaderValue::from_str(value)
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
            request.headers_mut().insert(hn, hv);
        }

        if !request.body().is_empty() {
            request.headers_mut().insert(
                http::header::CONTENT_TYPE,
                http::header::HeaderValue::from_str(self.codec.media_type())
                    .map_err(|e| HttpBindingError::new(e.to_string()))?,
            );
        }

        Ok(request)
    }
}

/// Collected HTTP response parts (status code, headers).
struct ResponseParts<'b> {
    status_code: Option<u16>,
    headers: Vec<(&'b http::header::HeaderName, String)>,
    // Headers from `@httpPrefixHeaders` — names are computed at runtime.
    prefix_headers: Vec<(String, String)>,
    // Reusable scratch buffer for serializing scalar values.
    scratch: String,
}

impl<'b> ResponseParts<'b> {
    fn new() -> Self {
        Self {
            status_code: None,
            headers: Vec::with_capacity(8),
            prefix_headers: Vec::new(),
            scratch: String::with_capacity(64),
        }
    }
}

/// Serializes a value into an `http::Response<Vec<u8>>` by routing struct
/// members to their HTTP binding locations and delegating body content to a
/// [`Codec`].
pub struct HttpResponseSerializer<'a, C: Codec> {
    codec: &'a C,
}

impl<'a, C: Codec> HttpResponseSerializer<'a, C> {
    /// Create a new response serializer backed by the given codec.
    pub fn new(codec: &'a C) -> Self {
        Self { codec }
    }

    /// Serialize a value into an HTTP response using a pre-built operation binding.
    pub fn serialize<T: SerializeWithSchema>(
        &self,
        value: &T,
        output_schema: &Schema,
        op_binding: &OperationBinding<'_>,
    ) -> Result<http::Response<Vec<u8>>, HttpBindingError> {
        let has_body = !op_binding.response.body_members().is_empty()
            || op_binding.response.payload_member().is_some();
        let mut body_buf = Vec::with_capacity(if has_body { 1024 } else { 0 });
        let mut parts = ResponseParts::new();

        let codec_serializer = self.codec.serializer(&mut body_buf);
        let http_ser = ResponseHttpSerializer {
            codec_serializer: Some(codec_serializer),
            parts: &mut parts,
            bindings: &op_binding.response,
            body_member_count: op_binding.response.body_members().len(),
        };

        value.serialize_with_schema(output_schema, http_ser)?;

        let status = parts.status_code.unwrap_or(op_binding.default_status);

        let mut response = http::Response::new(body_buf);
        *response.status_mut() =
            http::StatusCode::from_u16(status).map_err(|e| HttpBindingError::new(e.to_string()))?;

        let header_count = parts.headers.len()
            + parts.prefix_headers.len()
            + usize::from(!response.body().is_empty());
        response.headers_mut().reserve(header_count);

        for (name, value) in &parts.headers {
            let hv = http::header::HeaderValue::from_str(value)
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
            response.headers_mut().insert((*name).clone(), hv);
        }
        for (name, value) in &parts.prefix_headers {
            let hn = http::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
            let hv = http::header::HeaderValue::from_str(value)
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
            response.headers_mut().insert(hn, hv);
        }

        if !response.body().is_empty() {
            response.headers_mut().insert(
                http::header::CONTENT_TYPE,
                http::header::HeaderValue::from_str(self.codec.media_type())
                    .map_err(|e| HttpBindingError::new(e.to_string()))?,
            );
        }

        Ok(response)
    }
}

// --- ResponseHttpSerializer — top-level serializer for responses ---

struct ResponseHttpSerializer<'a, 'b, S: se::Serializer> {
    codec_serializer: Option<S>,
    parts: &'a mut ResponseParts<'b>,
    bindings: &'b MemberBindings<'b>,
    body_member_count: usize,
}

impl<'a, 'b, S: se::Serializer<Ok = ()>> se::Serializer for ResponseHttpSerializer<'a, 'b, S> {
    type Error = HttpBindingError;
    type Ok = ();
    type ListWriter = Never<Self::Error>;
    type MapWriter = Never<Self::Error>;
    type StructWriter = ResponseHttpStructWriter<'a, 'b, S>;

    fn write_struct(self, schema: &Schema, _len: usize) -> Result<Self::StructWriter, Self::Error> {
        Ok(ResponseHttpStructWriter {
            codec_serializer: self.codec_serializer,
            body_struct: None,
            parts: self.parts,
            remaining: self.bindings.as_slice(),
            parent_schema: schema.clone(),
            body_member_count: self.body_member_count,
        })
    }

    fn write_list(self, _schema: &Schema, _len: usize) -> Result<Self::ListWriter, Self::Error> {
        Err(HttpBindingError::new(
            "HTTP operation output must be a struct",
        ))
    }

    fn write_map(self, _schema: &Schema, _len: usize) -> Result<Self::MapWriter, Self::Error> {
        Err(HttpBindingError::new(
            "HTTP operation output must be a struct",
        ))
    }

    fn write_boolean(self, schema: &Schema, value: bool) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_boolean(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_byte(self, schema: &Schema, value: i8) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_byte(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_short(self, schema: &Schema, value: i16) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_short(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_integer(self, schema: &Schema, value: i32) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_integer(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_long(self, schema: &Schema, value: i64) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_long(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_float(self, schema: &Schema, value: f32) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_float(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_double(self, schema: &Schema, value: f64) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_double(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_big_integer(self, schema: &Schema, value: &BigInt) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_big_integer(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_big_decimal(
        self,
        schema: &Schema,
        value: &BigDecimal,
    ) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_big_decimal(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_string(self, schema: &Schema, value: &str) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_string(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_blob(self, schema: &Schema, value: &ByteBuffer) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_blob(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_timestamp(self, schema: &Schema, value: &Instant) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_timestamp(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_document(
        self,
        schema: &Schema,
        value: &Box<dyn Document>,
    ) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_document(schema, value)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn write_null(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        self.codec_serializer
            .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?
            .write_null(schema)
            .map_err(|e| HttpBindingError::new(e.to_string()))
    }

    fn skip(self, _schema: &Schema) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// --- ResponseHttpStructWriter — routes response struct members by binding ---

struct ResponseHttpStructWriter<'a, 'b, S: se::Serializer> {
    codec_serializer: Option<S>,
    body_struct: Option<S::StructWriter>,
    parts: &'a mut ResponseParts<'b>,
    // Remaining bindings slice, advanced with each member.
    remaining: &'b [MemberBinding<'b>],
    parent_schema: Schema,
    body_member_count: usize,
}

impl<'a, 'b, S: se::Serializer<Ok = ()>> ResponseHttpStructWriter<'a, 'b, S> {
    /// Lazily initialize the body struct serializer if not already created.
    fn ensure_body_struct(&mut self) -> Result<&mut S::StructWriter, HttpBindingError> {
        if self.body_struct.is_none() {
            let ser = self
                .codec_serializer
                .take()
                .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?;
            let body_struct = ser
                .write_struct(&self.parent_schema, self.body_member_count)
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
            self.body_struct = Some(body_struct);
        }
        Ok(self.body_struct.as_mut().expect("body_struct was just set"))
    }

    fn next_binding(&mut self) -> Result<&'b MemberBinding<'b>, HttpBindingError> {
        let (binding, rest) = self
            .remaining
            .split_first()
            .ok_or_else(|| HttpBindingError::new("more members written than bindings declared"))?;
        self.remaining = rest;
        Ok(binding)
    }
}

impl<'a, 'b, S: se::Serializer<Ok = ()>> se::StructWriter for ResponseHttpStructWriter<'a, 'b, S> {
    type Error = HttpBindingError;
    type Ok = ();

    fn write_member<T>(&mut self, member_schema: &Schema, value: &T) -> Result<(), Self::Error>
    where
        T: SerializeWithSchema,
    {
        let binding = self.next_binding()?;

        match binding {
            MemberBinding::StatusCode { .. } => {
                value.serialize_with_schema(
                    member_schema,
                    StatusCodeSerializer {
                        out: &mut self.parts.status_code,
                    },
                )?;
            }
            MemberBinding::Header { name, .. } => {
                self.parts.scratch.clear();
                value.serialize_with_schema(
                    member_schema,
                    HeaderSerializer::new(&mut self.parts.scratch),
                )?;
                if !self.parts.scratch.is_empty() {
                    self.parts.headers.push((name, self.parts.scratch.clone()));
                }
            }
            MemberBinding::PrefixHeaders { prefix, .. } => {
                value.serialize_with_schema(
                    member_schema,
                    PrefixHeadersSerializer::new(prefix, &mut self.parts.prefix_headers),
                )?;
            }
            MemberBinding::Body { .. } => {
                let body = self.ensure_body_struct()?;
                body.write_member(member_schema, value)
                    .map_err(|e| HttpBindingError::new(e.to_string()))?;
            }
            MemberBinding::Payload { .. } => {
                let ser = self
                    .codec_serializer
                    .take()
                    .ok_or_else(|| HttpBindingError::new("body serializer already consumed"))?;
                value
                    .serialize_with_schema(member_schema, ser)
                    .map_err(|e| HttpBindingError::new(e.to_string()))?;
            }
            // Label, QueryParam, QueryParams are request-only bindings
            MemberBinding::Label { .. }
            | MemberBinding::QueryParam { .. }
            | MemberBinding::QueryParams { .. } => {}
        }
        Ok(())
    }

    fn skip_member(&mut self, _schema: &Schema) -> Result<(), Self::Error> {
        self.next_binding()?;
        Ok(())
    }

    fn end(self, schema: &Schema) -> Result<Self::Ok, Self::Error> {
        if let Some(body_struct) = self.body_struct {
            body_struct
                .end(schema)
                .map_err(|e| HttpBindingError::new(e.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use smithy4rs_core::{schema::prelude::INTEGER, serde::se::Serializer};

    use super::*;

    // --- percent_encode_segment ---

    #[test]
    fn percent_encode_unreserved_passthrough() {
        let mut out = String::new();
        percent_encode_segment(&mut out, "test-value_123.~");
        assert_eq!(out, "test-value_123.~");
    }

    #[test]
    fn percent_encode_space() {
        let mut out = String::new();
        percent_encode_segment(&mut out, "hello world");
        assert_eq!(out, "hello%20world");
    }

    #[test]
    fn percent_encode_multibyte_utf8() {
        let mut out = String::new();
        percent_encode_segment(&mut out, "café");
        assert_eq!(out, "caf%C3%A9");
    }

    #[test]
    fn percent_encode_reserved_chars() {
        let mut out = String::new();
        percent_encode_segment(&mut out, "a/b?c#d&e=f");
        assert_eq!(out, "a%2Fb%3Fc%23d%26e%3Df");
    }

    #[test]
    fn percent_encode_empty() {
        let mut out = String::new();
        percent_encode_segment(&mut out, "");
        assert_eq!(out, "");
    }

    // --- build_uri ---

    #[test]
    fn build_uri_literal_only() {
        let pattern = UriPattern::parse("/buckets").unwrap();
        let parts = RequestParts::new(0);
        let uri = build_uri(&pattern, &parts).unwrap();
        assert_eq!(uri, "/buckets");
    }

    #[test]
    fn build_uri_single_label() {
        let pattern = UriPattern::parse("/buckets/{bucket}").unwrap();
        let mut parts = RequestParts::new(1);
        parts.labels[0] = Some("my-bucket".to_string());
        let uri = build_uri(&pattern, &parts).unwrap();
        assert_eq!(uri, "/buckets/my-bucket");
    }

    #[test]
    fn build_uri_greedy_label() {
        let pattern = UriPattern::parse("/objects/{key+}").unwrap();
        let mut parts = RequestParts::new(1);
        parts.labels[0] = Some("a/b/c".to_string());
        let uri = build_uri(&pattern, &parts).unwrap();
        assert_eq!(uri, "/objects/a/b/c");
    }

    #[test]
    fn build_uri_with_query_params() {
        let pattern = UriPattern::parse("/items?format=json").unwrap();
        let mut parts = RequestParts::new(0);
        parts.query_params.push(("limit", "10".to_string()));
        parts
            .dynamic_query_params
            .push(("extra".to_string(), "yes".to_string()));
        let uri = build_uri(&pattern, &parts).unwrap();
        assert_eq!(uri, "/items?format=json&limit=10&extra=yes");
    }

    #[test]
    fn build_uri_missing_label() {
        let pattern = UriPattern::parse("/buckets/{bucket}").unwrap();
        let parts = RequestParts::new(1); // label slot is None
        assert!(build_uri(&pattern, &parts).is_err());
    }

    #[test]
    fn build_uri_empty_path() {
        let pattern = UriPattern::parse("/").unwrap();
        let parts = RequestParts::new(0);
        let uri = build_uri(&pattern, &parts).unwrap();
        assert_eq!(uri, "/");
    }

    #[test]
    fn build_uri_label_percent_encoded() {
        let pattern = UriPattern::parse("/items/{id}").unwrap();
        let mut parts = RequestParts::new(1);
        parts.labels[0] = Some("hello world".to_string());
        let uri = build_uri(&pattern, &parts).unwrap();
        assert_eq!(uri, "/items/hello%20world");
    }

    // --- StatusCodeSerializer ---

    #[test]
    fn status_code_valid() {
        let mut out = None;
        StatusCodeSerializer { out: &mut out }
            .write_integer(&INTEGER, 200)
            .unwrap();
        assert_eq!(out, Some(200));
    }

    #[test]
    fn status_code_out_of_range_low() {
        let mut out = None;
        assert!(
            StatusCodeSerializer { out: &mut out }
                .write_integer(&INTEGER, 99)
                .is_err()
        );
    }

    #[test]
    fn status_code_out_of_range_high() {
        let mut out = None;
        assert!(
            StatusCodeSerializer { out: &mut out }
                .write_integer(&INTEGER, 600)
                .is_err()
        );
    }

    #[test]
    fn status_code_negative() {
        let mut out = None;
        assert!(
            StatusCodeSerializer { out: &mut out }
                .write_integer(&INTEGER, -1)
                .is_err()
        );
    }
}
