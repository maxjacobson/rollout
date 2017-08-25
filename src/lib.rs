extern crate redis;
use redis::Commands;

// TODO: add percentage-based rollouts
// TODO: add namespacing??
// TODO: consider whether we're modeling the data in any kind of consistent way
//       with the ruby library we're porting

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
        let key = format!("{}_{}", feature, ident);
        let result: Result<String, redis::RedisError> = self.connection.get(&key);
        match result {
            Ok(value) => Ok(value == "true"), // TODO: validate that it is either true or false?
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
        let key = format!("{}_{}", feature, ident);
        let success: () = self.connection.set(&key, "true")?;
        Ok(success)
    }

    pub fn deactivate<T: std::hash::Hash + std::fmt::Display>(
        &self,
        feature: &str,
        ident: &T,
    ) -> Result<(), redis::RedisError> {
        let key = format!("{}_{}", feature, ident);
        let success: () = self.connection.set(&key, "false")?;
        Ok(success)
    }
}
