# Validation

Validation allows us to compare a shape against a set of constraints.

## Some basic requirements
Validation in `smithy4rs` has a few fundamental requirements:
1. **Validation MUST occur AFTER deserialization** -- Validation should occur only 
   once all data has been properly unmarshalled into a Builder. Multiple different
   sources could be deserialized into a single shape definition, so ALL deserialization
   must be completed before validation can occur. This also avoids validating multiple times
   if multiple sources are used. It does unfortunately mean that a second pass over the
   data is required, but I believe that is a worthwhile tradeoff.
2. **Validation and deserialization errors MUST be distinct** -- For example they should 
   not use some shared error trait. They should also not be raised by the same method (i.e. not 
   both raised by `deserialize`). This allows users to clearly distinguish _where_ issues 
   occured in the processing pipeline.
3. **Validation must aggregate all errors from all nested types** -- Users should get a single
   error for _ALL_ of their validation errors so they can fix them all at once. **NOTE**: Validation
   errors are the ONLY errors that we will aggregate in this way.
4. **Validation must have a depth limit** -- If we allowed validation to walk arbitrarily deep into
   a shape tree then it would be relatively easy to implement a DOS attack against any Document or 
   recursive types.

## Build and Validation

We do not want users to be able to manually construct invalid shapes by default.
For `smithy4rs`, this would require that [`Builder`] implementations are validated on [`build`].

 If a user has a different definition of "Invalid" that is fine, but they have to encode that in a
 custom [`ValidationStrategy`] implementation.  For example, if you don't care if a response from a
 server missed a `@required` value, then you could create a `clientValidator` that encodes
 that mode/behavior.

### Solution Validation Strategy
What all of the above leads us to is an implementation of validation where a `build` method takes
a generic validation strategy (see: [strategy pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/strategy.html)). 

There will be a default strategy that has a maximum depth of 20. The 20 number is just a guess.
More nesting than that is unlikely to be necesssary.

#### Basic user experience

 Basically, I expect shape construction to look like:
 ```rust, ignore
 let shape = Shape::builder()
     .deserialize(deserializer)? // this will raise any deserialization errors
     .build(validator // optional)? // this with raise any build/validation errors
 ```

 This allows us to neatly separate validation and serialization exceptions so we can
 clearly understand _where_ in the request/response pipeline an error occured.

 Ok... more examples. As with Smithy-java we want to be able to
 ```rust,ignore
 use std::fmt::Error;
 use smithy4rs_core::errors;
 use smithy4rs_core::schema::SchemaRef;
 use smithy4rs_core::serde::validation::*;

  /// Example type
 struct Test {
     a: Option<String>,
     nest: Nested
 }

 struct TestBuilder {
     a: Option<String>,
     b: Option<Nested>
 }

 struct Nested {
     c: String
 }

 struct NestedBuilder {
     c: Option<String>
 }
 static MEMBER_A: &SchemaRef = todo!();
 static MEMBER_B: &SchemaRef = todo!();
 /// NOTE: Consumes self
 impl TestBuilder {
 fn build_impl(self, validator: &mut impl Validator) -> Result<Self, ValidationErrors> {
     let errors = ValidationErrors::new();
     let value = Self {
         a: validator.validate_optional_and_extend(&MEMBER_A, self.a, errors).map_err(errors::extend),
         b: validator.validate_required_and_extend(&MEMBER_B, self.b).map_err(errors::extend)
     };
     if errors.is_empty() {
         Ok(value)
     } else {
         Err(errors)
     }
  }
 }
 ```

### A note on defaults

