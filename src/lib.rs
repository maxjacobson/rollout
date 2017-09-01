extern crate redis;
use redis::Commands;

// TODO: deactivate just should remove one ident, not all of 'em
// TODO: handle multiple users in the same rollout without overwriting data
// TODO: add percentage-based rollouts
// TODO: add group-based rollouts
// TODO: consider maintaining state in memory rather than re-querying for it
//       each time
// TODO: rename ident
// TODO: consider simplifying the ident constraints


// TODO: make some trait to allow more flexibility in key and value types?
pub trait Store {
    fn write(&self, key: String, value: String) -> Result<(), StoreError>;

    fn read(&self, key: String) -> Result<Option<String>, StoreError>;
}

#[derive(Debug)]
pub enum StoreError {
    RedisError(redis::RedisError),
    MiscError,
}

pub struct Flipper<S: Store> {
    pub store: S,
}

impl Store for redis::Connection {
    fn write(&self, key: String, value: String) -> Result<(), StoreError> {
        let result: Result<(), redis::RedisError> = self.set(key, value);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(StoreError::RedisError(e)),
        }
    }

    fn read(&self, key: String) -> Result<Option<String>, StoreError> {
        let result: Result<String, redis::RedisError> = self.get(key);

        match result {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                match e.kind() {
                    redis::ErrorKind::TypeError => Ok(None),
                    _ => Err(StoreError::RedisError(e)),
                }
            }
        }
    }
}

impl<S: Store> Flipper<S> {
    pub fn active<T: std::hash::Hash + std::fmt::Display>(
        &self,
        feature: &str,
        ident: &T,
    ) -> Result<bool, StoreError> {
        // TODO: use ?
        let data: Result<Option<String>, StoreError> =
            self.store.read(format!("feature:{}", feature));

        match data {
            Ok(Some(results)) => {
                let parts: Vec<_> = results.split("|").collect();
                let users = parts[1];
                let idents: Vec<_> = users.split(",").collect();
                let str_ident = format!("{}", ident);
                Ok(idents.contains(&str_ident.as_str()))
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn activate<T: std::hash::Hash + std::fmt::Display>(
        &self,
        feature: &str,
        ident: &T,
    ) -> Result<(), StoreError> {
        let mut list = self.all_features()?;
        if !list.contains(&feature.to_owned()) {
            list.push(feature.to_owned());
            let _: () = self.store.write(
                "feature:__features__".to_owned(),
                list.join(","),
            )?;
        }

        let value = format!("{}|{}||{}", "0", ident, "{}");

        let success: () = self.store.write(format!("feature:{}", feature), value)?;

        Ok(success)
    }

    pub fn all_features(&self) -> Result<Vec<String>, StoreError> {
        let features: Option<String> = self.store.read("feature:__features__".to_owned())?;

        match features {
            Some(csv_features) => {
                let mut features_to_return = vec![];
                let features_split: Vec<&str> = csv_features.split(",").collect();

                for feature in features_split {
                    features_to_return.push(feature.to_owned());
                }

                Ok(features_to_return)
            }

            None => Ok(vec![]),
        }
    }

    pub fn deactivate<T: std::hash::Hash + std::fmt::Display>(
        &self,
        feature: &str,
        ident: &T,
    ) -> Result<(), StoreError> {
        let success: () = self.store.write(
            format!("feature:{}", feature),
            format!("{}|{}||{}", "0", "", "{}"),
        )?;

        Ok(success)
    }
}
