/// Representation of fully specified D-Bus types
///
/// [dbus::arg::ArgType] does not sufficiently specify the types inside of a container type,
/// and also doesn't provide any parse/dump capability
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DbusType {
    Byte,
    Boolean,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Double,
    String,
    ObjectPath,
    Signature,
    Array(Box<DbusType>),
    Struct(Vec<DbusType>),
    Variant,
    DictEntry(Box<DbusType>, Box<DbusType>),
}

impl DbusType {
    /// Parse one type from a D-Bus signature, and return the remainder
    pub fn parse(input: &str) -> Result<(DbusType, &str), String> {
        use self::DbusType::*;

        if input.is_empty() {
            return Err("unexpected end of D-Bus type string".into());
        }

        match input.chars().nth(0).unwrap() {
            'y' => Ok((Byte, &input[1..])),
            'b' => Ok((Boolean, &input[1..])),
            'n' => Ok((Int16, &input[1..])),
            'q' => Ok((UInt16, &input[1..])),
            'i' => Ok((Int32, &input[1..])),
            'u' => Ok((UInt32, &input[1..])),
            'x' => Ok((Int64, &input[1..])),
            't' => Ok((UInt64, &input[1..])),
            'd' => Ok((Double, &input[1..])),
            's' => Ok((String, &input[1..])),
            'o' => Ok((ObjectPath, &input[1..])),
            'g' => Ok((Signature, &input[1..])),
            'a' => {
                // The next type is the content type of the array
                let (content_type, remainder) = Self::parse(&input[1..])?;
                Ok((Array(content_type.into()), remainder))
            },
            '(' => {
                // Parse the struct content until we get to the end ) char
                let mut remainder = &input[1..];
                let mut types = vec![];
                loop {
                    if remainder.is_empty() {
                        break Err("unexpected end of D-Bus type string \
                            before end of array".into());
                    } else if remainder.starts_with(')') {
                        break Ok((DbusType::Struct(types), &remainder[1..]));
                    } else {
                        let (r#type, new_remainder) = Self::parse(remainder)?;
                        types.push(r#type);
                        remainder = new_remainder;
                    }
                }
            },
            'v' => Ok((Variant, &input[1..])),
            '{' => {
                // Expect two types
                let (key_type, key_remainder) = Self::parse(&input[1..])?;
                let (val_type, val_remainder) = Self::parse(key_remainder)?;
                // Must end with }
                if val_remainder.starts_with('}') {
                    Ok((DbusType::DictEntry(key_type.into(), val_type.into()), &val_remainder[1..]))
                } else {
                    Err(format!("expected `}}` char to end dictionary in D-Bus type \
                        but remainder is {:?}", val_remainder))
                }
            },
            other => Err(format!("unexpected char {other:?} in D-Bus type representation"))
        }
    }

    /// Parse multiple types from a D-Bus signature
    pub fn parse_all(mut input: &str) -> Result<Vec<DbusType>, String> {
        let mut out = vec![];
        while !input.is_empty() {
            let (parsed, remainder) = Self::parse(input)?;
            out.push(parsed);
            input = remainder;
        }
        Ok(out)
    }

    /// Convert the D-Bus type into a string suitable for the wire format
    pub fn stringify(&self) -> String {
        use self::DbusType::*;

        match self {
            Byte       => 'y'.into(),
            Boolean    => 'b'.into(),
            Int16      => 'n'.into(),
            UInt16     => 'q'.into(),
            Int32      => 'i'.into(),
            UInt32     => 'u'.into(),
            Int64      => 'x'.into(),
            UInt64     => 't'.into(),
            Double     => 'd'.into(),
            String     => 's'.into(),
            ObjectPath => 'o'.into(),
            Signature  => 'g'.into(),

            // a<type>
            Array(content) => format!("a{}", content.stringify()),

            // (<type1><type2><typeN..>)
            Struct(types) => std::iter::once("(".to_owned())
                .chain(types.iter().map(|t| t.stringify()))
                .chain(std::iter::once(")".to_owned()))
                .collect(),

            Variant => 'v'.into(),

            // {<key><val>}
            DictEntry(key, val) => format!("{{{}{}}}", key.stringify(), val.stringify()),
        }
    }
}

#[cfg(test)]
macro_rules! should_parse_to {
    ($str:expr, $result:expr) => (
        assert_eq!(DbusType::parse($str), Ok(($result, "")))
    )
}

#[test]
fn test_parse_simple_types() {
    use self::DbusType::*;
    should_parse_to!("y", Byte);
    should_parse_to!("b", Boolean);
    should_parse_to!("n", Int16);
    should_parse_to!("q", UInt16);
    should_parse_to!("i", Int32);
    should_parse_to!("u", UInt32);
    should_parse_to!("x", Int64);
    should_parse_to!("t", UInt64);
    should_parse_to!("d", Double);
    should_parse_to!("s", String);
    should_parse_to!("o", ObjectPath);
    should_parse_to!("g", Signature);
    should_parse_to!("v", Variant);
}

#[test]
fn test_parse_simple_type_remainder() -> Result<(), String> {
    let (_, remainder) = DbusType::parse("gyn")?;
    assert_eq!(remainder, "yn");
    Ok(())
}

#[test]
fn test_parse_simple_invalid() {
    assert!(DbusType::parse("*").is_err());
}

#[test]
fn test_parse_simple_array() {
    use self::DbusType::*;
    should_parse_to!("ay", Array(Byte.into()));
}

#[test]
fn test_parse_nested_array() {
    use self::DbusType::*;
    should_parse_to!("aai", Array(Array(Int32.into()).into()));
}

#[test]
fn test_parse_array_remainder() -> Result<(), String> {
    let (_, remainder) = DbusType::parse("ay(oi)")?;
    assert_eq!(remainder, "(oi)");
    Ok(())
}

#[test]
fn test_parse_array_unclosed() {
    assert!(DbusType::parse("a").is_err());
}

#[test]
fn test_parse_simple_struct() {
    use self::DbusType::*;
    should_parse_to!("()", Struct(vec![]));
    should_parse_to!("(y)", Struct(vec![Byte]));
    should_parse_to!("(sy)", Struct(vec![String, Byte]));
    should_parse_to!("(xto)", Struct(vec![Int64, UInt64, ObjectPath]));
}

#[test]
fn test_parse_nested_struct() {
    use self::DbusType::*;
    should_parse_to!("((xx))", Struct(vec![Struct(vec![Int64, Int64])]));
    should_parse_to!("(y(xx))", Struct(vec![Byte, Struct(vec![Int64, Int64])]));
    should_parse_to!("(y(ss)o)", Struct(vec![Byte, Struct(vec![String, String]), ObjectPath]));
    should_parse_to!("((yy)s)", Struct(vec![Struct(vec![Byte, Byte]), String]));
}

#[test]
fn test_parse_struct_remainder() -> Result<(), String> {
    let (_, remainder) = DbusType::parse("(oi)ay")?;
    assert_eq!(remainder, "ay");
    Ok(())
}

#[test]
fn test_parse_struct_unclosed() {
    assert!(DbusType::parse("(ss").is_err());
}

#[test]
fn test_parse_dict_entry() {
    use self::DbusType::*;
    should_parse_to!("{ss}", DictEntry(String.into(), String.into()));
    should_parse_to!("{s(bd)}", DictEntry(String.into(), Struct(vec![Boolean, Double]).into()));
}

#[test]
fn test_parse_array_dict() {
    use self::DbusType::*;
    should_parse_to!("a{sd}", Array(DictEntry(String.into(), Double.into()).into()));
}

#[test]
fn test_parse_dict_entry_remainder() -> Result<(), String> {
    let (_, remainder) = DbusType::parse("{sd}{sai}")?;
    assert_eq!(remainder, "{sai}");
    Ok(())
}

#[test]
fn test_parse_dict_entry_unclosed() {
    assert!(DbusType::parse("{ss").is_err());
}

#[test]
fn test_parse_all() {
    use self::DbusType::*;
    assert_eq!(DbusType::parse_all(""), Ok(vec![]));
    assert_eq!(
        DbusType::parse_all("s"),
        Ok(vec![
            String,
        ])
    );
    assert_eq!(
        DbusType::parse_all("isbb"),
        Ok(vec![
            Int32,
            String,
            Boolean,
            Boolean,
        ])
    );
    assert_eq!(
        DbusType::parse_all("ia{s(bi)}s"),
        Ok(vec![
            Int32,
            Array(DictEntry(String.into(), Struct(vec![Boolean, Int32]).into()).into()),
            String,
        ])
    );
}

#[cfg(test)]
macro_rules! should_stringify_to {
    ($type:expr, $result:expr) => (
        assert_eq!(DbusType::stringify(&$type), $result)
    )
}

#[test]
fn test_stringify_simple_types() {
    use self::DbusType::*;
    should_stringify_to!(Byte,       "y");
    should_stringify_to!(Boolean,    "b");
    should_stringify_to!(Int16,      "n");
    should_stringify_to!(UInt16,     "q");
    should_stringify_to!(Int32,      "i");
    should_stringify_to!(UInt32,     "u");
    should_stringify_to!(Int64,      "x");
    should_stringify_to!(UInt64,     "t");
    should_stringify_to!(Double,     "d");
    should_stringify_to!(String,     "s");
    should_stringify_to!(ObjectPath, "o");
    should_stringify_to!(Signature,  "g");
    should_stringify_to!(Variant,    "v");
}

#[test]
fn test_stringify_array() {
    use self::DbusType::*;
    should_stringify_to!(Array(Variant.into()), "av");
    should_stringify_to!(Array(Array(String.into()).into()), "aas");
}

#[test]
fn test_stringify_struct() {
    use self::DbusType::*;
    should_stringify_to!(Struct(vec![]), "()");
    should_stringify_to!(Struct(vec![Int32]), "(i)");
    should_stringify_to!(Struct(vec![Int32, String]), "(is)");
    should_stringify_to!(Struct(vec![Byte, Int32, String]), "(yis)");
    should_stringify_to!(Struct(vec![Byte, Struct(vec![String, Boolean]), String]), "(y(sb)s)");
}

#[test]
fn test_stringify_dict_entry() {
    use self::DbusType::*;
    should_stringify_to!(DictEntry(String.into(), Int32.into()), "{si}");
    should_stringify_to!(DictEntry(Int32.into(), String.into()), "{is}");
}

#[test]
fn test_stringify_nested() {
    use self::DbusType::*;
    should_stringify_to!(Array(DictEntry(String.into(), Int32.into()).into()), "a{si}");
    should_stringify_to!(
        Array(
            DictEntry(
                String.into(),
                Struct(vec![
                    Byte,
                    Array(Int32.into())
                ]).into()
            ).into()
        ),
        "a{s(yai)}"
    );
}
