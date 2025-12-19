//
// use std::ops::Deref;
// use smithy4rs_core::smithy;
//
// smithy!("com.test#MyEnum": {
//     enum MY_ENUM {
//         A = "stuff",
//         B = "things"
//     }
// });
//
//
//
// const _: () = {
//     extern crate smithy4rs_core as _smithy4rs;
//     use _smithy4rs::schema::SchemaRef as _SchemaRef;
//     use _smithy4rs::serde::serializers::Serializer as _Serializer;
//     use _smithy4rs::serde::serializers::SerializeWithSchema as _SerializeWithSchema;
//     use _smithy4rs::serde::serializers::StructSerializer as _StructSerializer;
//     #[automatically_derived]
//     impl _SerializeWithSchema for Test {
//         fn serialize_with_schema<S: _Serializer>(
//             &self,
//             schema: &_SchemaRef,
//             serializer: S,
//         ) -> Result<S::Ok, S::Error> {
//             serializer.write_string(SCHEMA, self.value())
//         }
//     }
// };
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     fn check_enum_deref() {
//         let check = Test(TestInner::A);
//
//     }
// }
