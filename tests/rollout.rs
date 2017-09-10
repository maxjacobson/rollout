extern crate rollout;

// TODO: decide if we want to have _any_ tests that actually hit redis. Maybe
//       that would be the integration test I was imagining which tests the
//       compatibility with the rubygem by actually using it

mod tests {
    use std;
    use rollout::{Flipper, Store, StoreError};
    use std::collections::HashMap;
    use std::cell::RefCell;

    struct FakeDatabase {
        pairs: HashMap<String, String>,
    }

    impl FakeDatabase {
        fn new() -> FakeDatabase {
            FakeDatabase { pairs: HashMap::new() }
        }

        fn insert(&mut self, key: String, value: String) {
            self.pairs.insert(key, value);
        }

        fn get(&self, key: String) -> Option<String> {
            match self.pairs.get(key.as_str()) {
                Some(value) => Some(value.to_owned()),
                None => None,
            }
        }
    }

    struct FakeStore {
        // TODO: can this just be a RefCell<HashMap<String, String>>? I added
        //       the layer of indirection without fully confirming that I needed
        //       it
        database: RefCell<FakeDatabase>,
    }

    impl FakeStore {
        fn new() -> FakeStore {
            let database = FakeDatabase::new();

            FakeStore { database: RefCell::new(database) }
        }
    }

    impl Store for FakeStore {
        fn write(&self, key: String, value: String) -> Result<(), StoreError> {
            let mut db = self.database.borrow_mut();
            db.insert(key, value);

            Ok(())
        }

        fn read(&self, key: String) -> Result<Option<String>, StoreError> {
            match self.database.borrow().get(key) {
                Some(value) => Ok(Some(value.to_owned())),
                None => Ok(None),
            }
        }
    }

    fn test_for_id<T: std::fmt::Display>(feature: &str, id: &T) {
        let store = FakeStore::new();

        let f = Flipper { store: store };

        let other_feature = &"shared_feature_in_both_tests";
        let other_id = &"4567";

        let no_features_yet: Vec<String> = vec![];
        assert_eq!(f.all_features().unwrap(), no_features_yet);
        assert_eq!(f.active(feature, id).unwrap(), false);
        assert_eq!(f.active(other_feature, other_id).unwrap(), false);
        assert_eq!(f.active(feature, other_id).unwrap(), false);

        f.activate(feature, id).unwrap();
        f.activate(feature, other_id).unwrap();

        assert_eq!(f.all_features().unwrap(), vec![feature]);
        f.activate(other_feature, other_id).unwrap();
        assert_eq!(f.all_features().unwrap(), vec![feature, other_feature]);

        assert_eq!(f.active(feature, id).unwrap(), true);
        assert_eq!(f.active(feature, other_id).unwrap(), true);
        assert_eq!(f.active(other_feature, other_id).unwrap(), true);

        f.deactivate(feature, id).unwrap();
        f.deactivate(other_feature, other_id).unwrap();

        assert_eq!(f.active(feature, id).unwrap(), false);
        assert_eq!(f.active(other_feature, other_id).unwrap(), false);
        assert_eq!(f.active(feature, other_id).unwrap(), true);

        assert_eq!(f.all_features().unwrap(), vec![feature, other_feature]);
    }

    #[test]
    fn it_works_for_string_features() {
        let feature = "string_feature";
        let id = "1240";
        test_for_id(&feature, &id);
    }

    #[test]
    fn it_works_for_int_features() {
        let feature = "int_feature";
        let id = 4444;
        test_for_id(&feature, &id);
    }
}
