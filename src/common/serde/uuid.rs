use std::fmt::Formatter;

use serde::de::{Error, Visitor};
use serde::{Deserializer, Serializer};
use uuid::Uuid;

pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u128(uuid.as_u128())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    struct UuidVisitor;

    impl Visitor<'_> for UuidVisitor {
        type Value = Uuid;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("Bad uuid")
        }

        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Uuid::from_u128(v))
        }
    }

    deserializer.deserialize_u128(UuidVisitor)
}
