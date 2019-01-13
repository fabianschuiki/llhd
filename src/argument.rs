// Copyright (c) 2017 Fabian Schuiki

use ty::*;
use value::*;

/// A function argument or process/entity input or output.
pub struct Argument {
    id: ArgumentRef,
    ty: Type,
    name: Option<String>,
}

impl Argument {
    /// Create a new argument of the given type.
    pub fn new(ty: Type) -> Argument {
        Argument {
            id: ArgumentRef::new(ValueId::alloc()),
            ty: ty,
            name: None,
        }
    }

    /// Obtain a reference to this argument.
    pub fn as_ref(&self) -> ArgumentRef {
        self.id
    }

    /// Set the name of the argument.
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = Some(name.into());
    }
}

impl Value for Argument {
    fn id(&self) -> ValueId {
        self.id.into()
    }

    fn ty(&self) -> Type {
        self.ty.clone()
    }

    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|x| x as &str)
    }

    fn is_global(&self) -> bool {
        false
    }
}
