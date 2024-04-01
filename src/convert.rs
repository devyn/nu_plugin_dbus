use dbus::{
    arg::{
        messageitem::{MessageItem, MessageItemArray, MessageItemDict},
        ArgType, RefArg,
    },
    Message, Signature,
};
use nu_protocol::{LabeledError, Record, Span, Value};
use std::str::FromStr;

use crate::dbus_type::DbusType;

/// Get the arguments of a message as nushell Values
pub fn from_message(message: &Message, span: Span) -> Result<Vec<Value>, String> {
    let mut out = vec![];
    for refarg in message.iter_init() {
        out.push(from_refarg(&refarg, span)?);
    }
    Ok(out)
}

pub fn from_refarg(refarg: &dyn RefArg, span: Span) -> Result<Value, String> {
    Ok(match refarg.arg_type() {
        ArgType::Array => {
            if refarg.signature().starts_with("a{") {
                // This is a dictionary
                let mut record = Record::new();
                let mut iter = refarg.as_iter().unwrap();
                while let Some(key) = iter.next() {
                    if let Some(val) = iter.next() {
                        if let Some(key_str) = key.as_str() {
                            record.insert(key_str, from_refarg(val, span)?);
                        }
                    }
                }
                Value::record(record, span)
            } else if &*refarg.signature() == "ay" {
                // Byte array - better to return as binary
                let bytes = dbus::arg::cast::<Vec<u8>>(&refarg.box_clone())
                    .unwrap()
                    .to_owned();
                Value::binary(bytes, span)
            } else {
                // It's an array
                Value::list(
                    refarg
                        .as_iter()
                        .unwrap()
                        .flat_map(|v| from_refarg(v, span))
                        .collect(),
                    span,
                )
            }
        }
        ArgType::Variant => {
            let inner = refarg.as_iter().unwrap().next().unwrap();
            return from_refarg(inner, span);
        }
        ArgType::Boolean => Value::bool(refarg.as_i64().unwrap() != 0, span),

        // Strings
        ArgType::String | ArgType::ObjectPath | ArgType::Signature => {
            Value::string(refarg.as_str().unwrap(), span)
        }
        // Ints
        ArgType::Byte
        | ArgType::Int16
        | ArgType::UInt16
        | ArgType::Int32
        | ArgType::UInt32
        | ArgType::Int64
        | ArgType::UnixFd => Value::int(refarg.as_i64().unwrap(), span),

        // Nushell doesn't support u64, so present it as a string
        ArgType::UInt64 => Value::string(refarg.as_u64().unwrap().to_string(), span),

        // Floats
        ArgType::Double => Value::float(refarg.as_f64().unwrap(), span),

        ArgType::Struct => Value::list(
            refarg
                .as_iter()
                .unwrap()
                .flat_map(|v| from_refarg(v, span))
                .collect(),
            span,
        ),

        ArgType::DictEntry => {
            return Err("Encountered dictionary entry outside of dictionary".into())
        }
        ArgType::Invalid => return Err("Encountered invalid D-Bus value".into()),
    })
}

