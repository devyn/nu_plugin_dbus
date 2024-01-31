use dbus::{channel::{Channel, BusType}, Message};
use nu_plugin::LabeledError;
use nu_protocol::{Spanned, Value};

use crate::config::{DbusClientConfig, DbusBusChoice};

/// Executes D-Bus actions on a connection, handling nushell types
pub struct DbusClient {
    config: DbusClientConfig,
    conn: Channel,
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

    /// Call a D-Bus method and wait for the response
    pub fn call(
        &self,
        dest: &Spanned<String>,
        object: &Spanned<String>,
        interface: &Spanned<String>,
        method: &Spanned<String>,
    ) -> Result<Value, LabeledError> {
        // TODO accept arguments
        macro_rules! error {
            ($label:expr) => (LabeledError {
                label: $label,
                msg: "while calling a D-Bus method".into(),
                span: Some(self.config.span)
            })
        }

        // Validate inputs before sending to the dbus lib so we don't panic
        macro_rules! validate_with {
            ($type:ty, $spanned:expr) => (<$type>::new(&$spanned.item).map_err(|msg| {
                LabeledError {
                    label: msg,
                    msg: "this argument is incorrect".into(),
                    span: Some($spanned.span),
                }
            }))
        }
        let valid_dest = validate_with!(dbus::strings::BusName, dest)?;
        let valid_object = validate_with!(dbus::strings::Path, object)?;
        let valid_interface = validate_with!(dbus::strings::Interface, interface)?;
        let valid_method = validate_with!(dbus::strings::Member, method)?;

        // Construct the method call message
        let message = Message::new_method_call(
            valid_dest,
            valid_object,
            valid_interface,
            valid_method,
        ).map_err(|err| error!(err))?;

        // Send it on the channel and get the response
        let resp = self.conn.send_with_reply_and_block(message, self.config.timeout.item)
            .map_err(|err| error!(err.to_string()))?;

        crate::convert::from_message(&resp).map_err(|err| error!(err))
    }
}
