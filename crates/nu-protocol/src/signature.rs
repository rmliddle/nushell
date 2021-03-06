use crate::syntax_shape::SyntaxShape;
use crate::type_shape::Type;
use indexmap::IndexMap;
use nu_source::{b, DebugDocBuilder, PrettyDebug, PrettyDebugWithSource};
use serde::{Deserialize, Serialize};

/// The types of named parameter that a command can have
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NamedType {
    /// A flag without any associated argument. eg) `foo --bar`
    Switch,
    /// A mandatory flag, with associated argument. eg) `foo --required xyz`
    Mandatory(SyntaxShape),
    /// An optional flag, with associated argument. eg) `foo --optional abc`
    Optional(SyntaxShape),
    Help,
}

/// The type of positional arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionalType {
    /// A mandatory postional argument with the expected shape of the value
    Mandatory(String, SyntaxShape),
    /// An optional positional argument with the expected shape of the value
    Optional(String, SyntaxShape),
}

impl PrettyDebug for PositionalType {
    /// Prepare the PositionalType for pretty-printing
    fn pretty(&self) -> DebugDocBuilder {
        match self {
            PositionalType::Mandatory(string, shape) => {
                b::description(string) + b::delimit("(", shape.pretty(), ")").into_kind().group()
            }
            PositionalType::Optional(string, shape) => {
                b::description(string)
                    + b::operator("?")
                    + b::delimit("(", shape.pretty(), ")").into_kind().group()
            }
        }
    }
}

impl PositionalType {
    /// Helper to create a mandatory positional argument type
    pub fn mandatory(name: &str, ty: SyntaxShape) -> PositionalType {
        PositionalType::Mandatory(name.to_string(), ty)
    }

    /// Helper to create a mandatory positional argument with an "any" type
    pub fn mandatory_any(name: &str) -> PositionalType {
        PositionalType::Mandatory(name.to_string(), SyntaxShape::Any)
    }

    /// Helper to create a mandatory positional argument with a block type
    pub fn mandatory_block(name: &str) -> PositionalType {
        PositionalType::Mandatory(name.to_string(), SyntaxShape::Block)
    }

    /// Helper to create a optional positional argument type
    pub fn optional(name: &str, ty: SyntaxShape) -> PositionalType {
        PositionalType::Optional(name.to_string(), ty)
    }

    /// Helper to create a optional positional argument with an "any" type
    pub fn optional_any(name: &str) -> PositionalType {
        PositionalType::Optional(name.to_string(), SyntaxShape::Any)
    }

    /// Gets the name of the positional argument
    pub fn name(&self) -> &str {
        match self {
            PositionalType::Mandatory(s, _) => s,
            PositionalType::Optional(s, _) => s,
        }
    }

    /// Gets the expected type of a positional argument
    pub fn syntax_type(&self) -> SyntaxShape {
        match *self {
            PositionalType::Mandatory(_, t) => t,
            PositionalType::Optional(_, t) => t,
        }
    }
}

type Description = String;

/// The full signature of a command. All commands have a signature similar to a function signature.
/// Commands will use this information to register themselves with Nu's core engine so that the command
/// can be invoked, help can be displayed, and calls to the command can be error-checked.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Signature {
    /// The name of the command. Used when calling the command
    pub name: String,
    /// Usage instructions about the command
    pub usage: String,
    /// The list of positional arguments, both required and optional, and their corresponding types and help text
    pub positional: Vec<(PositionalType, Description)>,
    /// After the positional arguments, a catch-all for the rest of the arguments that might follow, their type, and help text
    pub rest_positional: Option<(SyntaxShape, Description)>,
    /// The named flags with corresponding type and help text
    pub named: IndexMap<String, (NamedType, Description)>,
    /// The type of values being sent out from the command into the pipeline, if any
    pub yields: Option<Type>,
    /// The type of values being read in from the pipeline into the command, if any
    pub input: Option<Type>,
    /// If the command is expected to filter data, or to consume it (as a sink)
    pub is_filter: bool,
}

impl PrettyDebugWithSource for Signature {
    /// Prepare a Signature for pretty-printing
    fn pretty_debug(&self, source: &str) -> DebugDocBuilder {
        b::typed(
            "signature",
            b::description(&self.name)
                + b::preceded(
                    b::space(),
                    b::intersperse(
                        self.positional
                            .iter()
                            .map(|(ty, _)| ty.pretty_debug(source)),
                        b::space(),
                    ),
                ),
        )
    }
}

impl Signature {
    /// Create a new command signagure with the given name
    pub fn new(name: impl Into<String>) -> Signature {
        Signature {
            name: name.into(),
            usage: String::new(),
            positional: vec![],
            rest_positional: None,
            named: indexmap::indexmap! {"help".into() => (NamedType::Help, "Display this help message".into())},
            is_filter: false,
            yields: None,
            input: None,
        }
    }

    /// Create a new signature
    pub fn build(name: impl Into<String>) -> Signature {
        Signature::new(name.into())
    }

    /// Add a description to the signature
    pub fn desc(mut self, usage: impl Into<String>) -> Signature {
        self.usage = usage.into();
        self
    }

    /// Add a required positional argument to the signature
    pub fn required(
        mut self,
        name: impl Into<String>,
        ty: impl Into<SyntaxShape>,
        desc: impl Into<String>,
    ) -> Signature {
        self.positional.push((
            PositionalType::Mandatory(name.into(), ty.into()),
            desc.into(),
        ));

        self
    }

    /// Add an optional positional argument to the signature
    pub fn optional(
        mut self,
        name: impl Into<String>,
        ty: impl Into<SyntaxShape>,
        desc: impl Into<String>,
    ) -> Signature {
        self.positional.push((
            PositionalType::Optional(name.into(), ty.into()),
            desc.into(),
        ));

        self
    }

    /// Add an optional named flag argument to the signature
    pub fn named(
        mut self,
        name: impl Into<String>,
        ty: impl Into<SyntaxShape>,
        desc: impl Into<String>,
    ) -> Signature {
        self.named
            .insert(name.into(), (NamedType::Optional(ty.into()), desc.into()));

        self
    }

    /// Add a required named flag argument to the signature
    pub fn required_named(
        mut self,
        name: impl Into<String>,
        ty: impl Into<SyntaxShape>,
        desc: impl Into<String>,
    ) -> Signature {
        self.named
            .insert(name.into(), (NamedType::Mandatory(ty.into()), desc.into()));

        self
    }

    /// Add a switch to the signature
    pub fn switch(mut self, name: impl Into<String>, desc: impl Into<String>) -> Signature {
        self.named
            .insert(name.into(), (NamedType::Switch, desc.into()));

        self
    }

    /// Remove the default help switch
    pub fn remove_help(mut self) -> Signature {
        self.named.remove("help");

        self
    }

    /// Set the filter flag for the signature
    pub fn filter(mut self) -> Signature {
        self.is_filter = true;
        self
    }

    /// Set the type for the "rest" of the positional arguments
    pub fn rest(mut self, ty: SyntaxShape, desc: impl Into<String>) -> Signature {
        self.rest_positional = Some((ty, desc.into()));
        self
    }

    /// Add a type for the output of the command to the signature
    pub fn yields(mut self, ty: Type) -> Signature {
        self.yields = Some(ty);
        self
    }

    /// Add a type for the input of the command to the signature
    pub fn input(mut self, ty: Type) -> Signature {
        self.input = Some(ty);
        self
    }
}
