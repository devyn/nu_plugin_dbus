use std::time::Duration;

use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{Spanned, Span};

/// General configuration related to the D-Bus client connection
#[derive(Debug, Clone)]
pub struct DbusClientConfig {
    pub span: Span,
    /// Which bus should we connect to?
    pub bus_choice: Spanned<DbusBusChoice>,
    /// How long to wait for a method call to return
    pub timeout: Spanned<Duration>,
    /// Enable introspection if signature unknown (default true)
    pub introspect: bool,
}

/// Where to connect to the D-Bus server
#[derive(Debug, Clone, Default)]
pub enum DbusBusChoice {
    /// Connect to the session bus
    #[default]
    Session,
    /// Connect to the system bus
    System,
    /// Connect to the bus that started this process
    Started,
    /// Connect to a bus instance at the given address
    Bus(String),
    /// Connect to a non-bus D-Bus server at the given address (will not send Hello)
    Peer(String),
}

impl TryFrom<&EvaluatedCall> for DbusClientConfig {
    type Error = LabeledError;

    fn try_from(call: &EvaluatedCall) -> Result<Self, Self::Error> {
        let mut config = DbusClientConfig {
            span: call.head,
            bus_choice: Spanned { item: DbusBusChoice::default(), span: call.head },
            timeout: Spanned { item: Duration::from_secs(2), span: call.head },
            introspect: true,
        };

        // Handle recognized config args
        for (name, value) in &call.named {
            match &name.item[..] {
                r#type @ ("session" | "system" | "started") => {
                    if value.is_none() || value.as_ref().is_some_and(|v| v.is_true()) {
                        let dest = match r#type {
                            "session" => DbusBusChoice::Session,
                            "system" => DbusBusChoice::System,
                            "started" => DbusBusChoice::Started,
                            _ => unreachable!()
                        };
                        config.bus_choice = Spanned { item: dest, span: name.span };
                    }
                },
                r#type @ ("bus" | "peer") => {
                    if let Some(value) = value {
                        let address = value.as_str()?;
                        let dest = match r#type {
                            "bus" => DbusBusChoice::Bus(address.to_owned()),
                            "peer" => DbusBusChoice::Peer(address.to_owned()),
                            _ => unreachable!()
                        };
                        config.bus_choice = Spanned { item: dest, span: value.span() };
                    }
                },
                "timeout" => {
                    if let Some(value) = value {
                        let nanos: u64 = value.as_duration()?.try_into().map_err(|_| {
                            LabeledError {
                                label: "Timeout must be a positive duration".into(),
                                msg: "invalid timeout specified here".into(),
                                span: Some(value.span()),
                            }
                        })?;
                        let item = Duration::from_nanos(nanos);
                        config.timeout = Spanned { item, span: value.span() };
                    }
                },
                "no-introspect" => {
                    config.introspect = !value.as_ref()
                        .and_then(|v| v.as_bool().ok())
                        .unwrap_or(false);
                },
                _ => ()
            }
        }

        Ok(config)
    }
}
