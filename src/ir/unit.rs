// Copyright (c) 2017-2020 Fabian Schuiki

//! Common functionality of `Function`, `Process`, and `Entity`.

/// A name of a function, process, or entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitName {
    /// An anonymous name, like `%42`.
    Anonymous(u32),
    /// A local name, like `%foo`.
    Local(String),
    /// A global name, like `@foo`.
    Global(String),
}

impl UnitName {
    // Create a new anonymous unit name.
    pub fn anonymous(id: u32) -> Self {
        UnitName::Anonymous(id)
    }

    // Create a new local unit name.
    pub fn local(name: impl Into<String>) -> Self {
        UnitName::Local(name.into())
    }

    // Create a new global unit name.
    pub fn global(name: impl Into<String>) -> Self {
        UnitName::Global(name.into())
    }

    /// Check whether this is a local name.
    ///
    /// Local names can only be linked within the same module.
    pub fn is_local(&self) -> bool {
        match self {
            UnitName::Anonymous(..) | UnitName::Local(..) => true,
            _ => false,
        }
    }

    /// Check whether this is a global name.
    ///
    /// Global names may be referenced by other modules and are considered by
    /// the global linker.
    pub fn is_global(&self) -> bool {
        match self {
            UnitName::Global(..) => true,
            _ => false,
        }
    }

    /// Get the underlying name.
    pub fn get_name(&self) -> Option<&str> {
        match self {
            UnitName::Global(n) | UnitName::Local(n) => Some(n.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Display for UnitName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnitName::Anonymous(id) => write!(f, "%{}", id),
            UnitName::Local(n) => write!(f, "%{}", n),
            UnitName::Global(n) => write!(f, "@{}", n),
        }
    }
}

/// The three different units that may appear in LLHD IR.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitKind {
    /// A `Function`.
    Function,
    /// A `Process`.
    Process,
    /// An `Entity`.
    Entity,
}

impl std::fmt::Display for UnitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnitKind::Function => write!(f, "func"),
            UnitKind::Process => write!(f, "proc"),
            UnitKind::Entity => write!(f, "entity"),
        }
    }
}
