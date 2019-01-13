// Copyright (c) 2017 Fabian Schuiki

use argument::*;
use block::*;
use inst::*;
use module::ModuleContext;
use seq_body::*;
use ty::*;
use unit::*;
use value::*;

/// A function. Sequentially executes instructions to determine a result value
/// from its inputs. Implements *control flow* and *immediate execution*.
pub struct Function {
    id: FunctionRef,
    global: bool,
    name: String,
    ty: Type,
    args: Vec<Argument>,
    body: SeqBody,
}

impl Function {
    /// Create a new function with the given name and type signature. Anonymous
    /// arguments are created for each argument in the type signature. Use the
    /// `args_mut` function to get a hold of these arguments and assign names
    /// and additional data to them.
    pub fn new(name: String, ty: Type) -> Function {
        let args = {
            let (arg_tys, _) = ty.as_func();
            arg_tys.iter().map(|t| Argument::new(t.clone())).collect()
        };
        Function {
            id: FunctionRef::new(ValueId::alloc()),
            global: true,
            name: name,
            ty: ty,
            args: args,
            body: SeqBody::new(),
        }
    }

    /// Obtain a reference to this function.
    pub fn as_ref(&self) -> FunctionRef {
        self.id
    }

    /// Get the name of the function.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the return type of the function.
    pub fn return_ty(&self) -> &Type {
        self.ty.as_func().1
    }

    /// Get a graph reference to one of the arguments of the function.
    pub fn arg(&self, idx: usize) -> ArgumentRef {
        self.args[idx].as_ref()
    }

    /// Get a reference to the arguments of the function.
    pub fn args(&self) -> &[Argument] {
        &self.args
    }

    /// Get a mutable reference to the arguments of the function.
    pub fn args_mut(&mut self) -> &mut [Argument] {
        &mut self.args
    }

    /// Get a reference to the sequential body of the function.
    pub fn body(&self) -> &SeqBody {
        &self.body
    }

    /// Get a mutable reference to the sequential body of the function.
    pub fn body_mut(&mut self) -> &mut SeqBody {
        &mut self.body
    }
}

impl Value for Function {
    fn id(&self) -> ValueId {
        self.id.into()
    }

    fn ty(&self) -> Type {
        self.ty.clone()
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn is_global(&self) -> bool {
        self.global
    }
}

pub struct FunctionContext<'tctx> {
    module: &'tctx ModuleContext<'tctx>,
    function: &'tctx Function,
}

impl<'tctx> FunctionContext<'tctx> {
    pub fn new(module: &'tctx ModuleContext, function: &'tctx Function) -> FunctionContext<'tctx> {
        FunctionContext {
            module: module,
            function: function,
        }
    }
}

impl<'tctx> Context for FunctionContext<'tctx> {
    fn parent(&self) -> Option<&Context> {
        Some(self.module.as_context())
    }

    fn try_value(&self, value: &ValueRef) -> Option<&Value> {
        match *value {
            ValueRef::Inst(id) => Some(self.inst(id)),
            ValueRef::Block(id) => Some(self.block(id)),
            ValueRef::Argument(id) => Some(self.argument(id)),
            _ => None,
        }
    }
}

impl<'tctx> UnitContext for FunctionContext<'tctx> {
    fn inst(&self, inst: InstRef) -> &Inst {
        self.function.body.inst(inst)
    }

    fn argument(&self, argument: ArgumentRef) -> &Argument {
        self.function
            .args
            .iter()
            .find(|x| argument == x.as_ref())
            .unwrap()
    }
}

impl<'tctx> SequentialContext for FunctionContext<'tctx> {
    fn block(&self, block: BlockRef) -> &Block {
        self.function.body.block(block)
    }
}
