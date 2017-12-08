use sys;

/// CLIPS value
pub struct Value(pub(crate) sys::CLIPSValue);

impl Value {
    fn new(val: sys::clipsValue__bindgen_ty_1) -> Self {
        Value(sys::CLIPSValue {
            __bindgen_anon_1: val
        })
    }
}

use enum_primitive::FromPrimitive;

enum_from_primitive! {
/// Native CLIPS data types
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Float = sys::FLOAT_TYPE as isize,
    Integer = sys::INTEGER_TYPE as isize,
    Symbol = sys::SYMBOL_TYPE as isize,
    String = sys::STRING_TYPE as isize,
    Multifield = sys::MULTIFIELD_TYPE as isize,
    ExternalAddress = sys::EXTERNAL_ADDRESS_TYPE as isize,
    FactAddress = sys::FACT_ADDRESS_TYPE as isize,
    InstanceAddress = sys::INSTANCE_ADDRESS_TYPE as isize,
    InstanceName = sys::INSTANCE_NAME_TYPE as isize,
    Void = sys::VOID_TYPE as isize,
    Bitmap = sys::BITMAP_TYPE as isize,
}
}

impl Value {
    /// Value's type
    pub fn type_of(&self) -> Type {
        unsafe { Type::from_u16((*self.0.__bindgen_anon_1.header).type_).unwrap() }
    }
}

pub trait EnvAllocatable {
    fn allocate(&self, env: &super::Environment) -> Value;
}

macro_rules! impl_env_allocatable_for_integer {
    ($t: ty) => {
      impl EnvAllocatable for $t {

        fn allocate(&self, env: &super::Environment) -> Value {
            let int = unsafe {
                sys::CreateInteger(env.env, *self as i64)
            };
            Value::new(sys::clipsValue__bindgen_ty_1 {
               integerValue: int
            })
        }
      }
    }
}

impl_env_allocatable_for_integer!(u8);
impl_env_allocatable_for_integer!(i8);
impl_env_allocatable_for_integer!(u16);
impl_env_allocatable_for_integer!(i16);
impl_env_allocatable_for_integer!(u32);
impl_env_allocatable_for_integer!(i32);
impl_env_allocatable_for_integer!(u64);
impl_env_allocatable_for_integer!(i64);

macro_rules! impl_env_allocatable_for_float {
    ($t: ty) => {
      impl EnvAllocatable for $t {

        fn allocate(&self, env: &super::Environment) -> Value {
            let float = unsafe {
                sys::CreateFloat(env.env, *self as f64)
            };
            Value::new(sys::clipsValue__bindgen_ty_1 {
               floatValue: float
            })
        }
      }
    }
}

impl_env_allocatable_for_float!(f32);
impl_env_allocatable_for_float!(f64);

impl EnvAllocatable for str {

  fn allocate(&self, env: &super::Environment) -> Value {
      use std::ffi::CString;
      let c_str = CString::new(self).unwrap();
      let str = unsafe {
          sys::CreateString(env.env, c_str.as_ptr())
      };
      Value::new(sys::clipsValue__bindgen_ty_1 {
          lexemeValue: str
      })
  }
}

pub struct Symbol<S: AsRef<str>>(pub S);

impl<S: AsRef<str>> EnvAllocatable for Symbol<S> {

  fn allocate(&self, env: &super::Environment) -> Value {
      use std::ffi::CString;
      let c_str = CString::new(self.0.as_ref()).unwrap();
      let str = unsafe {
          sys::CreateSymbol(env.env, c_str.as_ptr())
      };
      Value::new(sys::clipsValue__bindgen_ty_1 {
          lexemeValue: str
      })
  }
}



#[cfg(test)]
mod tests {

    use super::*;
    use super::super::*;

    #[test]
    pub fn integer() {
        let env = Environment::new().unwrap();
        assert_eq!(0u8.allocate(&env).type_of(), Type::Integer);
        assert_eq!(0i8.allocate(&env).type_of(), Type::Integer);
        assert_eq!(0u16.allocate(&env).type_of(), Type::Integer);
        assert_eq!(0i16.allocate(&env).type_of(), Type::Integer);
        assert_eq!(0u32.allocate(&env).type_of(), Type::Integer);
        assert_eq!(0i32.allocate(&env).type_of(), Type::Integer);
        assert_eq!(0u64.allocate(&env).type_of(), Type::Integer);
        assert_eq!(0i64.allocate(&env).type_of(), Type::Integer);
    }

    #[test]
    pub fn float() {
        let env = Environment::new().unwrap();
        assert_eq!(0f32.allocate(&env).type_of(), Type::Float);
        assert_eq!(0f64.allocate(&env).type_of(), Type::Float);
    }

    #[test]
    pub fn string() {
        let env = Environment::new().unwrap();
        assert_eq!("test".allocate(&env).type_of(), Type::String);
    }

    #[test]
    pub fn symbol() {
        let env = Environment::new().unwrap();
        assert_eq!("test".allocate(&env).type_of(), Type::String);
    }


}