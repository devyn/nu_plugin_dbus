use nu_plugin::{serve_plugin, MsgPackSerializer, Plugin, EvaluatedCall, LabeledError};
use nu_protocol::{PluginSignature, Value};

fn main() {
    serve_plugin(&mut NuPluginDbus, MsgPackSerializer)
}

struct NuPluginDbus;

impl Plugin for NuPluginDbus {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![
            PluginSignature::build("dbus")
                .usage("Commands for interacting with D-Bus")
                .search_terms(vec!["dbus".into()])
                .category(nu_protocol::Category::Platform),
        ]
    }

    fn run(
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        match name {
            "dbus" => Err(LabeledError {
                label: "The `dbus` command requires a subcommand".into(),
                msg: "add --help to see subcommands".into(),
                span: Some(call.head)
            }),

            _ => Err(LabeledError {
                label: "Plugin invoked with unknown command name".into(),
                msg: "unknown command".into(),
                span: Some(call.head)
            })
        }
    }
}
