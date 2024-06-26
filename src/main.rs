use nu_plugin::{serve_plugin, MsgPackSerializer, Plugin, PluginCommand};
use nu_protocol::SyntaxShape;

mod client;
mod commands;
mod config;
mod convert;
mod dbus_type;
mod introspection;
mod pattern;

fn main() {
    serve_plugin(&NuPluginDbus, MsgPackSerializer)
}

/// The main plugin interface for nushell
pub struct NuPluginDbus;

impl Plugin for NuPluginDbus {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(commands::Main),
            Box::new(commands::Introspect),
            Box::new(commands::Call),
            Box::new(commands::Get),
            Box::new(commands::GetAll),
            Box::new(commands::Set),
            Box::new(commands::List),
        ]
    }
}

/// For conveniently adding the base options to a dbus command
trait DbusSignatureUtilExt {
    fn dbus_command(self) -> Self;
    fn accepts_dbus_client_options(self) -> Self;
    fn accepts_timeout(self) -> Self;
}

impl DbusSignatureUtilExt for nu_protocol::Signature {
    fn dbus_command(self) -> Self {
        self.category(nu_protocol::Category::Platform)
    }

    fn accepts_dbus_client_options(self) -> Self {
        self.switch("session", "Send to the session message bus (default)", None)
            .switch("system", "Send to the system message bus", None)
            .switch(
                "started",
                "Send to the bus that started this process, if applicable",
                None,
            )
            .named(
                "bus",
                SyntaxShape::String,
                "Send to the bus server at the given address",
                None,
            )
            .named(
                "peer",
                SyntaxShape::String,
                "Send to a non-bus D-Bus server at the given address. \
                 Will not call the Hello method on initialization.",
                None,
            )
    }

    fn accepts_timeout(self) -> Self {
        self.named(
            "timeout",
            SyntaxShape::Duration,
            "How long to wait for a response",
            None,
        )
    }
}
