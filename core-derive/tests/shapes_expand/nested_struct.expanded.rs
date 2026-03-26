use smithy4rs_core::{
    prelude::{INTEGER, STRING},
    smithy,
};
use smithy4rs_core_derive::SmithyShape;
#[doc(hidden)]
pub static ADDRESS_SCHEMA_BUILDER: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::Ref<::smithy4rs_core::schema::SchemaBuilder>,
> = ::smithy4rs_core::LazyLock::new(|| ::smithy4rs_core::Ref::new(
    ::smithy4rs_core::schema::Schema::structure_builder("test#Address", Vec::new()),
));
pub static ADDRESS_SCHEMA: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| {
    (&*ADDRESS_SCHEMA_BUILDER)
        .put_member("street", &STRING, Vec::new())
        .put_member("zip", &STRING, Vec::new())
        .build()
});
static _ADDRESS_SCHEMA_MEMBER_STREET: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| ADDRESS_SCHEMA.expect_member("street"));
static _ADDRESS_SCHEMA_MEMBER_ZIP: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| ADDRESS_SCHEMA.expect_member("zip"));
#[doc(hidden)]
pub static PERSON_SCHEMA_BUILDER: ::smithy4rs_core::LazyLock<
    ::smithy4rs_core::Ref<::smithy4rs_core::schema::SchemaBuilder>,
> = ::smithy4rs_core::LazyLock::new(|| ::smithy4rs_core::Ref::new(
    ::smithy4rs_core::schema::Schema::structure_builder("test#Person", Vec::new()),
));
pub static PERSON_SCHEMA: ::smithy4rs_core::LazyLock<::smithy4rs_core::schema::Schema> = ::smithy4rs_core::LazyLock::new(||
{
    (&*PERSON_SCHEMA_BUILDER)
        .put_member("name", &STRING, Vec::new())
        .put_member("age", &INTEGER, Vec::new())
        .put_member("home", &ADDRESS_SCHEMA, Vec::new())
        .put_member("work", &ADDRESS_SCHEMA, Vec::new())
        .build()
});
static _PERSON_SCHEMA_MEMBER_NAME: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| PERSON_SCHEMA.expect_member("name"));
static _PERSON_SCHEMA_MEMBER_AGE: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| PERSON_SCHEMA.expect_member("age"));
static _PERSON_SCHEMA_MEMBER_HOME: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| PERSON_SCHEMA.expect_member("home"));
static _PERSON_SCHEMA_MEMBER_WORK: ::smithy4rs_core::LazyLock<
    &::smithy4rs_core::schema::Schema,
