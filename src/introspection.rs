use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Node {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "interface")]
    pub interfaces: Vec<Interface>,
    #[serde(default, rename = "node")]
    pub children: Vec<Node>,
}

impl Node {
    pub fn from_xml(xml: &str) -> Result<Node, serde_xml_rs::Error> {
        let mut deserializer = serde_xml_rs::de::Deserializer::new_from_reader(xml.as_bytes())
            .non_contiguous_seq_elements(true);
        Node::deserialize(&mut deserializer)
    }

    #[cfg(test)]
    pub fn with_name(name: impl Into<String>) -> Node {
        Node {
            name: Some(name.into()),
            interfaces: vec![],
            children: vec![],
        }
    }

    pub fn get_interface(&self, name: &str) -> Option<&Interface> {
        self.interfaces.iter().find(|i| i.name == name)
    }

    /// Find a method on an interface on this node, and then generate the signature of the method
    /// args
    pub fn get_method_args_signature(&self, interface: &str, method: &str) -> Option<String> {
        Some(self.get_interface(interface)?.get_method(method)?.in_signature())
    }

    /// Find the signature of a property on an interface on this node
    pub fn get_property_signature(&self, interface: &str, property: &str) -> Option<&str> {
        Some(&self.get_interface(interface)?.get_property(property)?.r#type)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Interface {
    pub name: String,
    #[serde(default, rename = "method")]
    pub methods: Vec<Method>,
    #[serde(default, rename = "signal")]
    pub signals: Vec<Signal>,
    #[serde(default, rename = "property")]
    pub properties: Vec<Property>,
    #[serde(default, rename = "annotation")]
    pub annotations: Vec<Annotation>,
}

impl Interface {
    pub fn get_method(&self, name: &str) -> Option<&Method> {
        self.methods.iter().find(|m| m.name == name)
    }

    #[allow(dead_code)]
    pub fn get_signal(&self, name: &str) -> Option<&Signal> {
        self.signals.iter().find(|s| s.name == name)
    }

    pub fn get_property(&self, name: &str) -> Option<&Property> {
        self.properties.iter().find(|p| p.name == name)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Method {
    pub name: String,
    #[serde(default, rename = "arg")]
    pub args: Vec<MethodArg>,
    #[serde(default, rename = "annotation")]
    pub annotations: Vec<Annotation>,
}

impl Method {
    /// Get the signature of the method args
    pub fn in_signature(&self) -> String {
        self.args.iter()
            .filter(|arg| arg.direction == Direction::In)
            .map(|arg| &arg.r#type[..])
            .collect()
    }

    #[allow(dead_code)]
    /// Get the signature of the method result
    pub fn out_signature(&self) -> String {
        self.args.iter()
            .filter(|arg| arg.direction == Direction::Out)
            .map(|arg| &arg.r#type[..])
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct MethodArg {
    #[serde(default)]
    pub name: Option<String>,
    pub r#type: String,
    #[serde(default)]
    pub direction: Direction,
}

impl MethodArg {
    #[cfg(test)]
    pub fn new(
        name: impl Into<String>,
        r#type: impl Into<String>,
        direction: Direction
    ) -> MethodArg {
        MethodArg {
            name: Some(name.into()),
            r#type: r#type.into(),
            direction,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    #[default]
    In,
    Out,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Signal {
    pub name: String,
    #[serde(default, rename = "arg")]
    pub args: Vec<SignalArg>,
    #[serde(default, rename = "annotation")]
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct SignalArg {
    #[serde(default)]
    pub name: Option<String>,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Property {
    pub name: String,
    pub r#type: String,
    pub access: Access,
    #[serde(default, rename = "annotation")]
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Access {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Annotation {
    pub name: String,
    pub value: String,
}

impl Annotation {
    #[cfg(test)]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Annotation {
        Annotation { name: name.into(), value: value.into() }
    }
}

#[cfg(test)]
pub fn test_introspection_doc_rs() -> Node {
    Node {
        name: Some("/com/example/sample_object0".into()),
        interfaces: vec![Interface {
            name: "com.example.SampleInterface0".into(),
            methods: vec![
                Method {
                    name: "Frobate".into(),
                    args: vec![
                        MethodArg::new("foo", "i", Direction::In),
                        MethodArg::new("bar", "as", Direction::In),
                        MethodArg::new("baz", "a{us}", Direction::Out),
                    ],
                    annotations: vec![
                        Annotation::new("org.freedesktop.DBus.Deprecated", "true"),
                    ],
                },
                Method {
                    name: "Bazify".into(),
                    args: vec![
                        MethodArg::new("bar", "(iiu)", Direction::In),
                        MethodArg::new("len", "u", Direction::Out),
                        MethodArg::new("bar", "v", Direction::Out),
                    ],
                    annotations: vec![],
                },
                Method {
                    name: "Mogrify".into(),
                    args: vec![
                        MethodArg::new("bar", "(iiav)", Direction::In),
                    ],
                    annotations: vec![]
                },
            ],
            signals: vec![
                Signal {
                    name: "Changed".into(),
                    args: vec![
                        SignalArg { name: "new_value".into(), r#type: "b".into() },
                    ],
                    annotations: vec![]
                },
            ],
            properties: vec![
                Property {
                    name: "Bar".into(),
                    r#type: "y".into(),
                    access: Access::ReadWrite,
                    annotations: vec![],
                }
            ],
            annotations: vec![]
        }],
        children: vec![
            Node::with_name("child_of_sample_object"),
            Node::with_name("another_child_of_sample_object"),
        ]
    }
}

#[test]
pub fn test_parse_introspection_doc() -> Result<(), serde_xml_rs::Error> {
    let xml = include_str!("test_introspection_doc.xml");
    let result = Node::from_xml(xml)?;
    assert_eq!(result, test_introspection_doc_rs());
    Ok(())
}

#[test]
pub fn test_get_method_args_signature() {
    assert_eq!(
        test_introspection_doc_rs()
            .get_method_args_signature("com.example.SampleInterface0", "Frobate"),
        Some("ias".into())
    );
}
