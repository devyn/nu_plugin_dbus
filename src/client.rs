use dbus::{
    arg::messageitem::MessageItem,
    channel::{BusType, Channel},
    Message,
};
use nu_plugin::LabeledError;
use nu_protocol::{Spanned, Value};

use crate::{
    config::{DbusBusChoice, DbusClientConfig},
    convert::to_message_item,
    dbus_type::DbusType,
    introspection::Node,
    pattern::Pattern,
};

/// Executes D-Bus actions on a connection, handling nushell types
pub struct DbusClient {
    config: DbusClientConfig,
    conn: Channel,
}

// Convenience macros for error handling
macro_rules! validate_with {
    ($type:ty, $spanned:expr) => {
        <$type>::new(&$spanned.item).map_err(|msg| LabeledError {
            label: msg,
            msg: "this argument is incorrect".into(),
            span: Some($spanned.span),
        })
    };
}

impl DbusClient {
    pub fn new(config: DbusClientConfig) -> Result<DbusClient, LabeledError> {
        // Try to connect to the correct D-Bus destination, as specified in the config
        let channel = match &config.bus_choice.item {
            DbusBusChoice::Session => Channel::get_private(BusType::Session),
            DbusBusChoice::System => Channel::get_private(BusType::System),
            DbusBusChoice::Started => Channel::get_private(BusType::Starter),
            DbusBusChoice::Peer(address) => Channel::open_private(address),
            DbusBusChoice::Bus(address) => Channel::open_private(address).and_then(|mut ch| {
                ch.register()?;
                Ok(ch)
            }),
        }
        .map_err(|err| LabeledError {
            label: err.to_string(),
            msg: "while connecting to D-Bus as specified here".into(),
            span: Some(config.bus_choice.span),
        })?;
        Ok(DbusClient {
            config,
            conn: channel,
        })
    }

    fn error(&self, err: impl std::fmt::Display, msg: impl std::fmt::Display) -> LabeledError {
        LabeledError {
            label: err.to_string(),
            msg: msg.to_string(),
            span: Some(self.config.span),
        }
    }

    /// Introspect a D-Bus object
    pub fn introspect(
        &self,
        dest: &Spanned<String>,
        object: &Spanned<String>,
    ) -> Result<Node, LabeledError> {
        let context = "while introspecting a D-Bus method";
        let valid_dest = validate_with!(dbus::strings::BusName, dest)?;
        let valid_object = validate_with!(dbus::strings::Path, object)?;

        // Create the introspection method call
        let message = Message::new_method_call(
            valid_dest,
            valid_object,
            "org.freedesktop.DBus.Introspectable",
            "Introspect",
        )
        .map_err(|err| self.error(err, context))?;

        // Send and get the response
        let resp = self
            .conn
            .send_with_reply_and_block(message, self.config.timeout.item)
            .map_err(|err| self.error(err, context))?;

        // Parse it to a Node
        let xml: &str = resp
            .get1()
            .ok_or_else(|| self.error("Introspect method returned the wrong type", context))?;

        Node::from_xml(xml).map_err(|err| self.error(err, context))
    }

    /// Try to use introspection to get the signature of a method
    fn get_method_signature_by_introspection(
        &self,
        dest: &Spanned<String>,
        object: &Spanned<String>,
        interface: &Spanned<String>,
        method: &Spanned<String>,
    ) -> Result<Vec<DbusType>, LabeledError> {
        let node = self.introspect(dest, object)?;

        if let Some(sig) = node.get_method_args_signature(&interface.item, &method.item) {
            DbusType::parse_all(&sig).map_err(|err| LabeledError {
                label: format!(
                    "while getting interface {:?} method {:?} signature: {}",
                    interface.item, method.item, err
                ),
                msg: "try running with --no-introspect or --signature".into(),
                span: Some(self.config.span),
            })
        } else {
            Err(LabeledError {
                label: format!("Method {:?} not found on {:?}", method.item, interface.item),
                msg: "check that this method/interface is correct".into(),
                span: Some(method.span),
            })
        }
    }

