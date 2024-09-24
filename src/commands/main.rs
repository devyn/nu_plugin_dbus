use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Value};

use crate::DbusSignatureUtilExt;

pub struct Main;

impl SimplePluginCommand for Main {
    type Plugin = crate::NuPluginDbus;

    fn name(&self) -> &str {
        "dbus"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).dbus_command()
    }

    fn description(&self) -> &str {
        "Commands for interacting with D-Bus"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["dbus"]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        Ok(Value::string(engine.get_help()?, call.head))
    }
}
