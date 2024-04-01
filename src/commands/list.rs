use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Example, LabeledError, Signature, SyntaxShape, Type, Value};

use crate::{client::DbusClient, config::DbusClientConfig, pattern::Pattern, DbusSignatureUtilExt};

pub struct List;

impl SimplePluginCommand for List {
    type Plugin = crate::NuPluginDbus;

    fn name(&self) -> &str {
        "dbus list"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .dbus_command()
            .accepts_dbus_client_options()
            .accepts_timeout()
            .input_output_type(Type::Nothing, Type::List(Type::String.into()))
            .optional(
                "pattern",
                SyntaxShape::String,
                "An optional glob-like pattern to filter the result by",
            )
    }

    fn usage(&self) -> &str {
        "List all available connection names on the bus"
    }

    fn extra_usage(&self) -> &str {
        "These can be used as arguments for --dest on any of the other commands."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["dbus", "list", "find", "search", "help"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "dbus list",
                description: "List all names available on the bus",
                result: None,
            },
            Example {
                example: "dbus list org.freedesktop.*",
                description: "List top-level freedesktop.org names on the bus \
                            (e.g. matches `org.freedesktop.PowerManagement`, \
                             but not `org.freedesktop.Management.Inhibit`)",
                result: Some(Value::test_list(vec![
                    Value::test_string("org.freedesktop.DBus"),
                    Value::test_string("org.freedesktop.Flatpak"),
                    Value::test_string("org.freedesktop.Notifications"),
                ])),
            },
            Example {
                example: "dbus list org.mpris.MediaPlayer2.**",
                description: "List all MPRIS2 media players on the bus",
                result: Some(Value::test_list(vec![
                    Value::test_string("org.mpris.MediaPlayer2.spotify"),
                    Value::test_string("org.mpris.MediaPlayer2.kdeconnect.mpris_000001"),
                ])),
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
        let pattern = call
            .opt::<String>(0)?
            .map(|pat| Pattern::new(&pat, Some('.')));
        let result = dbus.list(pattern.as_ref())?;
        Ok(Value::list(
            result
                .into_iter()
                .map(|s| Value::string(s, call.head))
                .collect(),
            call.head,
        ))
    }
}
