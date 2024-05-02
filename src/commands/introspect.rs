use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Example, LabeledError, Signature, SyntaxShape, Type, Value};

use crate::{client::DbusClient, config::DbusClientConfig, DbusSignatureUtilExt};

pub struct Introspect;

impl SimplePluginCommand for Introspect {
    type Plugin = crate::NuPluginDbus;

    fn name(&self) -> &str {
        "dbus introspect"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .dbus_command()
            .accepts_dbus_client_options()
            .accepts_timeout()
            .input_output_type(Type::Nothing, Type::Record([].into()))
            .required_named(
                "dest",
                SyntaxShape::String,
                "The name of the connection that owns the object",
                None,
            )
            .required(
                "object",
                SyntaxShape::String,
                "The path to the object to introspect",
            )
    }

    fn usage(&self) -> &str {
        "Introspect a D-Bus object"
    }

    fn extra_usage(&self) -> &str {
        "Returns information about available nodes, interfaces, methods, \
            signals, and properties on the given object path"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["dbus", "help", "method"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "dbus introspect --dest=org.mpris.MediaPlayer2.spotify \
                    /org/mpris/MediaPlayer2 | explore",
                description: "Look at the MPRIS2 interfaces exposed by Spotify",
                result: None,
            },
            Example {
                example: "dbus introspect --dest=org.kde.plasmashell \
                    /org/kde/osdService | get interfaces | \
                    where name == org.kde.osdService | get 0.methods",
                description: "Get methods exposed by KDE Plasma's on-screen display \
                    service",
                result: None,
            },
            Example {
                example: "dbus introspect --dest=org.kde.KWin / | get children | \
                    select name",
                description: "List objects exposed by KWin",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let config = DbusClientConfig::try_from(call)?;
        let dbus = DbusClient::new(config)?;
        let node = dbus.introspect(&call.get_flag("dest")?.unwrap(), &call.req(0)?)?;
        Ok(node.to_value(call.head))
    }
}
