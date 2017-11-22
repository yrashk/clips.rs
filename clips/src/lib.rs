extern crate clips_sys as sys;
#[macro_use] extern crate enum_primitive;


use std::ffi::CString;

/// This structure holds a native CLIPS data object
pub struct DataObject {
    object: sys::DATA_OBJECT,
}

use enum_primitive::FromPrimitive;
enum_from_primitive! {
/// Native CLIPS data types
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
  Float = 0,
  Integer = 1,
  Symbol = 2,
  String = 3,
  Multifield = 4,
  ExternalAddress = 5,
  FactAddress = 6,
  InstanceAddress = 7,
  InstanceName = 8,
}
}

impl DataObject {

    /// Returns data object's type
    pub fn data_type(&self) -> Type {
        Type::from_u16(self.object.type_).unwrap()
    }
}

/// CLIPS environment. Vast majority of APIs is only
/// available through an environment
pub struct Environment {
    env: *mut ::std::os::raw::c_void,
}

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
    pub fn eval<S: AsRef<str>>(&self, expr: S) -> Result<DataObject, ()> {
        let c_string = CString::new(expr.as_ref()).unwrap();
        let mut data_object : DataObject = unsafe { ::std::mem::zeroed() };
        let return_code = unsafe { sys::EnvEval(self.env, c_string.as_ptr(), &mut data_object.object) };
        match return_code {
            1 => Ok(data_object),
            0 => Err(()),
            err => panic!("unexpected return code {}", err),
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
        assert_eq!(Environment::new().unwrap().eval("\"a\"").unwrap().data_type(),
                   Type::String);
    }
}