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
    pub fn assert(self) -> Result<Fact<'a>, ()> {
        let fact_ptr = unsafe {
            sys::FBAssert(self.fb)
        };
        if fact_ptr.is_null() {
            Err(())
        } else {
            Ok(Fact(fact_ptr, self.env))
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

pub struct Fact<'a>(*mut sys::Fact, &'a Environment);

impl<'a> Fact<'a> {

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


    pub fn slot<S: AsRef<str>>(&self, name: S) -> Value {
        let mut val : Value = unsafe { ::std::mem::zeroed() };
        let c_string = CString::new(name.as_ref()).unwrap();
        unsafe {
            sys::FactSlotValue(self.1.env, self.0, c_string.as_ptr(), &mut val.0)
        }
        val
    }

}

impl<'a> EnvAllocatable for Fact<'a> {
    fn allocate(&self, _env: &super::Environment) -> Value {
        Value::new(sys::clipsValue__bindgen_ty_1 {
            factValue: self.0
      })
    }
}

pub struct Iter<'a> {
    env: &'a Environment,
    ptr: *mut sys::Fact,
    end: bool,
}

impl<'a> Iter<'a> {
    pub fn new(env: &'a Environment) -> Self {
        Iter {
            env,
            ptr: ::std::ptr::null_mut(),
            end: false,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Fact<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        self.ptr = unsafe {
            sys::GetNextFact(self.env.env, self.ptr)
        };
        if self.ptr.is_null() {
            self.end = true;
            None
        } else {
            Some(Fact(self.ptr, self.env))
        }
    }
}

pub struct TemplateIter<'a> {
    env: &'a Environment,
    ptr: *mut sys::Fact,
    template: *mut sys::Deftemplate,
    end: bool,
}

impl<'a> TemplateIter<'a> {
    fn new(env: &'a Environment, template: *mut sys::Deftemplate) -> Self {
        TemplateIter {
            env,
            ptr: ::std::ptr::null_mut(),
            template,
            end: false,
        }
    }
}

impl<'a> Iterator for TemplateIter<'a> {
    type Item = Fact<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        self.ptr = unsafe {
            sys::GetNextFactInTemplate(self.template, self.ptr)
        };
        if self.ptr.is_null() {
            self.end = true;
            None
        } else {
            Some(Fact(self.ptr, self.env))
        }
    }
}

/// Represents a template (deftemplate)
pub struct Template<'a> {
    pub(crate) env: &'a Environment,
    pub(crate) template: *mut sys::Deftemplate,
}

impl<'a> Template<'a> {

    /// Returns an iterator over facts with this template
    pub fn fact_iter(&self) -> TemplateIter {
        TemplateIter::new(self.env, self.template)
    }

    /// Returns an iterator over facts with this template,
    /// consuming the template itself.
    ///
    /// This is useful when the iterator needs to be
    /// carried around independently of the template
    pub fn into_fact_iter(self) -> TemplateIter<'a> {
        TemplateIter::new(self.env, self.template)
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

    #[test]
    fn access_fact_slot_value() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate f1 (slot a) (slot b))
        "#).unwrap();
        assert_eq!(env.number_of_facts(), 0);
        let fb = env.new_fact_builder("f1");
        fb.put("a", 1).unwrap();
        fb.put("b", "a").unwrap();
        let fact = fb.assert().unwrap();
        let val = fact.slot("a");
        assert_eq!(val.type_of(), Type::Integer);
        assert_eq!((ValueAccess::value(&val) as Option<i64>).unwrap(), 1);
        let val = fact.slot("b");
        assert_eq!(val.type_of(), Type::String);
        assert_eq!((ValueAccess::value(&val) as Option<&str>).unwrap(), "a");
    }

    #[test]
    fn fact_iterator() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate f1 (slot a) (slot b))
        "#).unwrap();
        let fb = env.new_fact_builder("f1");
        fb.put("a", 1).unwrap();
        fb.put("b", "a").unwrap();
        fb.assert().unwrap();
        let mut facts : Vec<Fact> = env.fact_iter().collect();
        assert_eq!(facts.len(), 1);
        let fact = facts.pop().unwrap();
        let val = fact.slot("a");
        assert_eq!(val.type_of(), Type::Integer);
        assert_eq!((ValueAccess::value(&val) as Option<i64>).unwrap(), 1);
        let val = fact.slot("b");
        assert_eq!(val.type_of(), Type::String);
        assert_eq!((ValueAccess::value(&val) as Option<&str>).unwrap(), "a");
    }

    #[test]
    fn template_fact_iterator() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate f1 (slot a) (slot b))
        (deftemplate f2 (slot a) (slot b))
        "#).unwrap();
        let fb = env.new_fact_builder("f1");
        fb.put("a", 1).unwrap();
        fb.put("b", "a").unwrap();
        fb.assert().unwrap();
        let fb = env.new_fact_builder("f2");
        fb.put("a", 1).unwrap();
        fb.put("b", "a").unwrap();
        fb.assert().unwrap();
        let template = env.find_template("f1").unwrap();
        let mut facts : Vec<Fact> = template.fact_iter().collect();
        assert_eq!(facts.len(), 1);
        let fact = facts.pop().unwrap();
        let val = fact.slot("a");
        assert_eq!(val.type_of(), Type::Integer);
        assert_eq!((ValueAccess::value(&val) as Option<i64>).unwrap(), 1);
        let val = fact.slot("b");
        assert_eq!(val.type_of(), Type::String);
        assert_eq!((ValueAccess::value(&val) as Option<&str>).unwrap(), "a");
    }


    #[test]
    fn template_consuming_fact_iterator() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate f1 (slot a) (slot b))
        (deftemplate f2 (slot a) (slot b))
        "#).unwrap();
        let fb = env.new_fact_builder("f1");
        fb.put("a", 1).unwrap();
        fb.put("b", "a").unwrap();
        fb.assert().unwrap();
        let fb = env.new_fact_builder("f2");
        fb.put("a", 1).unwrap();
        fb.put("b", "a").unwrap();
        fb.assert().unwrap();
        let template = env.find_template("f1").unwrap();
        let facts : Vec<Fact> = template.into_fact_iter().collect();
        assert_eq!(facts.len(), 1);
    }

}
