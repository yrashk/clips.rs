extern crate clips;
#[macro_use] extern crate clips_derive;

use clips::fact::Assertable;

#[derive(clips_fact)]
#[clips(template="fact")]
struct Fact {
    test: String,
    #[clips(return_type="clone")]
    test1: String,
    i0: i64,
}

#[test]
fn test_trait_getter() {
  let fact = Fact {
      test: String::from("Hello"),
      test1: String::from("Hi"),
      i0: 0,
  };
  assert_eq!(fact.test(), "Hello");
}

#[test]
fn asserting() {
    let env = clips::Environment::new().unwrap();

    let fact = Fact {
      test: String::from("Hello"),
      test1: String::from("a"),
      i0: 0,
    };

    env.load_string("(deftemplate fact (slot test) (slot test1) (slot i0))").unwrap();

    let f = fact.assert(&env).unwrap();
    assert_eq!(f.test(), fact.test());
    assert_eq!(f.test1(), fact.test1());
    assert_eq!(f.i0(), fact.i0());
}


#[derive(clips_fact)]
#[clips(template="ref")]
struct Ref {
    test: &'static str,
}

#[test]
fn ref_slot() {
    let r = Ref {
        test: "test",
    };
    assert_eq!(r.test(), "test");
}

#[derive(clips_fact)]
#[clips(template="tpl")]
struct NonConsumable {}

#[test]
fn non_consumable_assert() {
    let non_consumable = NonConsumable {};
    let env = clips::Environment::new().unwrap();

    env.load_string("(deftemplate tpl)").unwrap();

    non_consumable.assert(&env).unwrap();
    non_consumable.assert(&env).unwrap();
}

#[derive(clips_fact)]
#[clips(template="tpl",consume_on_assert)]
struct Consumable {}

#[test]
fn consumable_assert() {
    let consumable = Consumable {};
    let env = clips::Environment::new().unwrap();

    env.load_string("(deftemplate tpl)").unwrap();

    consumable.assert(&env).unwrap();
    // not sure how to test it, but asserting consumable again
    // won't compile (as the value has moved)
}

use clips::fact::Recoverable;

#[derive(Debug, PartialEq, Clone, clips_fact)]
#[clips(template="tpl")]
struct Rec {
    value: String,
    value1: i8,
}

#[test]
fn recoverable() {
    let rec = Rec {
        value: String::from("hello"),
        value1: 1,
    };
    let rec1 = rec.clone();

    let env = clips::Environment::new().unwrap();
    env.load_string("(deftemplate tpl (slot value) (slot value1))").unwrap();

    let f = rec.assert(&env).unwrap();
    let rec2 = f.recover();
    assert_eq!(rec2, rec1);
}