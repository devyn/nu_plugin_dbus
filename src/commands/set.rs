use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Example, LabeledError, Signature, SyntaxShape, Type, Value};

use crate::{client::DbusClient, config::DbusClientConfig, DbusSignatureUtilExt};

pub struct Set;

impl SimplePluginCommand for Set {
    type Plugin = crate::NuPluginDbus;

    fn name(&self) -> &str {
        "dbus set"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .dbus_command()
            .accepts_dbus_client_options()
            .accepts_timeout()
            .input_output_type(Type::Nothing, Type::Nothing)
            .named(
                "signature",
                SyntaxShape::String,
                "Signature of the value to set, in D-Bus format.\n    \
                     If not provided, it will be determined from introspection.\n    \
                     If --no-introspect is specified and this is not provided, it will \
                       be guessed (poorly)",
                None,
            )
            .required_named(
                "dest",
                SyntaxShape::String,
                "The name of the connection to write the property on",
                None,
            )
            .required(
                "object",
                SyntaxShape::String,
                "The path to the object to write the property on",
            )
            .required(
                "interface",
                SyntaxShape::String,
                "The name of the interface the property belongs to",
            )
            .required(
                "property",
                SyntaxShape::String,
                "The name of the property to write",
            )
            .required(
                "value",
                SyntaxShape::Any,
                "The value to write to the property",
            )
    }

    fn description(&self) -> &str {
        "Set a D-Bus property"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["dbus", "property", "write", "put"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "dbus set --dest=org.mpris.MediaPlayer2.spotify \
                            /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player \
                            Volume 0.5",
            description: "Set the volume of Spotify to 50%",
            result: None,
        }]
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
        dbus.set(
            &call.get_flag("dest")?.unwrap(),
            &call.req(0)?,
            &call.req(1)?,
            &call.req(2)?,
            call.get_flag("signature")?.as_ref(),
            &call.req(3)?,
        )?;
        Ok(Value::nothing(call.head))
    }
}
