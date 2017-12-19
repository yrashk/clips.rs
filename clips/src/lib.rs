extern crate clips_sys as sys;
#[macro_use] extern crate enum_primitive;
#[macro_use] extern crate derive_error;
#[cfg(test)] extern crate tempfile;

pub mod value;
pub use value::{Type, Symbol, Value, ValueAccess};

pub mod fact;
pub use fact::{Fact, FactBuilder, Template};

use std::ffi::CString;

/// CLIPS environment. Vast majority of APIs is only
/// available through an environment
pub struct Environment {
    pub(crate) env: *mut ::sys::environmentData,
}

use enum_primitive::FromPrimitive;

enum_from_primitive! {
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum LoadError {
    OpenFileError = sys::LoadError::LE_OPEN_FILE_ERROR as isize,
    ParsingError = sys::LoadError::LE_PARSING_ERROR as isize,
}
}

enum_from_primitive! {
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum EvalError {
    ParsingError = sys::EvalError::EE_PARSING_ERROR as isize,
    ProcessingError = sys::EvalError::EE_PROCESSING_ERROR as isize,
}
}


use std::path::Path;

impl Environment {

    /// Creates a new environment and initializes it
    pub fn new() -> Result<Self, ()> {
        let env = unsafe { sys::CreateEnvironment() };
        if env == ::std::ptr::null_mut() {
            Err(())
        } else {
            Ok(Environment {
                env,
            })
        }
    }

    /// Allows an expression to be evaluated
    pub fn eval<S: AsRef<str>>(&self, expr: S) -> Result<Value, EvalError> {
        let c_string = CString::new(expr.as_ref()).unwrap();
        let mut val : Value = unsafe { ::std::mem::zeroed() };
        let return_code = unsafe {
            sys::Eval(self.env, c_string.as_ptr(), &mut val.0)
        };
        match return_code {
            sys::EvalError::EE_NO_ERROR => Ok(val),
            err => Err(EvalError::from_isize(err as isize).expect("valid return code")),
        }
    }

    /// Loads a set of constructs into the CLIPS data base (the equivalent
    /// of the CLIPS load command).
    pub fn load<P: AsRef<Path>>(&self, file: P) -> Result<(), LoadError> {
        let c_string = CString::new(file.as_ref().to_str().unwrap()).unwrap();
        let return_code = unsafe {
            sys::Load(self.env, c_string.as_ptr())
        };
        match return_code {
            sys::LoadError::LE_NO_ERROR => Ok(()),
            err => Err(LoadError::from_isize(err as isize).expect("valid return code")),
        }
    }

    /// Loads a set of constructs into the CLIPS database from a memory-based
    /// source (as opposed to an existing file)
    pub fn load_string<S: AsRef<str>>(&self, str: S) -> Result<(), ()> {
        let c_string = CString::new(str.as_ref()).unwrap();
        let success = unsafe {
           sys::LoadFromString(self.env, c_string.as_ptr(), str.as_ref().as_bytes().len())
        };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// The main entry point for building new facts. Takes template name
    pub fn new_fact_builder<S: AsRef<str>>(&self, template: S) -> FactBuilder {
        FactBuilder::new(self, template)
    }

    /// Returns the number of asserted facts
    pub fn number_of_facts(&self) -> usize {
        unsafe {
            sys::GetNumberOfFacts(self.env) as usize
        }
    }

    /// Returns an iterator over all asserted facts
    pub fn fact_iter(&self) -> fact::Iter {
        fact::Iter::new(self)
    }

    /// Finds an iterator (if there's one by the given name)
    pub fn find_template<S: AsRef<str>>(&self, template: S) -> Option<Template> {
        let c_string = CString::new(template.as_ref()).unwrap();
        let deftemplate =unsafe {
            sys::FindDeftemplate(self.env, c_string.as_ptr())
        };
        if deftemplate.is_null() {
            None
        } else {
            Some(Template { env: self, template: deftemplate })
        }
    }

    /// Starts execution of rules.
    /// It is the equivalent of the CLIPS `run` command
    ///
    /// If `limit` is `Some(n)`, the execution will cease
    /// after `n` number of rule firings or when the agenda
    /// contains no rule activations.
    pub fn run(&self, limit: Option<usize>) -> usize{
        let limit = match limit {
            None => -1,
            Some(n) => n as isize,
        };
        unsafe {
            sys::Run(self.env, limit as i64) as usize
        }
    }

}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe { sys::DestroyEnvironment(self.env); }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sanity_check() {
        assert_eq!(Environment::new().unwrap().eval("\"a\"").unwrap().type_of(),
                   Type::String);
    }

    use tempfile;
    use std::io::Write;

    #[test]
    fn load() {
        let env = Environment::new().unwrap();
        let content = r#"
        (deffunction test () 1)
        "#;

        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write(content.as_bytes()).unwrap();

        env.load(file.path()).unwrap();

        assert_eq!(env.eval("(test)").unwrap().type_of(), Type::Integer);
    }

    #[test]
    fn load_file_error() {
        let env = Environment::new().unwrap();
        assert_eq!(env.load(Path::new("no_such_file")).unwrap_err(), LoadError::OpenFileError);
    }

    #[test]
    fn load_loading_error() {
        let env = Environment::new().unwrap();
        let content = r#"
        (deffunction test () 1
        "#;

        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write(content.as_bytes()).unwrap();

        assert_eq!(env.load(file.path()).unwrap_err(), LoadError::ParsingError);
    }

    #[test]
    fn load_string() {
        let env = Environment::new().unwrap();
        let content = r#"
        (deffunction test () 1)
        "#;

        env.load_string(content).unwrap();
        assert_eq!(env.eval("(test)").unwrap().type_of(), Type::Integer);
    }

    #[test]
    fn run_empty() {
        let env = Environment::new().unwrap();
        assert_eq!(env.run(None), 0);
    }

    #[test]
    fn run_some() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate tpl1)
        (defrule rule1 (tpl1) => )
        "#).unwrap();
        let fb = env.new_fact_builder("tpl1");
        fb.assert().unwrap();
        assert_eq!(env.run(None), 1);
    }

    #[test]
    fn run_limit() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate tpl1)
        (defrule rule1 (tpl1) => )
        "#).unwrap();
        let fb = env.new_fact_builder("tpl1");
        fb.assert().unwrap();
        assert_eq!(env.run(Some(0)), 0);
    }

    #[test]
    fn find_template() {
        let env = Environment::new().unwrap();
        env.load_string(r#"
        (deftemplate tpl1)
        "#).unwrap();
        assert!(env.find_template("tpl1").is_some());
        assert!(env.find_template("tpl2").is_none());
    }
}
