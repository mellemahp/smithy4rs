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
 ```rust
 use std::fmt::Error;
 use smithy4rs_core::errors;
 use smithy4rs_core::schema::SchemaRef;
 use smithy4rs_core::serde::validation::*;

pub struct SimpleStructBuilder {
    // This field is required.
    field_a: Option<String>,
    field_b: Option<i32>,
}
impl SimpleStructBuilder {
    pub fn new() -> Self {
        Self {
            field_a: None,
            field_b: None,
        }
    }
    pub fn field_a(mut self, value: String) -> Self {
        self.field_a = Some(value);
        self
    }
    pub fn field_b(mut self, value: i32) -> Self {
        self.field_b = Some(value);
        self
    }
    pub fn build(self) -> Result<SimpleStruct, ValidationErrors> {
        let validator = MyValidator::new();
        let instance = Example {
            field_a: validator.validate_required(self.field_a)?, // -> Returns a string even if no string present.
            field_b: validate.validate_optional(field_b)?,
        };
        // If the validator encountered any errors, raise those.
        validator.results()?;
        instance
    }
}
 ```

The validator here _consumes_ the builder fields and returns an owned value. 
If the field is required and the builder does not have a value set for that field we still return 
a default value (`""`) from the `validate_required` method so that we can aggregate _all_ validation 
errors for a stucture. Consider, for example, the line with `field_b`. How would we validate that 
as well as `field_a` if `field_a` couldn't be set?


## Discussion
### Nested Shapes in builders
When constructing a shape with a nested shape member, a user should be able to pass in an already-constructed
shape as the input: 

```rust
fn example() {
    let nested = Nested::builder().field_a("A string field").build()?; // Raise any errors
    let myShape = MyShape::builder().nested(nested).build()?;
}
```


## What if two different validators are used?
What happens though if the `myShape` builder uses a _different_ validator? For example: 
```rust
fn example() {
    let nested = Nested::builder().field_a("A string field").build()?; // Raise any errors
    let myShape = MyShape::builder().nested(nested).build<MyCustomValidator>()?;
}
```

With the new constraints of the `MyCustomValidator`, the nested shape may no longer be considered valid!

We will not try to solve this generally as this is a pretty niche use case. Instead, to support this 
users can manually use the `Into<BuilderType>` implementation that will be generated for 
all `Buildable` shapes. The `Into` implementation is also just generally useful for updating/modifying Shapes.

## Okay, but doest that mean that we need to support both builders and built shapes?
Yeah

## List assembly?
How will we allow users to build vectors of shapes?

## In-place validation
Consider, for a moment, the case of a list of integers.