    /// Try to use introspection to get the signature of a property
    fn get_property_signature_by_introspection(
        &self,
        dest: &Spanned<String>,
        object: &Spanned<String>,
        interface: &Spanned<String>,
        property: &Spanned<String>,
    ) -> Result<Vec<DbusType>, LabeledError> {
        let node = self.introspect(dest, object)?;

        if let Some(sig) = node.get_property_signature(&interface.item, &property.item) {
            DbusType::parse_all(sig).map_err(|err| LabeledError {
                label: format!(
                    "while getting interface {:?} property {:?} signature: {}",
                    interface.item, property.item, err
                ),
                msg: "try running with --no-introspect or --signature".into(),
                span: Some(self.config.span),
            })
        } else {
            Err(LabeledError {
                label: format!(
                    "Property {:?} not found on {:?}",
                    property.item, interface.item
                ),
                msg: "check that this property/interface is correct".into(),
                span: Some(property.span),
            })
        }
    }

    /// Call a D-Bus method and wait for the response
    pub fn call(
        &self,
        dest: &Spanned<String>,
        object: &Spanned<String>,
        interface: &Spanned<String>,
        method: &Spanned<String>,
        signature: Option<&Spanned<String>>,
        args: &[Value],
    ) -> Result<Vec<Value>, LabeledError> {
        let context = "while calling a D-Bus method";

        // Validate inputs before sending to the dbus lib so we don't panic
        let valid_dest = validate_with!(dbus::strings::BusName, dest)?;
        let valid_object = validate_with!(dbus::strings::Path, object)?;
        let valid_interface = validate_with!(dbus::strings::Interface, interface)?;
        let valid_method = validate_with!(dbus::strings::Member, method)?;

        // Parse the signature
        let mut valid_signature = signature
            .map(|s| {
                DbusType::parse_all(&s.item).map_err(|err| LabeledError {
                    label: err,
                    msg: "in signature specified here".into(),
                    span: Some(s.span),
                })
            })
            .transpose()?;

        // If not provided, try introspection (unless disabled)
        if valid_signature.is_none() && self.config.introspect {
            match self.get_method_signature_by_introspection(dest, object, interface, method) {
                Ok(sig) => {
                    valid_signature = Some(sig);
                }
                Err(err) => {
                    eprintln!(
                        "Warning: D-Bus introspection failed on {:?}. \
                        Use `--no-introspect` or pass `--signature` to silence this warning. \
                        Cause: {}",
                        object.item, err.label
                    );
                }
            }
        }

        if let Some(sig) = &valid_signature {
            if sig.len() != args.len() {
                self.error(
                    format!("expected {} arguments, got {}", sig.len(), args.len()),
                    context,
                );
            }
        }

        // Construct the method call message
        let mut message =
            Message::new_method_call(valid_dest, valid_object, valid_interface, valid_method)
                .map_err(|err| self.error(err, context))?;

        // Convert the args to message items
        let sigs_iter = valid_signature
            .iter()
            .flatten()
            .map(Some)
            .chain(std::iter::repeat(None));
        for (val, sig) in args.iter().zip(sigs_iter) {
            message = message.append1(to_message_item(val, sig)?);
        }

        // Send it on the channel and get the response
        let resp = self
            .conn
            .send_with_reply_and_block(message, self.config.timeout.item)
            .map_err(|err| self.error(err, context))?;

        crate::convert::from_message(&resp, self.config.span)
            .map_err(|err| self.error(err, context))
    }

    /// Get a D-Bus property from the given object
    pub fn get(
        &self,
        dest: &Spanned<String>,
        object: &Spanned<String>,
        interface: &Spanned<String>,
        property: &Spanned<String>,
    ) -> Result<Value, LabeledError> {
        let interface_val = Value::string(&interface.item, interface.span);
        let property_val = Value::string(&property.item, property.span);

        self.call(
            dest,
            object,
            &Spanned {
                item: "org.freedesktop.DBus.Properties".into(),
                span: self.config.span,
            },
            &Spanned {
                item: "Get".into(),
                span: self.config.span,
            },
            Some(&Spanned {
                item: "ss".into(),
                span: self.config.span,
            }),
            &[interface_val, property_val],
        )
        .map(|val| val.into_iter().nth(0).unwrap_or_default())
    }

