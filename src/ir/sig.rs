// Copyright (c) 2017-2021 Fabian Schuiki

//! Representation of the input and output arguments of functions, processes,
//! and entitites.

use crate::{
    ir::{Arg, Unit},
    table::PrimaryTable,
    ty::Type,
};

/// A description of the input and output arguments of a unit.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Signature {
    args: PrimaryTable<Arg, ArgData>,
    inp: Vec<Arg>,
    oup: Vec<Arg>,
    retty: Option<Type>,
}

/// Argument direction.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
enum ArgDir {
    Input,
    Output,
}

/// A single argument of a `Function`, `Process`, or `Entity`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArgData {
    ty: Type,
    dir: ArgDir,
    num: u16,
}

impl Signature {
    /// Create a new signature.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add an input argument.
    pub fn add_input(&mut self, ty: Type) -> Arg {
        let arg = self.args.add(ArgData {
            ty,
            dir: ArgDir::Input,
            num: self.inp.len() as u16,
        });
        self.inp.push(arg);
        arg
    }

    /// Add an output argument.
    pub fn add_output(&mut self, ty: Type) -> Arg {
        let arg = self.args.add(ArgData {
            ty,
            dir: ArgDir::Output,
            num: self.oup.len() as u16,
        });
        self.oup.push(arg);
        arg
    }

    /// Set the return type of the signature.
    pub fn set_return_type(&mut self, ty: Type) {
        self.retty = Some(ty);
    }

    /// Get the return type of the signature.
    pub fn return_type(&self) -> Type {
        self.retty.clone().unwrap()
    }

    /// Check whether the signature has any inputs.
    pub fn has_inputs(&self) -> bool {
        !self.inp.is_empty()
    }

    /// Check whether the signature has any outputs.
    pub fn has_outputs(&self) -> bool {
        !self.oup.is_empty()
    }

    /// Check whether the signature has a return type.
    pub fn has_return_type(&self) -> bool {
        self.retty.is_some()
    }

    /// Return an iterator over the inputs of the signature.
    pub fn inputs<'a>(&'a self) -> impl Iterator<Item = Arg> + 'a {
        self.inp.iter().cloned()
    }

    /// Return an iterator over the outputs of the signature.
    pub fn outputs<'a>(&'a self) -> impl Iterator<Item = Arg> + 'a {
        self.oup.iter().cloned()
    }

    /// Return an iterator over the arguments of the signature.
    ///
    /// Inputs come first, then outputs.
    pub fn args<'a>(&'a self) -> impl Iterator<Item = Arg> + 'a {
        self.inputs().chain(self.outputs())
    }

    /// Return the type of argument `arg`.
    pub fn arg_type(&self, arg: Arg) -> Type {
        self.args[arg].ty.clone()
    }

    /// Check whether `arg` is an input.
    pub fn is_input(&self, arg: Arg) -> bool {
        self.args[arg].dir == ArgDir::Input
    }

    /// Check whether `arg` is an output.
    pub fn is_output(&self, arg: Arg) -> bool {
        self.args[arg].dir == ArgDir::Output
    }

    /// Dump the signature in human-readable form.
    pub fn dump<'a>(&'a self, unit: &Unit<'a>) -> SignatureDumper<'a> {
        SignatureDumper(self, *unit)
    }
}

impl Eq for Signature {}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        self.args().count() == other.args().count()
            && self
                .args()
                .zip(other.args())
                .all(|(a, b)| self.args[a] == other.args[b])
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::iter::{once, repeat};
        write!(f, "(")?;
        for (arg, sep) in self.inputs().zip(once("").chain(repeat(", "))) {
            write!(f, "{}{}", sep, self.arg_type(arg))?;
        }
        if self.has_outputs() {
            write!(f, ") -> (")?;
            for (arg, sep) in self.outputs().zip(once("").chain(repeat(", "))) {
                write!(f, "{}{}", sep, self.arg_type(arg))?;
            }
        }
        write!(f, ")")?;
        if let Some(ref retty) = self.retty {
            write!(f, " {}", retty)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Temporary object to dump a `Signature` in human-readable form for debugging.
pub struct SignatureDumper<'a>(&'a Signature, Unit<'a>);

impl std::fmt::Display for SignatureDumper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::iter::{once, repeat};
        write!(f, "(")?;
        for (arg, sep) in self.0.inputs().zip(once("").chain(repeat(", "))) {
            let value = self.1.arg_value(arg);
            write!(
                f,
                "{}{} {}",
                sep,
                self.1.value_type(value),
                value.dump(&self.1)
            )?;
        }
        write!(f, ")")?;
        if self.0.has_outputs() {
            write!(f, " -> (")?;
            for (arg, sep) in self.0.outputs().zip(once("").chain(repeat(", "))) {
                let value = self.1.arg_value(arg);
                write!(
                    f,
                    "{}{} {}",
                    sep,
                    self.1.value_type(value),
                    value.dump(&self.1)
                )?;
            }
            write!(f, ")")?;
        }
        if self.0.has_return_type() {
            write!(f, " {}", self.0.return_type())?;
        }
        Ok(())
    }
}
