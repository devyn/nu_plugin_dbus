use dbus::{channel::{Channel, BusType}, Message};
use nu_plugin::LabeledError;
use nu_protocol::{Spanned, Value};

use crate::{config::{DbusClientConfig, DbusBusChoice}, dbus_type::DbusType, convert::to_message_item, introspection::Node};

/// Executes D-Bus actions on a connection, handling nushell types
pub struct DbusClient {
    config: DbusClientConfig,
    conn: Channel,
}

// Convenience macros for error handling
macro_rules! validate_with {
    ($type:ty, $spanned:expr) => (<$type>::new(&$spanned.item).map_err(|msg| {
        LabeledError {
            label: msg,
            msg: "this argument is incorrect".into(),
            span: Some($spanned.span),
        }
    }))
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
        }.map_err(|err| {
            LabeledError {
                label: err.to_string(),
                msg: "while connecting to D-Bus as specified here".into(),
                span: Some(config.bus_choice.span),
            }
        })?;
        Ok(DbusClient {
            config,
            conn: channel
        })
    }

    fn error(&self, err: impl std::fmt::Display, msg: impl std::fmt::Display) -> LabeledError {
        LabeledError {
            label: err.to_string(),
            msg: msg.to_string(),
            span: Some(self.config.span)
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
            "Introspect"
        ).map_err(|err| self.error(err, context))?;

        // Send and get the response
        let resp = self.conn.send_with_reply_and_block(message, self.config.timeout.item)
            .map_err(|err| self.error(err, context))?;

        // Parse it to a Node
        let xml: &str = resp.get1()
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
                label: format!("while getting interface {:?} method {:?} signature: {}",
                    interface.item,
                    method.item,
                    err),
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
        let mut valid_signature = signature.map(|s| DbusType::parse_all(&s.item).map_err(|err| {
            LabeledError {
                label: err,
                msg: "in signature specified here".into(),
                span: Some(s.span),
            }
        })).transpose()?;

        // If not provided, try introspection (unless disabled)
        if valid_signature.is_none() && self.config.introspect {
            match self.get_method_signature_by_introspection(dest, object, interface, method) {
                Ok(sig) => {
                    valid_signature = Some(sig);
                },
                Err(err) => {
                    eprintln!("Warning: D-Bus introspection failed on {:?}. \
                        Use `--no-introspect` or pass `--signature` to silence this warning. \
                        Cause: {}",
                        object.item,
                        err.label);
                }
            }
        }

        if let Some(sig) = &valid_signature {
            if sig.len() != args.len() {
                self.error(format!("expected {} arguments, got {}", sig.len(), args.len()), context);
            }
        }

        // Construct the method call message
        let mut message = Message::new_method_call(
            valid_dest,
            valid_object,
            valid_interface,
            valid_method,
        ).map_err(|err| self.error(err, context))?;

        // Convert the args to message items
        let sigs_iter = valid_signature.iter().flatten().map(Some).chain(std::iter::repeat(None));
        for (val, sig) in args.iter().zip(sigs_iter) {
            message = message.append1(to_message_item(val, sig)?);
        }

        // Send it on the channel and get the response
        let resp = self.conn.send_with_reply_and_block(message, self.config.timeout.item)
            .map_err(|err| self.error(err, context))?;

        crate::convert::from_message(&resp).map_err(|err| self.error(err, context))
    }
}
