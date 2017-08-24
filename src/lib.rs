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
    pub fn new() -> Flipper {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();

        Flipper { connection: client.get_connection().unwrap() }
    }

    pub fn active(&self, feature: &str, ident: &str) -> bool {
        let key = format!("{}_{}", feature, ident);
        let val: Result<String, redis::RedisError> = self.connection.get(&key);
        match val {
            Ok(val) => val == "true",
            Err(_) => false,
        }
    }

    pub fn activate(&self, feature: &str, ident: &str) {
        let key = format!("{}_{}", feature, ident);
        let _: () = self.connection.set(&key, "true").unwrap();
    }

    pub fn deactivate(&self, feature: &str, ident: &str) {
        let key = format!("{}_{}", feature, ident);
        let _: () = self.connection.set(&key, "false").unwrap();
    }
}