pub fn to_message_item(
    value: &Value,
    expected_type: Option<&DbusType>,
) -> Result<MessageItem, LabeledError> {
    // Report errors from conversion. Error must support Display
    macro_rules! try_convert {
        ($result_expr:expr) => {
            $result_expr.map_err(|err| {
                LabeledError::new(format!(
                    "Failed to convert value to the D-Bus `{:?}` type",
                    expected_type.unwrap()
                ))
                .with_label(err.to_string(), value.span())
            })?
        };
    }

    // Try to match values to expected types
    match (value, expected_type) {
        // Boolean
        (Value::Bool { val, .. }, Some(DbusType::Boolean)) => Ok(MessageItem::Bool(*val)),

        // Strings and specialized strings
        (Value::String { val, .. }, Some(DbusType::String)) => Ok(MessageItem::Str(val.to_owned())),
        (Value::String { val, .. }, Some(DbusType::ObjectPath)) => Ok(MessageItem::ObjectPath(
            try_convert!(dbus::strings::Path::new(val)),
        )),
        (Value::String { val, .. }, Some(DbusType::Signature)) => Ok(MessageItem::Signature(
            try_convert!(dbus::strings::Signature::new(val)),
        )),

        // Signed ints
        (Value::Int { val, .. }, Some(DbusType::Int64)) => Ok(MessageItem::Int64(*val)),
        (Value::Int { val, .. }, Some(DbusType::Int32)) => {
            Ok(MessageItem::Int32(try_convert!(i32::try_from(*val))))
        }
        (Value::Int { val, .. }, Some(DbusType::Int16)) => {
            Ok(MessageItem::Int16(try_convert!(i16::try_from(*val))))
        }

        // Unsigned ints
        (Value::Int { val, .. }, Some(DbusType::UInt64)) => {
            Ok(MessageItem::UInt64(try_convert!(u64::try_from(*val))))
        }
        (Value::Int { val, .. }, Some(DbusType::UInt32)) => {
            Ok(MessageItem::UInt32(try_convert!(u32::try_from(*val))))
        }
        (Value::Int { val, .. }, Some(DbusType::UInt16)) => {
            Ok(MessageItem::UInt16(try_convert!(u16::try_from(*val))))
        }
        (Value::Int { val, .. }, Some(DbusType::Byte)) => {
            Ok(MessageItem::Byte(try_convert!(u8::try_from(*val))))
        }

        // Ints from string
        (Value::String { val, .. }, Some(DbusType::Int64)) => {
            Ok(MessageItem::Int64(try_convert!(i64::from_str(&val[..]))))
        }
        (Value::String { val, .. }, Some(DbusType::Int32)) => {
            Ok(MessageItem::Int32(try_convert!(i32::from_str(&val[..]))))
        }
        (Value::String { val, .. }, Some(DbusType::Int16)) => {
            Ok(MessageItem::Int16(try_convert!(i16::from_str(&val[..]))))
        }
        (Value::String { val, .. }, Some(DbusType::UInt64)) => {
            Ok(MessageItem::UInt64(try_convert!(u64::from_str(&val[..]))))
        }
        (Value::String { val, .. }, Some(DbusType::UInt32)) => {
            Ok(MessageItem::UInt32(try_convert!(u32::from_str(&val[..]))))
        }
        (Value::String { val, .. }, Some(DbusType::UInt16)) => {
            Ok(MessageItem::UInt16(try_convert!(u16::from_str(&val[..]))))
        }
        (Value::String { val, .. }, Some(DbusType::Byte)) => {
            Ok(MessageItem::Byte(try_convert!(u8::from_str(&val[..]))))
        }

        // Float
        (Value::Float { val, .. }, Some(DbusType::Double)) => Ok(MessageItem::Double(*val)),
        (Value::String { val, .. }, Some(DbusType::Double)) => {
            Ok(MessageItem::Double(try_convert!(f64::from_str(&val[..]))))
        }

        // Binary
        (Value::Binary { val, .. }, Some(r#type @ DbusType::Array(content_type)))
            if matches!(**content_type, DbusType::Byte) =>
        {
            // FIXME: this is likely pretty inefficient for a bunch of bytes
            let sig = Signature::from(r#type.stringify());
            let items = val
                .iter()
                .cloned()
                .map(MessageItem::Byte)
                .collect::<Vec<_>>();
            Ok(MessageItem::Array(
                MessageItemArray::new(items, sig).unwrap(),
            ))
        }

        // List/array
        (Value::List { vals, .. }, Some(r#type @ DbusType::Array(content_type))) => {
            let sig = Signature::from(r#type.stringify());
            let items = vals
                .iter()
                .map(|content| to_message_item(content, Some(content_type)))
                .collect::<Result<Vec<MessageItem>, _>>()?;
            Ok(MessageItem::Array(
                MessageItemArray::new(items, sig).unwrap(),
            ))
        }

        // Struct
        (Value::List { vals, .. }, Some(DbusType::Struct(types))) => {
            if vals.len() != types.len() {
                return Err(LabeledError::new(format!(
                    "expected struct with {} element(s) ({:?})",
                    types.len(),
                    types
                ))
                .with_label(
                    format!("this list has {} element(s) instead", vals.len()),
                    value.span(),
                ));
            }
            let items = vals
                .iter()
                .zip(types)
                .map(|(content, r#type)| to_message_item(content, Some(r#type)))
                .collect::<Result<Vec<MessageItem>, _>>()?;
            Ok(MessageItem::Struct(items))
        }

        // Record/dict
        (Value::Record { val, .. }, Some(DbusType::Array(content_type)))
            if matches!(**content_type, DbusType::DictEntry(_, _)) =>
        {
            if let DbusType::DictEntry(ref key_type, ref val_type) = **content_type {
                let key_sig = Signature::from(key_type.stringify());
                let val_sig = Signature::from(val_type.stringify());
                let pairs = val
                    .iter()
                    .map(|(key, val)| {
                        let key_as_value = Value::string(key, value.span());
                        let key_message_item = to_message_item(&key_as_value, Some(key_type))?;
                        let val_message_item = to_message_item(val, Some(val_type))?;
                        Ok((key_message_item, val_message_item))
                    })
                    .collect::<Result<Vec<_>, LabeledError>>()?;
                Ok(MessageItem::Dict(
                    MessageItemDict::new(pairs, key_sig, val_sig).unwrap(),
                ))
            } else {
                unreachable!()
            }
        }

        // Variant - use automatic type
        (other_value, Some(DbusType::Variant)) => Ok(MessageItem::Variant(Box::new(
            to_message_item(other_value, None)?,
        ))),

        // Value not compatible with expected type
        (other_value, Some(expectation)) => Err(LabeledError::new(format!(
            "`{}` can not be converted to the D-Bus `{:?}` type",
            other_value.get_type(),
            expectation
        ))
        .with_label(
            format!("expected a `{:?}` here", expectation),
            other_value.span(),
        )),

        // Automatic types (with no type expectation)
        (Value::String { .. }, None) => to_message_item(value, Some(&DbusType::String)),
        (Value::Int { .. }, None) => to_message_item(value, Some(&DbusType::Int64)),
        (Value::Float { .. }, None) => to_message_item(value, Some(&DbusType::Double)),
        (Value::Bool { .. }, None) => to_message_item(value, Some(&DbusType::Boolean)),
        (Value::List { .. }, None) => {
            to_message_item(value, Some(&DbusType::Array(DbusType::Variant.into())))
        }
        (Value::Record { .. }, None) => to_message_item(
            value,
            Some(&DbusType::Array(
                DbusType::DictEntry(DbusType::String.into(), DbusType::Variant.into()).into(),
            )),
        ),

        // No expected type, but can't handle this type
        _ => Err(LabeledError::new(format!(
            "can not use values of type `{}` in D-Bus calls",
            value.get_type()
        ))
        .with_label("use a supported type here instead", value.span())),
    }
}
