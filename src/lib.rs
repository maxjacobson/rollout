extern crate redis;
use redis::Commands;

// TODO: handle multiple users in the same rollout without overwriting data
// TODO: add percentage-based rollouts
// TODO: add group-based rollouts
// TODO: test that things work with multiple features
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
                println!("Received from database: {}", results);
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
        let _: () = self.connection.set("feature:__features__", feature)?;
        let value = format!("{}|{}||{}", "0", ident, "{}");
        println!("Setting value to: {}", value);

        let success: () = self.connection.set(format!("feature:{}", feature), value)?;

        Ok(success)
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
