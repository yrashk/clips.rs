use super::Environment;
use super::value::{Value, EnvAllocatable};
use sys;

/// Template-based fact builder
pub struct FactBuilder<'a> {
    env: &'a Environment,
    fb: *mut sys::FactBuilder,
}

use std::ffi::CString;

/// Allows building facts from templates
impl<'a> FactBuilder<'a> {

    /// Creates a new fact builder. Not available publicly,
    /// should be accessed through `Environment`
    pub(crate) fn new<S: AsRef<str>>(env: &'a Environment, template: S) -> Self {
        let template_c_string = CString::new(template.as_ref()).unwrap();
        let fb = unsafe {
            sys::CreateFactBuilder(env.env, template_c_string.as_ptr())
        };
        FactBuilder {
            env, fb,
        }
    }

    /// Put a slot into a fact
    pub fn put<S: AsRef<str>, V: EnvAllocatable>(&self, slot: S, value: V) -> Result<(), sys::PutSlotError> {
        let slot_c_string = CString::new(slot.as_ref()).unwrap();
        let result =
        unsafe {
            sys::FBPutSlot(self.fb, slot_c_string.as_ptr(),
                           &value.allocate(self.env) as *const _ as *mut _)
        };
        match result {
            sys::PutSlotError::PSE_NO_ERROR => Ok(()),
            err => Err(err)
        }
    }

    /// Assume the fact, consuming the builder. Returns a result with
    /// the asserted fact.
    pub fn assert(self) -> Result<Fact, ()> {
        let fact_ptr = unsafe {
            sys::FBAssert(self.fb)
        };
        if fact_ptr.is_null() {
            Err(())
        } else {
            Ok(Fact(fact_ptr))
        }
    }

    /// Abort fact building
    pub fn abort(self) {
        unsafe {
            sys::FBAbort(self.fb)
        }
    }
}

impl<'a> Drop for FactBuilder<'a> {
    fn drop(&mut self) {
        unsafe {
            sys::FBDispose(self.fb)
        }
    }
}

pub struct Fact(*mut sys::Fact);

impl Fact {

    /// Fact index
    pub fn index(&self) -> u64 {
        unsafe {
            sys::FactIndex(self.0) as u64
        }
    }

    /// Retract the fact, consuming it
    pub fn retract(self) -> Result<(), sys::RetractError> {
        let result = unsafe {
            sys::Retract(self.0)
        };
        match result {
            sys::RetractError::RE_NO_ERROR => Ok(()),
            err => Err(err),
        }
    }

}

impl EnvAllocatable for Fact {
    fn allocate(&self, _env: &super::Environment) -> Value {
        Value::new(sys::clipsValue__bindgen_ty_1 {
            factValue: self.0
      })
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn assert() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate f1 (slot a) (slot b))
        "#).unwrap();
        assert_eq!(env.number_of_facts(), 0);
        let fb = env.new_fact_builder("f1");
        fb.put("a", 1).unwrap();
        fb.put("b", "a").unwrap();
        fb.assert().unwrap();
        assert_eq!(env.number_of_facts(), 1);
    }


    #[test]
    fn fact_slot() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate f1 (slot a) (slot b))
        "#).unwrap();
        assert_eq!(env.number_of_facts(), 0);
        let fb = env.new_fact_builder("f1");
        fb.put("a", 1).unwrap();
        fb.put("b", "a").unwrap();
        let fact = fb.assert().unwrap();
        assert_eq!(env.number_of_facts(), 1);
        let fb = env.new_fact_builder("f1");
        fb.put("a", 1).unwrap();
        fb.put("b", fact).unwrap();
        fb.assert().unwrap();
        assert_eq!(env.number_of_facts(), 2);
    }

    #[test]
    fn retract() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate f1 (slot a) (slot b))
        "#).unwrap();
        assert_eq!(env.number_of_facts(), 0);
        let fb = env.new_fact_builder("f1");
        fb.put("a", 1).unwrap();
        fb.put("b", "a").unwrap();
        let fact = fb.assert().unwrap();
        assert_eq!(env.number_of_facts(), 1);
        fact.retract().unwrap();
        assert_eq!(env.number_of_facts(), 0);
    }

}
