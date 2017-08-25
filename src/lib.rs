extern crate redis;
use redis::Commands;

// TODO: figure out an error handling strategy
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

    pub fn active(&self, feature: &str, ident: &str) -> Result<bool, redis::RedisError> {
        let key = format!("{}_{}", feature, ident);
        let result: Result<String, redis::RedisError> = self.connection.get(&key);
        Ok(result? == "true")
    }

    pub fn activate(&self, feature: &str, ident: &str) -> Result<(), redis::RedisError> {
        let key = format!("{}_{}", feature, ident);
        let success: () = self.connection.set(&key, "true")?;
        Ok(success)
    }

    pub fn deactivate(&self, feature: &str, ident: &str) -> Result<(), redis::RedisError> {
        let key = format!("{}_{}", feature, ident);
        let success: () = self.connection.set(&key, "false")?;
        Ok(success)
    }
}
