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

pub struct Flipper {
    connection: redis::Connection,
}

impl Flipper {
    pub fn new() -> Result<Flipper, redis::RedisError> {
        let client = redis::Client::open("redis://127.0.0.1/")?;

        Ok(Flipper { connection: client.get_connection()? })
    }

    pub fn active<T: std::hash::Hash + std::fmt::Display>(
        &self,
        feature: &str,
        ident: &T,
    ) -> Result<bool, redis::RedisError> {
        let data: Result<String, redis::RedisError> =
            self.connection.get(format!("feature:{}", feature));

        match data {
            Ok(results) => {
                let parts: Vec<_> = results.split("|").collect();
                let users = parts[1];
                let idents: Vec<_> = users.split(",").collect();
                let str_ident = format!("{}", ident);
                Ok(idents.contains(&str_ident.as_str()))
            }
            Err(e) => {
                match e.kind() {
                    redis::ErrorKind::TypeError => Ok(false),
                    _ => Err(e),
                }
            }
        }
    }

    pub fn activate<T: std::hash::Hash + std::fmt::Display>(
        &self,
        feature: &str,
        ident: &T,
    ) -> Result<(), redis::RedisError> {
        let mut list = self.all_features()?;
        if !list.contains(&feature.to_owned()) {
            list.push(feature.to_owned());
            let _: () = self.connection.set("feature:__features__", list.join(","))?;
        }

        let value = format!("{}|{}||{}", "0", ident, "{}");

        let success: () = self.connection.set(format!("feature:{}", feature), value)?;

        Ok(success)
    }

    pub fn all_features(&self) -> Result<Vec<String>, redis::RedisError> {
        let features: Result<String, redis::RedisError> =
            self.connection.get("feature:__features__");

        match features {
            Ok(csv_features) => {
                let mut features_to_return = vec![];
                let features_split: Vec<&str> = csv_features.split(",").collect();

                for feature in features_split {
                    features_to_return.push(feature.to_owned());
                }

                Ok(features_to_return)
            }
            Err(e) => {
                match e.kind() {
                    redis::ErrorKind::TypeError => Ok(vec![]),
                    _ => Err(e),
                }
            }
        }
    }

    pub fn deactivate<T: std::hash::Hash + std::fmt::Display>(
        &self,
        feature: &str,
        ident: &T,
    ) -> Result<(), redis::RedisError> {
        let success: () = self.connection.set(
            format!("feature:{}", feature),
            format!("{}|{}||{}", "0", "", "{}"),
        )?;

        Ok(success)
    }
}
