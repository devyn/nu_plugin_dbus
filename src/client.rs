use dbus::{blocking::LocalConnection, channel::{Channel, BusType}};
use nu_plugin::LabeledError;
use nu_protocol::Spanned;

use crate::config::{DbusClientConfig, DbusBusChoice};

/// Executes D-Bus actions on a connection, handling nushell types
pub struct DbusClient {
    config: DbusClientConfig,
    conn: LocalConnection,
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
            conn: LocalConnection::from(channel)
        })
    }

    /// Call a D-Bus method and wait for the response
    pub fn call(
        &self,
        dest: &Spanned<String>,
        object: &Spanned<String>,
        interface: &Spanned<String>,
        method: &Spanned<String>,
    ) -> Result<(), LabeledError> {
        // TODO convert & return response
        // TODO accept arguments
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

        // Send method call
        let proxy = self.conn.with_proxy(
            valid_dest,
            valid_object,
            self.config.timeout.item
        );
        let () = proxy.method_call(valid_interface, valid_method, ()).map_err(|err| {
            LabeledError {
                label: err.to_string(),
                msg: "while calling a D-Bus method".into(),
                span: Some(self.config.span),
            }
        })?;
        Ok(())
    }
}
