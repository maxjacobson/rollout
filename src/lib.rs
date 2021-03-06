extern crate redis;
use redis::Commands;

// Feature completeness:
// TODO: add percentage-based rollouts
// TODO: add group-based rollouts
// TODO: what's the deal with the 3rd position in the string? It seems to go: pct|ids|(blank)|group
//       but that's kinds weird, right?
//
// Implementation ideas:
// TODO: consider maintaining state in memory rather than re-querying for it
//       each time
//
// Code cleanliness:
// TODO: resolve duplication across serializing/deserializing feature data

pub trait Store {
    fn write(&self, key: String, value: String) -> Result<(), StoreError>;

    fn read(&self, key: String) -> Result<Option<String>, StoreError>;
}

#[derive(Debug)]
pub enum StoreError {
    RedisError(redis::RedisError),
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
    pub fn active<T: std::fmt::Display>(&self, feature: &str, id: &T) -> Result<bool, StoreError> {
        let data: Option<String> = self.store.read(format!("feature:{}", feature))?;

        match data {
            Some(results) => {
                let parts: Vec<_> = results.split("|").collect();
                let users = parts[1];
                let ids: Vec<_> = users.split(",").collect();
                let str_id = format!("{}", id);
                Ok(ids.contains(&str_id.as_str()))
            }

            None => Ok(false),
        }
    }

    pub fn activate<T: std::fmt::Display>(&self, feature: &str, id: &T) -> Result<(), StoreError> {
        let mut list = self.all_features()?;
        if !list.contains(&feature.to_owned()) {
            list.push(feature.to_owned());
            let _: () = self.store.write(
                "feature:__features__".to_owned(),
                list.join(","),
            )?;
        }

        let id_string = format!("{}", id);
        let currently_active_for_feature: Option<String> =
            self.store.read(format!("feature:{}", feature))?;

        let new_feature_data = if let Some(results) = currently_active_for_feature {
            let parts: Vec<_> = results.split("|").collect();
            let pct = parts[0];
            let users = parts[1];
            let groups = parts[3];
            let mut ids: Vec<_> = users.split(",").collect();
            let id_str = id_string.as_str();
            if !ids.contains(&id_str) {
                ids.push(&id_str);
            }

            format!("{}|{}||{}", pct, ids.join(","), groups)
        } else {
            format!("{}|{}||{}", "0", id_string, "{}")
        };


        let success: () = self.store.write(
            format!("feature:{}", feature),
            new_feature_data,
        )?;

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

    pub fn deactivate<T: std::fmt::Display>(
        &self,
        feature: &str,
        id: &T,
    ) -> Result<(), StoreError> {
        let existing_data = self.store.read(format!("feature:{}", feature))?;

        match existing_data {
            Some(results) => {
                let parts: Vec<_> = results.split("|").collect();
                let pct = parts[0];
                let users = parts[1];
                let groups = parts[3];
                let ids: Vec<_> = users.split(",").collect();
                let str_id = format!("{}", id);
                let mut new_ids = Vec::new();
                for existing_id in ids {
                    if existing_id != str_id {
                        new_ids.push(existing_id);
                    }
                }

                let success: () = self.store.write(
                    format!("feature:{}", feature),
                    format!(
                        "{}|{}||{}",
                        pct,
                        new_ids.join(","),
                        groups
                    ),
                )?;

                Ok(success)
            }

            None => Ok(()),
        }
    }
}
