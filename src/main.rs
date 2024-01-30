use nu_plugin::{serve_plugin, MsgPackSerializer, Plugin, EvaluatedCall, LabeledError};
use nu_protocol::{PluginSignature, Value, SyntaxShape, PluginExample};

mod config;
mod client;

use config::*;
use client::*;

fn main() {
    serve_plugin(&mut NuPluginDbus, MsgPackSerializer)
}

/// The main plugin interface for nushell
struct NuPluginDbus;

impl Plugin for NuPluginDbus {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![
            PluginSignature::build("dbus")
                .is_dbus_command()
                .usage("Commands for interacting with D-Bus"),
            PluginSignature::build("dbus call")
                .is_dbus_command()
                .accepts_dbus_client_options()
                .usage("Call a method and get its response")
                .named("timeout", SyntaxShape::Duration, "How long to wait for a response", None)
                .required_named("dest", SyntaxShape::String,
                    "The name of the connection to send the method to",
                    None)
                .required("object", SyntaxShape::String,
                    "The path to the object to call the method on")
                .required("interface", SyntaxShape::String,
                    "The name of the interface the method belongs to")
                .required("method", SyntaxShape::String,
                    "The name of the method to send")
                .plugin_examples(vec![
                    PluginExample {
                        example: "dbus call --dest=org.freedesktop.DBus \
                            /org/freedesktop/DBus org.freedesktop.DBus.Peer Ping".into(),
                        description: "Ping the D-Bus server itself".into(),
                        result: None
                    }
                ]),
        ]
    }

    fn run(
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        match name {
            "dbus" => Err(LabeledError {
                label: "The `dbus` command requires a subcommand".into(),
                msg: "add --help to see subcommands".into(),
                span: Some(call.head)
            }),

            "dbus call" => self.call(call),

            _ => Err(LabeledError {
                label: "Plugin invoked with unknown command name".into(),
                msg: "unknown command".into(),
                span: Some(call.head)
            })
        }
    }
}

/// For conveniently adding the base options to a dbus command
trait DbusSignatureUtilExt {
    fn is_dbus_command(self) -> Self;
    fn accepts_dbus_client_options(self) -> Self;
}

impl DbusSignatureUtilExt for PluginSignature {
    fn is_dbus_command(self) -> Self {
        self.search_terms(vec!["dbus".into()])
            .category(nu_protocol::Category::Platform)
    }

    fn accepts_dbus_client_options(self) -> Self {
        self.switch("session", "Send to the session message bus (default)", None)
            .switch("system", "Send to the system message bus", None)
            .switch("started", "Send to the bus that started this process, if applicable", None)
            .named("bus", SyntaxShape::String, "Send to the bus server at the given address", None)
            .named("peer", SyntaxShape::String,
                "Send to a non-bus D-Bus server at the given address. \
                 Will not call the Hello method on initialization.",
                None)
    }
}

impl NuPluginDbus {
    fn call(&self, call: &EvaluatedCall) -> Result<Value, LabeledError> {
        let config = DbusClientConfig::try_from(call)?;
        let dbus = DbusClient::new(config)?;
        dbus.call(
            &call.get_flag("dest")?.unwrap(),
            &call.req(0)?,
            &call.req(1)?,
            &call.req(2)?,
        )?;
        // TODO handle response
        Ok(Value::nothing(call.head))
    }
}
