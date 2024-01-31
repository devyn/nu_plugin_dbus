use dbus::{Message, arg::{ArgType, RefArg}};
use nu_protocol::{Value, Span, Record};

pub fn from_message(message: &Message) -> Result<Value, String> {
    let mut out = vec![];
    for refarg in message.iter_init() {
        out.push(from_refarg(&refarg)?);
    }
    Ok(Value::list(out, Span::unknown()))
}

pub fn from_refarg(refarg: &dyn RefArg) -> Result<Value, String> {
    Ok(match refarg.arg_type() {
        ArgType::Array => {
            if refarg.signature().starts_with("a{") {
                // This is a dictionary
                let mut record = Record::new();
                for entry in refarg.as_iter().unwrap() {
                    let mut entry_iter = entry.as_iter().unwrap();
                    let key = entry_iter.next().unwrap();
                    let val = entry_iter.next().unwrap();
                    if let Some(key) = key.as_str() {
                        record.insert(key, from_refarg(val)?);
                    }
                }
                Value::record(record, Span::unknown())
            } else if &*refarg.signature() == "ay" {
                // Byte array - better to return as binary
                let bytes = dbus::arg::cast::<Vec<u8>>(&refarg.box_clone()).unwrap().to_owned();
                Value::binary(bytes, Span::unknown())
            } else {
                // It's an array
                Value::list(
                    refarg.as_iter().unwrap().map(from_refarg).flatten().collect(),
                    Span::unknown())
            }
        },
        ArgType::Variant => {
            let inner = refarg.as_iter().unwrap().nth(0).unwrap();
            return from_refarg(inner);
        },
        ArgType::Boolean =>
            Value::bool(refarg.as_i64().unwrap() != 0, Span::unknown()),

        // Strings
        ArgType::String | ArgType::ObjectPath | ArgType::Signature =>
            Value::string(refarg.as_str().unwrap(), Span::unknown()),
        // Ints
        ArgType::Byte | ArgType::Int16 | ArgType::UInt16 | ArgType::Int32 |
        ArgType::UInt32 | ArgType::Int64 | ArgType::UnixFd =>
            Value::int(refarg.as_i64().unwrap(), Span::unknown()),

        // Nushell doesn't support u64, so present it as a string
        ArgType::UInt64 => Value::string(refarg.as_u64().unwrap().to_string(), Span::unknown()),

        // Floats
        ArgType::Double =>
            Value::float(refarg.as_f64().unwrap(), Span::unknown()),

        ArgType::Struct =>
            Value::list(
                refarg.as_iter().unwrap().map(from_refarg).flatten().collect(),
                Span::unknown()),

        ArgType::DictEntry =>
            return Err("Encountered dictionary entry outside of dictionary".into()),
        ArgType::Invalid =>
            return Err("Encountered invalid D-Bus value".into()),
    })
}
