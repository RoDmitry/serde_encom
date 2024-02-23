# Serde EnCom

[![Crate](https://img.shields.io/crates/v/serde_encom.svg)](https://crates.io/crates/serde_encom)
[![API](https://docs.rs/serde_encom/badge.svg)](https://docs.rs/serde_encom)

### Made to replace old and loved JSON

[EnCom specification](https://github.com/RoDmitry/EnCom)

As long as you use standard serialization, and don't use `serde_bytes`, then the serialization output will be a valid string (utf8, not binary).

If you are using string `&str` or byte `&[u8]` slice in your resulting structure, it will not be copied (Zero-copy).

You can try `encom_from_json!()` macro to convert your own JSON and test it. Resulting EnCom will be sorted alphabetically.

## Todo:
- [ ] Fix Stream deserializer
- [ ] Fix File deserializer
- [ ] Fix extra space after `}` in serializer
- [ ] Fix parse exponent in deserializer
- [ ] Change errors

Maybe:
Skip Option::None by default, and only if `#[serde(serialize_with = "path")]` is passed, then serialize None

### Thanks [serde_json](https://github.com/serde-rs/json) and it's [contributors](https://github.com/serde-rs/json/graphs/contributors) for the base code that was used it this project