> = ::smithy4rs_core::LazyLock::new(|| PERSON_SCHEMA.expect_member("work"));
#[smithy_schema(ADDRESS_SCHEMA)]
pub struct Address {
    #[smithy_schema(STREET)]
    pub street: String,
    #[smithy_schema(ZIP)]
    pub zip: String,
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for Address {
        fn schema() -> &'static _Schema {
            &ADDRESS_SCHEMA
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for Address {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2usize)?;
            ser.write_member_named(
                "street",
                &_ADDRESS_SCHEMA_MEMBER_STREET,
                &self.street,
            )?;
            ser.write_member_named("zip", &_ADDRESS_SCHEMA_MEMBER_ZIP, &self.zip)?;
            ser.end(schema)
        }
    }
};
#[automatically_derived]
pub struct AddressBuilder {
    street: smithy4rs_core::serde::Required<String>,
    zip: smithy4rs_core::serde::Required<String>,
}
#[automatically_derived]
impl ::core::clone::Clone for AddressBuilder {
    #[inline]
    fn clone(&self) -> AddressBuilder {
        AddressBuilder {
            street: ::core::clone::Clone::clone(&self.street),
            zip: ::core::clone::Clone::clone(&self.zip),
        }
    }
}
#[automatically_derived]
impl AddressBuilder {
    pub fn new() -> Self {
        Self {
            street: smithy4rs_core::serde::Required::Unset,
            zip: smithy4rs_core::serde::Required::Unset,
        }
    }
    pub fn street<T: Into<String>>(mut self, value: T) -> Self {
        self.street = smithy4rs_core::serde::Required::Set(value.into());
        self
    }
    pub fn zip<T: Into<String>>(mut self, value: T) -> Self {
        self.zip = smithy4rs_core::serde::Required::Set(value.into());
        self
    }
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for AddressBuilder {
        fn schema() -> &'static _Schema {
            &ADDRESS_SCHEMA
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use _smithy4rs::serde::correction::ErrorCorrection as _ErrorCorrection;
    use _smithy4rs::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    use _smithy4rs::serde::BuildWithSchema as _BuildWithSchema;
    use _smithy4rs::serde::deserializers::StructReader as _StructReader;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for AddressBuilder {
        fn deserialize_with_schema<D>(
            schema: &_Schema,
            deserializer: D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let mut builder = AddressBuilder::new();
            let mut reader = deserializer.read_struct(schema)?;
            while let Some(member_schema) = reader.read_member(schema)? {
                if member_schema == *_ADDRESS_SCHEMA_MEMBER_STREET {
                    let value: String = reader.read_value(member_schema)?;
                    builder = builder.street(value);
                    continue;
                }
                if member_schema == *_ADDRESS_SCHEMA_MEMBER_ZIP {
                    let value: String = reader.read_value(member_schema)?;
                    builder = builder.zip(value);
                    continue;
                }
                reader.skip_value()?;
            }
            Ok(builder)
        }
    }
    use smithy4rs_core::serde::BuildableShape as _BuildableShape;
    use smithy4rs_core::serde::validation::ValidationErrors as _ValidationErrors;
    use smithy4rs_core::serde::validation::Validator as _Validator;
    #[automatically_derived]
    impl _BuildableShape for Address {
        type Builder = AddressBuilder;
    }
    #[automatically_derived]
    impl _ErrorCorrection for AddressBuilder {
        type Value = Address;
        fn correct(self) -> Self::Value {
            Address {
                street: self.street.get(),
                zip: self.zip.get(),
            }
        }
    }
    #[automatically_derived]
    impl _BuildWithSchema<Address> for AddressBuilder {
        fn new() -> Self {
            Self::new()
        }
    }
    #[automatically_derived]
    impl _ErrorCorrectionDefault for Address {
        fn default() -> Self {
            AddressBuilder::new().correct()
        }
    }
    #[automatically_derived]
    impl Address {
        /// Get a new builder for this shape.
        #[must_use]
        #[inline]
        pub fn builder() -> AddressBuilder {
            <Self as _BuildableShape>::builder()
        }
    }
    #[automatically_derived]
    impl AddressBuilder {
        /// Build the shape, validating with the default validator.
        #[inline]
        pub fn build(self) -> Result<Address, _ValidationErrors> {
            _BuildWithSchema::build(self)
        }
        /// Build the shape using a custom validator.
        #[inline]
        pub fn build_with_validator(
            self,
            validator: impl _Validator,
        ) -> Result<Address, _ValidationErrors> {
            _BuildWithSchema::build_with_validator(self, validator)
        }
    }
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for AddressBuilder {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 2usize)?;
            ser.write_member_named(
                "street",
                &_ADDRESS_SCHEMA_MEMBER_STREET,
                &self.street,
            )?;
            ser.write_member_named("zip", &_ADDRESS_SCHEMA_MEMBER_ZIP, &self.zip)?;
            ser.end(schema)
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for Address {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&ADDRESS_SCHEMA, self), f)
        }
    }
};
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Address {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Address {
    #[inline]
    fn eq(&self, other: &Address) -> bool {
        self.street == other.street && self.zip == other.zip
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Address {
    #[inline]
    fn clone(&self) -> Address {
        Address {
            street: ::core::clone::Clone::clone(&self.street),
            zip: ::core::clone::Clone::clone(&self.zip),
        }
    }
}
#[smithy_schema(PERSON_SCHEMA)]
pub struct Person {
    #[smithy_schema(NAME)]
    pub name: String,
    #[smithy_schema(AGE)]
    pub age: i32,
    #[smithy_schema(HOME)]
    pub home: Address,
    #[smithy_schema(WORK)]
    pub work: Option<Address>,
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for Person {
        fn schema() -> &'static _Schema {
            &PERSON_SCHEMA
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for Person {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 4usize)?;
            ser.write_member_named("name", &_PERSON_SCHEMA_MEMBER_NAME, &self.name)?;
            ser.write_member_named("age", &_PERSON_SCHEMA_MEMBER_AGE, &self.age)?;
            ser.write_member_named("home", &_PERSON_SCHEMA_MEMBER_HOME, &self.home)?;
            ser.write_optional_member_named(
                "work",
                &_PERSON_SCHEMA_MEMBER_WORK,
                &self.work,
            )?;
            ser.end(schema)
        }
    }
};
#[automatically_derived]
pub struct PersonBuilder {
    name: smithy4rs_core::serde::Required<String>,
    age: smithy4rs_core::serde::Required<i32>,
    home: smithy4rs_core::serde::Required<
        smithy4rs_core::serde::MaybeBuilt<Address, AddressBuilder>,
    >,
    work: Option<smithy4rs_core::serde::MaybeBuilt<Address, AddressBuilder>>,
}
#[automatically_derived]
impl ::core::clone::Clone for PersonBuilder {
    #[inline]
    fn clone(&self) -> PersonBuilder {
        PersonBuilder {
            name: ::core::clone::Clone::clone(&self.name),
            age: ::core::clone::Clone::clone(&self.age),
            home: ::core::clone::Clone::clone(&self.home),
            work: ::core::clone::Clone::clone(&self.work),
        }
    }
}
#[automatically_derived]
impl PersonBuilder {
    pub fn new() -> Self {
        Self {
            name: smithy4rs_core::serde::Required::Unset,
            age: smithy4rs_core::serde::Required::Unset,
            home: smithy4rs_core::serde::Required::Unset,
            work: None,
        }
    }
    pub fn name<T: Into<String>>(mut self, value: T) -> Self {
        self.name = smithy4rs_core::serde::Required::Set(value.into());
        self
    }
    pub fn age<T: Into<i32>>(mut self, value: T) -> Self {
        self.age = smithy4rs_core::serde::Required::Set(value.into());
        self
    }
    pub fn home(mut self, value: Address) -> Self {
        self.home = smithy4rs_core::serde::Required::Set(
            smithy4rs_core::serde::MaybeBuilt::Struct(value),
        );
        self
    }
    pub fn home_builder(mut self, value: AddressBuilder) -> Self {
        self.home = smithy4rs_core::serde::Required::Set(
            smithy4rs_core::serde::MaybeBuilt::Builder(value),
        );
        self
    }
    pub fn work(mut self, value: Address) -> Self {
        self.work = Some(smithy4rs_core::serde::MaybeBuilt::Struct(value));
        self
    }
    pub fn work_builder(mut self, value: AddressBuilder) -> Self {
        self.work = Some(smithy4rs_core::serde::MaybeBuilt::Builder(value));
        self
    }
}
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::schema::StaticSchemaShape as _StaticSchemaShape;
    #[automatically_derived]
    impl _StaticSchemaShape for PersonBuilder {
        fn schema() -> &'static _Schema {
            &PERSON_SCHEMA
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::schema::Schema as _Schema;
    use _smithy4rs::serde::deserializers::Deserializer as _Deserializer;
    use _smithy4rs::serde::deserializers::DeserializeWithSchema as _DeserializeWithSchema;
    use _smithy4rs::serde::correction::ErrorCorrection as _ErrorCorrection;
    use _smithy4rs::serde::correction::ErrorCorrectionDefault as _ErrorCorrectionDefault;
    use _smithy4rs::serde::BuildWithSchema as _BuildWithSchema;
    use _smithy4rs::serde::deserializers::StructReader as _StructReader;
    #[automatically_derived]
    impl<'de> _DeserializeWithSchema<'de> for PersonBuilder {
        fn deserialize_with_schema<D>(
            schema: &_Schema,
            deserializer: D,
        ) -> Result<Self, D::Error>
        where
            D: _Deserializer<'de>,
        {
            let mut builder = PersonBuilder::new();
            let mut reader = deserializer.read_struct(schema)?;
            while let Some(member_schema) = reader.read_member(schema)? {
                if member_schema == *_PERSON_SCHEMA_MEMBER_NAME {
                    let value: String = reader.read_value(member_schema)?;
                    builder = builder.name(value);
                    continue;
                }
                if member_schema == *_PERSON_SCHEMA_MEMBER_AGE {
                    let value: i32 = reader.read_value(member_schema)?;
                    builder = builder.age(value);
                    continue;
                }
                if member_schema == *_PERSON_SCHEMA_MEMBER_HOME {
                    let value: AddressBuilder = reader.read_value(member_schema)?;
                    builder = builder.home_builder(value);
                    continue;
                }
                if member_schema == *_PERSON_SCHEMA_MEMBER_WORK {
                    let value: Option<AddressBuilder> = reader
                        .read_value(member_schema)?;
                    if let Some(v) = value {
                        builder = builder.work_builder(v);
                    }
                    continue;
                }
                reader.skip_value()?;
            }
            Ok(builder)
        }
    }
    use smithy4rs_core::serde::BuildableShape as _BuildableShape;
    use smithy4rs_core::serde::validation::ValidationErrors as _ValidationErrors;
    use smithy4rs_core::serde::validation::Validator as _Validator;
    #[automatically_derived]
    impl _BuildableShape for Person {
        type Builder = PersonBuilder;
    }
    #[automatically_derived]
    impl _ErrorCorrection for PersonBuilder {
        type Value = Person;
        fn correct(self) -> Self::Value {
            Person {
                name: self.name.get(),
                age: self.age.get(),
                home: self.home.get().correct(),
                work: self.work.correct(),
            }
        }
    }
    #[automatically_derived]
    impl _BuildWithSchema<Person> for PersonBuilder {
        fn new() -> Self {
            Self::new()
        }
    }
    #[automatically_derived]
    impl _ErrorCorrectionDefault for Person {
        fn default() -> Self {
            PersonBuilder::new().correct()
        }
    }
    #[automatically_derived]
    impl Person {
        /// Get a new builder for this shape.
        #[must_use]
        #[inline]
        pub fn builder() -> PersonBuilder {
            <Self as _BuildableShape>::builder()
        }
    }
    #[automatically_derived]
    impl PersonBuilder {
        /// Build the shape, validating with the default validator.
        #[inline]
        pub fn build(self) -> Result<Person, _ValidationErrors> {
            _BuildWithSchema::build(self)
        }
        /// Build the shape using a custom validator.
        #[inline]
        pub fn build_with_validator(
            self,
            validator: impl _Validator,
        ) -> Result<Person, _ValidationErrors> {
            _BuildWithSchema::build_with_validator(self, validator)
        }
    }
    use _smithy4rs::serde::serializers::Serializer as _Serializer;
    use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
    use _smithy4rs::serde::serializers::StructWriter as _StructWriter;
    #[automatically_derived]
    impl _SerializeWithSchema for PersonBuilder {
        fn serialize_with_schema<S: _Serializer>(
            &self,
            schema: &_Schema,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            let mut ser = serializer.write_struct(schema, 4usize)?;
            ser.write_member_named("name", &_PERSON_SCHEMA_MEMBER_NAME, &self.name)?;
            ser.write_member_named("age", &_PERSON_SCHEMA_MEMBER_AGE, &self.age)?;
            ser.write_member_named("home", &_PERSON_SCHEMA_MEMBER_HOME, &self.home)?;
            ser.write_optional_member_named(
                "work",
                &_PERSON_SCHEMA_MEMBER_WORK,
                &self.work,
            )?;
            ser.end(schema)
        }
    }
};
const _: () = {
    extern crate smithy4rs_core as _smithy4rs;
    use _smithy4rs::serde::debug::DebugWrapper as _DebugWrapper;
    #[automatically_derived]
    impl std::fmt::Debug for Person {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(&_DebugWrapper::new(&PERSON_SCHEMA, self), f)
        }
    }
};
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Person {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Person {
    #[inline]
    fn eq(&self, other: &Person) -> bool {
        self.age == other.age && self.name == other.name && self.home == other.home
            && self.work == other.work
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Person {
    #[inline]
    fn clone(&self) -> Person {
        Person {
            name: ::core::clone::Clone::clone(&self.name),
            age: ::core::clone::Clone::clone(&self.age),
            home: ::core::clone::Clone::clone(&self.home),
            work: ::core::clone::Clone::clone(&self.work),
        }
    }
}
