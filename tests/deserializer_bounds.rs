//! Test which deserializer types satisfy our OwnedDeserializer bounds

use serde::de::Deserializer;

fn needs_ref_deserializer<'de, D>()
where
    for<'a> &'a mut D: Deserializer<'de>,
{
}

#[test]
fn test_serde_json_deserializer_bounds() {
    // serde_json::Deserializer implements Deserializer for &mut Self
    needs_ref_deserializer::<serde_json::Deserializer<serde_json::de::StrRead<'static>>>();
}
