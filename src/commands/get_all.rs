use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Example, LabeledError, Signature, SyntaxShape, Type, Value};

use crate::{client::DbusClient, config::DbusClientConfig, DbusSignatureUtilExt};

pub struct GetAll;

impl SimplePluginCommand for GetAll {
    type Plugin = crate::NuPluginDbus;

    fn name(&self) -> &str {
        "dbus get-all"
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
                "The name of the connection to read the property from",
                None,
            )
            .required(
                "object",
                SyntaxShape::String,
                "The path to the object to read the property from",
            )
            .required(
                "interface",
                SyntaxShape::String,
                "The name of the interface the property belongs to",
            )
    }

    fn usage(&self) -> &str {
        "Get all D-Bus properties for the given object"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["dbus", "properties", "property", "get"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "dbus get-all --dest=org.mpris.MediaPlayer2.spotify \
                            /org/mpris/MediaPlayer2 \
                            org.mpris.MediaPlayer2.Player",
            description: "Get the current player state of Spotify",
            result: Some(Value::test_record(nu_protocol::record!(
                "CanPlay" => Value::test_bool(true),
                "Volume" => Value::test_float(0.43),
                "PlaybackStatus" => Value::test_string("Paused"),
            ))),
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
        dbus.get_all(
            &call.get_flag("dest")?.unwrap(),
            &call.req(0)?,
            &call.req(1)?,
        )
    }
}