    /// Get all D-Bus properties from the given object
    pub fn get_all(
        &self,
        dest: &Spanned<String>,
        object: &Spanned<String>,
        interface: &Spanned<String>,
    ) -> Result<Value, LabeledError> {
        let interface_val = Value::string(&interface.item, interface.span);

        self.call(
            dest,
            object,
            &Spanned {
                item: "org.freedesktop.DBus.Properties".into(),
                span: self.config.span,
            },
            &Spanned {
                item: "GetAll".into(),
                span: self.config.span,
            },
            Some(&Spanned {
                item: "s".into(),
                span: self.config.span,
            }),
            &[interface_val],
        )
        .map(|val| val.into_iter().nth(0).unwrap_or_default())
    }

    /// Set a D-Bus property on the given object
    pub fn set(
        &self,
        dest: &Spanned<String>,
        object: &Spanned<String>,
        interface: &Spanned<String>,
        property: &Spanned<String>,
        signature: Option<&Spanned<String>>,
        value: &Value,
    ) -> Result<(), LabeledError> {
        let context = "while setting a D-Bus property";

        // Validate inputs before sending to the dbus lib so we don't panic
        let valid_dest = validate_with!(dbus::strings::BusName, dest)?;
        let valid_object = validate_with!(dbus::strings::Path, object)?;

        // Parse the signature
        let mut valid_signature = signature
            .map(|s| {
                DbusType::parse_all(&s.item).map_err(|err| LabeledError {
                    label: err,
                    msg: "in signature specified here".into(),
                    span: Some(s.span),
                })
            })
            .transpose()?;

        // If not provided, try introspection (unless disabled)
        if valid_signature.is_none() && self.config.introspect {
            match self.get_property_signature_by_introspection(dest, object, interface, property) {
                Ok(sig) => {
                    valid_signature = Some(sig);
                }
                Err(err) => {
                    eprintln!(
                        "Warning: D-Bus introspection failed on {:?}. \
                        Use `--no-introspect` or pass `--signature` to silence this warning. \
                        Cause: {}",
                        object.item, err.label
                    );
                }
            }
        }

        if let Some(sig) = &valid_signature {
            if sig.len() != 1 {
                self.error(
                    format!(
                        "expected single object signature, but there are {}",
                        sig.len()
                    ),
                    context,
                );
            }
        }

        // Construct the method call message
        let message = Message::new_method_call(
            valid_dest,
            valid_object,
            "org.freedesktop.DBus.Properties",
            "Set",
        )
        .map_err(|err| self.error(err, context))?
        .append2(&interface.item, &property.item)
        .append1(
            // Box it in a variant as required for property setting
            MessageItem::Variant(Box::new(to_message_item(
                value,
                valid_signature.as_ref().map(|s| &s[0]),
            )?)),
        );

        // Send it on the channel and get the response
        self.conn
            .send_with_reply_and_block(message, self.config.timeout.item)
            .map_err(|err| self.error(err, context))?;

        Ok(())
    }

    pub fn list(&self, pattern: Option<&Pattern>) -> Result<Vec<String>, LabeledError> {
        let context = "while listing D-Bus connection names";

        let message = Message::new_method_call(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "ListNames",
        )
        .map_err(|err| self.error(err, context))?;

        self.conn
            .send_with_reply_and_block(message, self.config.timeout.item)
            .map_err(|err| self.error(err, context))
            .and_then(|reply| reply.read1().map_err(|err| self.error(err, context)))
            .map(|names: Vec<String>| {
                // Filter the names by the pattern
                if let Some(pattern) = pattern {
                    names
                        .into_iter()
                        .filter(|name| pattern.is_match(name))
                        .collect()
                } else {
                    names
                }
            })
    }
}
