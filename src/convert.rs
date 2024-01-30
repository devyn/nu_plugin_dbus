use dbus::{Message, arg::{ArgType, RefArg}};
use nu_plugin::LabeledError;
use nu_protocol::{Value, Span, Record};

pub fn from_message(message: &Message) -> Result<Value, LabeledError> {
    let mut out = vec![];
    for refarg in message.iter_init() {
        if let Some(o) = from_refarg(&refarg) {
            out.push(o);
        }
    }
    Ok(Value::list(out, Span::unknown()))
}

pub fn from_refarg(refarg: &dyn RefArg) -> Option<Value> {
    Some(match refarg.arg_type() {
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
        // Floats
        ArgType::Double =>
            Value::float(refarg.as_f64().unwrap(), Span::unknown()),

        ArgType::Struct =>
            Value::list(
                refarg.as_iter().unwrap().map(from_refarg).flatten().collect(),
                Span::unknown()),

        ArgType::DictEntry => todo!(),
        ArgType::UInt64 => todo!(), // nushell only supports up to i64

        // End of iterator
        ArgType::Invalid => return None,
    })
}
