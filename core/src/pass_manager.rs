use core::fmt;
use std::{cell::RefCell, fmt::Write, rc::Rc};

use dyn_clone::DynClone;
use thiserror::Error;

use crate::{builtin::ModuleOp, utils, OpRef};

#[derive(Error, Debug)]
pub enum PassError {
    #[error("No pass registered with name `{0}`")]
    UnknownPass(String),
    #[error("Unexpected op type, expected `{0}`, got `{1}`")]
    UnexpectedOpType(String, String),
}

pub trait PassWrapper: DynClone + Sync + Send {
    fn run(&self, op: &OpRef) -> Result<(), PassError>;
    fn get_wrapper_name(&self) -> &'static str;
    fn get_pass_name(&self) -> &'static str;
}

/// PassManager holds an optimization pipeline.
///
/// An optimization pipeline is defined as a series of passes that are iteratively applied to input
/// operation. Each optimization is run exactly once (unless you add a particular pass multiple
/// times).
#[derive(Default)]
pub struct PassManager {
    passes: Vec<Box<dyn PassWrapper>>,
}

impl PassManager {
    /// Creates a new empty optimization pipeline
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a pass manager with pre-populated passes
    /// from a Pass Registry
    pub fn new_from_list<S: AsRef<str> + ToString>(passes: &[S]) -> Result<Self, PassError> {
        let mut pm = Self::new();

        for p in passes {
            let wrapper =
                find_pass_by_name(p.as_ref()).ok_or(PassError::UnknownPass(p.to_string()))?;
            pm.add_any_pass(wrapper);
        }

        Ok(pm)
    }

    /// Adds a new registered pass to the end of the pipeline
    pub fn add_pass(&mut self, name: &str) -> Result<(), PassError> {
        let wrapper = find_pass_by_name(name).ok_or(PassError::UnknownPass(name.to_string()))?;
        self.add_any_pass(wrapper);
        Ok(())
    }

    fn add_any_pass(&mut self, pass: Box<dyn PassWrapper>) {
        self.passes.push(pass)
    }

    /// Optimizes IR inside regions of a particular operation
    pub fn run(&self, op: &OpRef) -> Result<(), PassError> {
        for pass in &self.passes {
            pass.run(&op)?;
        }
        Ok(())
    }
}

impl fmt::Debug for PassManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for p in &self.passes {
            f.write_char('[')?;
            f.write_str(p.get_wrapper_name())?;
            f.write_str("] ")?;
            f.write_str(p.get_pass_name())?;
            f.write_char('\n')?;
        }

        Ok(())
    }
}

type ModulePassFn = fn(op: &Rc<RefCell<ModuleOp>>) -> Result<(), PassError>;

#[derive(Debug, Clone)]
pub struct ModulePassWrapper {
    pass: ModulePassFn,
    name: &'static str,
}

impl ModulePassWrapper {
    pub fn new(name: &'static str, pass: ModulePassFn) -> Self {
        ModulePassWrapper { pass, name }
    }
}

impl PassWrapper for ModulePassWrapper {
    fn get_wrapper_name(&self) -> &'static str {
        "ModulePass"
    }

    fn get_pass_name(&self) -> &'static str {
        self.name
    }

    fn run(&self, op: &OpRef) -> Result<(), PassError> {
        let cast = match utils::op_cast::<ModuleOp>(op.clone()) {
            Some(op) => op,
            None => {
                return Err(PassError::UnexpectedOpType(
                    "module".to_string(),
                    op.borrow().get_operation_name().to_string(),
                ))
            }
        };

        (self.pass)(&cast)
    }
}

pub struct PassRegistryEntry {
    wrapper: Box<dyn PassWrapper>,
}

impl PassRegistryEntry {
    pub const fn new(wrapper: Box<dyn PassWrapper>) -> Self {
        Self { wrapper }
    }
}

#[linkme::distributed_slice]
pub static TIR_PASS_REGISTRY: [once_cell::sync::Lazy<PassRegistryEntry>];

fn find_pass_by_name(name: &str) -> Option<Box<dyn PassWrapper>> {
    TIR_PASS_REGISTRY.iter().find_map(|pe| {
        if name == pe.wrapper.get_pass_name() {
            Some(dyn_clone::clone_box(&*pe.wrapper))
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::builtin::{ConstOp, VoidType};
    use crate::{self as tir_core, Attr, Context, OpRef};
    use crate::{builtin::ModuleOp, PassError, PassManager};

    #[tir_macros::pass(name = "test-pass", wrapper = super::ModulePassWrapper)]
    fn test_pass(_op: &Rc<RefCell<ModuleOp>>) -> Result<(), PassError> {
        Ok(())
    }

    #[test]
    fn passes_can_be_added() {
        let mut pm = PassManager::new();
        pm.add_pass("test-pass").expect("failed to add pass");
        assert!(pm.add_pass("unkn-pass").is_err());

        let passes = format!("{:?}", pm);

        assert!(passes.find("test-pass").is_some());

        let context = Context::new();
        let module: OpRef = ModuleOp::builder(&context).build();

        assert!(pm.run(&module).is_ok());

        let attr = Attr::I8(16);
        let ret_type = VoidType::build(context.clone());
        let constant: OpRef = ConstOp::builder(&context)
            .value(attr.clone())
            .return_type(ret_type.clone().into())
            .build();

        assert!(pm.run(&constant).is_err());
    }

    #[test]
    fn passes_from_list() {
        let pm = PassManager::new_from_list(&["test-pass"]).expect("failed to create PM");
        let passes = format!("{:?}", pm);
        assert!(passes.find("test-pass").is_some());
    }
}
