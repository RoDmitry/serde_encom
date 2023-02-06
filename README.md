# EnCom serialization format

[![Crate](https://img.shields.io/crates/v/serde_encom.svg)](https://crates.io/crates/serde_encom)
[![API](https://docs.rs/serde_encom/badge.svg)](https://docs.rs/serde_encom)

## Made to replace old and loved JSON

Encoding Compact, Encoded for Communication

The main goal was to remove extra characters, allow binary data, and speed up parsing of strings.

There is no need to escape characters in strings. And it's capable to work with binary data as is.

As long as you use standard serialization, and don't use `serde_bytes`, then the serialization output will be a valid string (utf8, not binary).

Also if you use string or byte slice in your resulting structure, it will not be cloned (Zero-copy).

## Examples
### EnCom
```
glossary{
    GlossDiv{
        GlossList{
            GlossEntry{
                Abbrev:13=ISO 8879:1986
                Acronym:4=SGML
                GlossDef{
                    GlossSeeAlso{
                        3=GML
                        3=XML
                    }
                    para:72=A meta-markup language, used to create markup languages such as DocBook.
                }
                GlossSee:6=markup
                GlossTerm:36=Standard Generalized Markup Language
                ID:4=SGML
                SortAs:4=SGML
            }
        }
        title:1=S
    }
    title:16=example glossary
}
```

### JSON
```
{
    "glossary": {
        "GlossDiv": {
            "GlossList": {
                "GlossEntry": {
                    "Abbrev": "ISO 8879:1986",
                    "Acronym": "SGML",
                    "GlossDef": {
                        "GlossSeeAlso": [
                            "GML",
                            "XML"
                        ],
                        "para": "A meta-markup language, used to create markup languages such as DocBook."
                    },
                    "GlossSee": "markup",
                    "GlossTerm": "Standard Generalized Markup Language",
                    "ID": "SGML",
                    "SortAs": "SGML"
                }
            },
            "title": "S"
        },
        "title": "example glossary"
    }
}
```

You can use `encom_from_json!()` macro to convert your own JSON and test it. Resulting EnCom will be sorted alphabetically.

## Format specification
Separator - space (actually it might be any character <= b' ', or less then 0x21)

Structure and array open with `{` and close with `}`

First structure/array don't have them, and if you don't specify the structure, and it's not a map (structure), then it will always be an array

If you have an Array of Enums, then Enums inside don't have `{` and `}`

Space after `}` is not necessary (todo: fix extra space)

Strings and binary data have length stated (in bytes), like `8=sometext` for strings and `8~somedata` for bytes

## Todo:
- [ ] Fix Value deserializer
- [ ] Pretty fix spacing
- [ ] Skip Option::None by default, and only if `#[serde(serialize_with = "path")] ` is passed, then serialize None
- [ ] Fix Stream deserializer
- [ ] Fix File deserializer

### Thanks [serde_json](https://github.com/serde-rs/json) and it's [contributors](https://github.com/serde-rs/json/graphs/contributors) for the base code that was used it this project
