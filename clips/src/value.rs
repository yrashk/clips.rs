use sys;
use std::ffi::{CStr, CString};

/// CLIPS value
pub struct Value(pub(crate) sys::CLIPSValue);

impl Value {
    pub(crate) fn new(val: sys::clipsValue__bindgen_ty_1) -> Self {
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

/// Allows accessing typed values inside of `Value`,
/// parametrized over resulting type `T`
pub trait ValueAccess : Sized {
    /// Returns `Some(value)` if the type is compatible,
    /// otherwise `None`
    fn value(val: &Value) -> Option<Self>;
}

macro_rules! value_access_for_int {
    ($ty: ty) => {
    impl ValueAccess for $ty {
        fn value(val: &Value) -> Option<$ty> {
            match val.type_of() {
                Type::Integer => unsafe { Some((*val.0.__bindgen_anon_1.integerValue).contents as $ty) },
                _ => None,
            }
        }
    }
    };
}

value_access_for_int!(i8);
value_access_for_int!(i16);
value_access_for_int!(i32);
value_access_for_int!(i64);
value_access_for_int!(u8);
value_access_for_int!(u16);
value_access_for_int!(u32);
value_access_for_int!(u64);

macro_rules! value_access_for_float {
    ($ty: ty) => {
    impl ValueAccess for $ty {
        fn value(val: &Value) -> Option<$ty> {
            match val.type_of() {
                Type::Float => unsafe { Some((*val.0.__bindgen_anon_1.floatValue).contents as $ty) },
                _ => None,
            }
        }
    }
    };
}

value_access_for_float!(f64);
value_access_for_float!(f32);

impl<'a> ValueAccess for &'a str {
    fn value(val: &Value) -> Option<&'a str> {
        match val.type_of() {
            Type::String => {
                let str = unsafe { (*val.0.__bindgen_anon_1.lexemeValue).contents };
                let cstr = unsafe { CStr::from_ptr(str) };
                Some(cstr.to_str().unwrap())
            },
            _ => None,
        }
    }
}

impl ValueAccess for String {
    fn value(val: &Value) -> Option<String> {
        let val: Option<&str> = ValueAccess::value(val);
        val.and_then(|v| Some(String::from(v))).or(None)
    }
}

impl<'a> ValueAccess for Symbol<&'a str> {
    fn value(val: &Value) -> Option<Symbol<&'a str>> {
        match val.type_of() {
            Type::Symbol => {
                let str = unsafe { (*val.0.__bindgen_anon_1.lexemeValue).contents };
                let cstr = unsafe { CStr::from_ptr(str) };
                Some(Symbol(cstr.to_str().unwrap()))
            },
            _ => None,
        }
    }
}

impl ValueAccess for bool {
    fn value(val: &Value) -> Option<bool> {
        let symbol = Symbol::<&str>::value(&val);
        symbol.and_then(|s| match s {
            Symbol("TRUE") => Some(true),
            Symbol("FALSE") => Some(false),
            _ => None,
        })
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


impl<'a> EnvAllocatable for String {
    fn allocate(&self, env: &super::Environment) -> Value {
        self.as_str().allocate(env)
    }
}

impl<'a, 'b> EnvAllocatable for &'b String {
    fn allocate(&self, env: &super::Environment) -> Value {
        self.as_str().allocate(env)
    }
}

impl<'a> EnvAllocatable for &'a str {

  fn allocate(&self, env: &super::Environment) -> Value {
      let c_str = CString::new(*self).unwrap();
      let str = unsafe {
          sys::CreateString(env.env, c_str.as_ptr())
      };
      Value::new(sys::clipsValue__bindgen_ty_1 {
          lexemeValue: str
      })
  }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Symbol<S: AsRef<str>>(pub S);

impl<S: AsRef<str>> EnvAllocatable for Symbol<S> {

  fn allocate(&self, env: &super::Environment) -> Value {
     let c_str = CString::new(self.0.as_ref()).unwrap();
      let str = unsafe {
          sys::CreateSymbol(env.env, c_str.as_ptr())
      };
      Value::new(sys::clipsValue__bindgen_ty_1 {
          lexemeValue: str
      })
  }
}

impl EnvAllocatable for bool {

    fn allocate(&self, env: &super::Environment) -> Value {
        let c_str = match self {
            &true => CString::new("TRUE").unwrap(),
            &false => CString::new("FALSE").unwrap(),
        };
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
    pub fn integer_value_access() {
        let env = Environment::new().unwrap();
        let val = 1u8.allocate(&env);
        let access = i64::value(&val).unwrap();
        assert_eq!(access, 1);
    }

    #[test]
    pub fn float() {
        let env = Environment::new().unwrap();
        assert_eq!(0f32.allocate(&env).type_of(), Type::Float);
        assert_eq!(0f64.allocate(&env).type_of(), Type::Float);
    }

    #[test]
    pub fn float_value_access() {
        let env = Environment::new().unwrap();
        let val = 1f32.allocate(&env);
        let access = f64::value(&val).unwrap();
        assert_eq!(access, 1f64);
    }

    #[test]
    pub fn string() {
        let env = Environment::new().unwrap();
        assert_eq!("test".allocate(&env).type_of(), Type::String);
    }

    #[test]
    pub fn string_value_access() {
        let env = Environment::new().unwrap();
        let val = "test".allocate(&env);
        let access: &str = ValueAccess::value(&val).unwrap();
        assert_eq!(access, "test");
    }

    #[test]
    pub fn symbol() {
        let env = Environment::new().unwrap();
        assert_eq!(Symbol("test").allocate(&env).type_of(), Type::Symbol);
    }

    #[test]
    pub fn symbol_value_access() {
        let env = Environment::new().unwrap();
        let val = Symbol("test").allocate(&env);
        let access = Symbol::<&str>::value(&val).unwrap();
        assert_eq!(access, Symbol("test"));
    }

    #[test]
    pub fn boolean() {
        let env = Environment::new().unwrap();
        assert_eq!(true.allocate(&env).type_of(), Type::Symbol);
        assert_eq!(false.allocate(&env).type_of(), Type::Symbol);
    }

    #[test]
    pub fn bool_value_access() {
        let env = Environment::new().unwrap();
        let val = true.allocate(&env);
        let access = bool::value(&val).unwrap();
        assert!(access);
        // invalid symbols lead to None
        let val = Symbol("something").allocate(&env);
        let access = bool::value(&val);
        assert!(access.is_none());
    }


}