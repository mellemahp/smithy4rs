# Document Types 

### What are “Documents”?
Documents are a protocol-agnostic representation of untyped data in the smithy data model.

The Smithy data model consists of:
- Numbers: byte, short, integer, long, float, double, bigInteger, bigDecimal. IntEnum shapes are
- represented as integers in the Smithy data model.
- boolean
- blob
- string: enum shapes are represented as strings in the Smithy data model
- timestamp: Represented as an {@link Instant}
- list: list of Documents
- map: map of int|long|string keys to Document values
- struct: structure or union

### Document Properties
1. Shape Conversion - All generated shapes should be able to be converted to/from a document
2. Lossless Serialization: Documents created from a shape should serialize exactly the same as the shape they are created from. I.e.

   ```rust
   // These two should be equivalent
   my_smithy_shape.serialize(serializer)?;
   my_smithy_shape.as_document().serialize(serializer)?;
   ```
   
3. Lossless Deserialization: Deserializing to a document then converting to a shape should be the same as deserializing to that shape. I.e. :

   ```rust
   // These two should be equivalent
   let my_shape = deserializer::deserialize::<MyShape>::();
   let my_shape = deserializer::deserialize::<Document>::().into();
   ```
   This is particularly important for dealing with errors, events, and over-the-wire polymorphism.

4. Protocol Smoothing: Documents should try to smooth over any protocol-specific incompatibilities with the smithy data model.
5. Discriminators - Documents with an encloded type should be able to be (de)serialized with data that identifies their 

## Protocol Smoothing
Because Document types are a protocol-agnostic view of untyped data, Protocol codecs should attempt to smooth over 
any protocol-specific incompatibilities with the Smithy data model when deserializing into a document type.

For example, If a protocol represents a blob as a base64 encoded string, then deserializing the value into a 
document should base64 decode the value and use the underlying bytes as the blob value in the document. 
When the document is re-serialized by the same procotol, the procotol should handle conversion from a byte 
buffer back to a base64 encoded string.

## Lossless serialization
Point two *REQUIRES* that documents created from a shape have the exact same schema as that shape.

A good example of why is to consider the following Smithy Shape:

```smithy
structure MyStruct {
    @jsonName("foo")
    bar: String
}
```

When we go to serialize an instance of this shape with a JSON protocol we would expect that the member `bar` 
is renamed to `foo` in the serialized JSON data:

```rust 
let myStructInstance = MyStruct::builder().bar("my-string").build()?;
myStructInstance.serialize(jsonSerializer);  // -> Yields `{ "foo": "my-string" }`
```

The JSON protocol will achieve this conversion by discovering the `jsonName` trait on the
schema of the `myStructInstance` `bar` member.

Another good example is the case of a `@sensitive` field on a structure:

```smithy 
structure MyStructRedacted {
   @sensitive
   password: String
}
```

`Display`-ing such a structure should result in the `@sensitive` field being redacted:
```rust
let myStructInstance = MyStruct::builder().password("Secret".to_string()).build()?;
assert_eq!("MyStruct[password:**REDACTED**]", format!("{myStructInstance}"))
```

We do _not_ want conversion to a document to suddenly start leaking a sensitive field.

In order to retain trait information Documents created from a shape MUST retain that shape’s Schema.

## Lossless Deserialization

Let’s say we want to deserialize an error type that has the schema:
```smithy
@error("client")
@http
struct MyClientError {
   @required
   message: String
   @jsonName("foo")
   bar: String
}
```

Over the wire we get a JSON blob like:

```json
{
  "__type": "com.example#MyClientError", 
  "message": "Something broke pretty bad!", 
  "foo": "quux!" 
}
```

Because the error response could be one of multiple error types, we dont know the type to deserialize it to in advance. 
We deserialize the JSON data to a Document, extract the discriminator, and use it to:

```rust
let error_document = Document::deserialize(json_deserializer)?;
let builder = error_document.get_builder(error_document.discriminator());
let output = builder.deserialize(error_document)?.build();
// This sort of type-erasure thing is hard in rust. Maybe using an enum or something
// to cast could work? Out of scope for this discussion 
```

The initial `error_document` has no schema information (beyond the base `Document` schema), so it does not perform any protocol-specific 
conversions during deserialization (i.e. it won't convert the field name `foo` to `bar` based on the `@jsonName` trait).

However, when we deserialize the document into the final Error type we need to execute the protocol-specific handling of the `jsonName` trait.

How in the world can we ensure documents can still be deserialized using the protocol-specific trait behavior we would expect? Here are the options I see:

### 1. Generic Type within documents [Rejected]
Maybe we could have a generic “DocumentTranslator” implementation that could be passed along with the

```rust
struct Document<T: DocumentTranslator = DefaultTranslator> {
    // ... other fields
    translator: PhantomData<T>
}
```

This could work pretty well when we know the type of the Protocol in advance. 
Then every protocol could return a document type with some required inner type. 
This would be neat, but unfortunately if we want to be able to dynamically load a Protocol 
then we need to allow this type to be dynamic.

This is basically not possible in rust. The type T will be required to be 
known at compile time with the above construction.

### 2. Optional Pointer
The other option is to provide a pointer to the translator implementation. This allows it to

```rust
struct Document {
    // ... other fields
    
    /// Pointer to a translator that ensures that a document deserialized from 
    /// a particular protocol retains the trait behavior of that protocol for 
    /// conversion to a typed Shape.
    ///
    /// NOTE: This should remain _private_ . It only matters for conversion to 
    /// a typed structure and should not be exposed. 
    /// Note also: Thread safety here is not an issue as immutable borrows are 
    /// generally thread safe and deserialized documents like this really shouldnt be 
    /// shared across threads anyway.
    translator: Option<Box<dyn DocumentTranslator>>
}
```

If a translator is found then that would be used for document deserialization into a typed Shape.

### 3. Smithy-Java-style
We could just try to make documents an interface and have all serde return a `dyn Document`. 
Each protocol would then implement it’s own document type. This would probably work OK, but is super not-”rusty”. 

Trait objects are not the norm in rust and are usually kept out of API’s as they are not particularly egonomic. 
A trait object is not actually much of a performance hit though relative to the previous “optional pointer” 
approach b/c it is pretty much just a type pointer anyway. I worry that having to deal with trait objects 
every time you wanted to work with a document would just be frustrating for users.

### Discriminators

Document types may have a ShapeId that indicates the type they correspond to. 
This ID can be serialized to allow consumers to handle over the wire polymorphism (primarily for over-the wire polymorphism). 
Typed documents must return the shape ID of the enclosed shape.

For example, let’s say we convert a shape of type `com.example#MyShape` into a document type. 
The document would then store the `ShapeID` as its discriminator.

Serializing this document with a JSON protocol might result in a field `__type` being added to the output JSON :

```json
{
  "__type": "com.example#MyShape",
  "other": {}
}
```

> NOTE: Discriminators are only really useful for documents (or other structs) being serialized as Structure and Union types. As such they are  serialized by the `StructureSerializer`.

Similarly, a deserializer might want to pull the `__type` data from a JSON blob when deserializing into a Document type. 
