use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{Example, LabeledError, Signature, SyntaxShape, Type, Value};

use crate::{client::DbusClient, config::DbusClientConfig, DbusSignatureUtilExt};

pub struct Call;

impl SimplePluginCommand for Call {
    type Plugin = crate::NuPluginDbus;

    fn name(&self) -> &str {
        "dbus call"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .dbus_command()
            .accepts_dbus_client_options()
            .accepts_timeout()
            .input_output_type(Type::Nothing, Type::Any)
            .named(
                "signature",
                SyntaxShape::String,
                "Signature of the arguments to send, in D-Bus format.\n    \
                 If not provided, they will be determined from introspection.\n    \
                 If --no-introspect is specified and this is not provided, they will \
                   be guessed (poorly)",
                None,
            )
            .switch(
                "no-flatten",
                "Always return a list of all return values",
                None,
            )
            .switch(
                "no-introspect",
                "Don't use introspection to determine the correct argument signature",
                None,
            )
            .required_named(
                "dest",
                SyntaxShape::String,
                "The name of the connection to send the method to",
                None,
            )
            .required(
                "object",
                SyntaxShape::String,
                "The path to the object to call the method on",
            )
            .required(
                "interface",
                SyntaxShape::String,
                "The name of the interface the method belongs to",
            )
            .required(
                "method",
                SyntaxShape::String,
                "The name of the method to send",
            )
            .rest(
                "args",
                SyntaxShape::Any,
                "Arguments to send with the method call",
            )
    }

    fn description(&self) -> &str {
        "Call a method and get its response"
    }

    fn extra_description(&self) -> &str {
        "Returns an array if the method call returns more than one value."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["dbus"]
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "dbus call --dest=org.freedesktop.DBus \
                    /org/freedesktop/DBus org.freedesktop.DBus.Peer Ping",
                description: "Ping the D-Bus server itself",
                result: None,
            },
            Example {
                example: "dbus call --dest=org.freedesktop.Notifications \
                    /org/freedesktop/Notifications org.freedesktop.Notifications \
                    Notify \"Floppy disks\" 0 \"media-floppy\" \"Rarely seen\" \
                    \"But sometimes still used\" [] {} 5000",
                description: "Show a notification on the desktop for 5 seconds",
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
        let values = dbus.call(
            &call.get_flag("dest")?.unwrap(),
            &call.req(0)?,
            &call.req(1)?,
            &call.req(2)?,
            call.get_flag("signature")?.as_ref(),
            &call.positional[3..],
        )?;

        let flatten = !call.get_flag::<bool>("no-flatten")?.unwrap_or(false);

        // Make the output easier to deal with by returning a list only if there are multiple return
        // values (not so common)
        match values.len() {
            0 if flatten => Ok(Value::nothing(call.head)),
            1 if flatten => Ok(values.into_iter().nth(0).unwrap()),
            _ => Ok(Value::list(values, call.head)),
        }
    }
}
