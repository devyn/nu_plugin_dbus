use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Example, LabeledError, Signature, SyntaxShape, Type, Value};

use crate::{client::DbusClient, config::DbusClientConfig, DbusSignatureUtilExt};

pub struct Get;

impl SimplePluginCommand for Get {
    type Plugin = crate::NuPluginDbus;

    fn name(&self) -> &str {
        "dbus get"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .dbus_command()
            .accepts_dbus_client_options()
            .accepts_timeout()
            .input_output_type(Type::Nothing, Type::Any)
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
            .required(
                "property",
                SyntaxShape::String,
                "The name of the property to read",
            )
    }

    fn description(&self) -> &str {
        "Get a D-Bus property"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["dbus", "property", "read"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            example: "dbus get --dest=org.mpris.MediaPlayer2.spotify \
                        /org/mpris/MediaPlayer2 \
                        org.mpris.MediaPlayer2.Player Metadata",
            description: "Get the currently playing song in Spotify",
            result: Some(Value::test_record(nu_protocol::record!(
                "xesam:title" => Value::test_string("Birdie"),
                "xesam:artist" => Value::test_list(vec![
                    Value::test_string("LOVE PSYCHEDELICO")
                ]),
                "xesam:album" => Value::test_string("Love Your Love"),
                "xesam:url" => Value::test_string("https://open.spotify.com/track/51748BvzeeMs4PIdPuyZmv"),
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
        dbus.get(
            &call.get_flag("dest")?.unwrap(),
            &call.req(0)?,
            &call.req(1)?,
            &call.req(2)?,
        )
    }
}
