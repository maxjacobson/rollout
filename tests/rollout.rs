extern crate redis;
extern crate rollout;

// TODO: think about cleanup for redis db between tests
// TODO: research what is the conventional way to organize a test suite with
//       before hooks? I'm looking at libraries like stainless and shiny. I'm
//       curious which has caught on...
//  TODO: make tests work in parallel by using a different redis db for each
//        test (would need to be able to inject the redis URL via a parameter)
mod tests {
    use std;
    use rollout::Flipper;
    use redis;

    fn cleanup<T: std::fmt::Display>(tag: T) {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let connection = client.get_connection().unwrap();

        println!("[{}] Flushing db...", tag);
        redis::cmd("FLUSHDB").execute(&connection);
    }

    fn test_for_ident<T: std::hash::Hash + std::fmt::Display>(feature: &str, ident: &T) {
        let f: Flipper = Flipper::new().expect("Couldn't create a flipper!?");

        let other_feature = &"shared_feature_in_both_tests";
        let other_ident = &"4567";

        let no_features_yet: Vec<String> = vec![];
        assert_eq!(f.all_features().unwrap(), no_features_yet);
        assert_eq!(f.active(feature, ident).unwrap(), false);
        assert_eq!(f.all_features().unwrap(), no_features_yet);
        assert_eq!(f.active(other_feature, other_ident).unwrap(), false);
        assert_eq!(f.all_features().unwrap(), no_features_yet);

        f.activate(feature, ident).unwrap();

        assert_eq!(f.all_features().unwrap(), vec![feature]);
        f.activate(other_feature, other_ident).unwrap();
        assert_eq!(f.all_features().unwrap(), vec![feature, other_feature]);

        assert_eq!(f.active(feature, ident).unwrap(), true);
        assert_eq!(f.active(other_feature, other_ident).unwrap(), true);

        f.deactivate(feature, ident).unwrap();
        f.deactivate(other_feature, other_ident).unwrap();

        assert_eq!(f.active(feature, ident).unwrap(), false);
        assert_eq!(f.active(other_feature, other_ident).unwrap(), false);

        assert_eq!(f.all_features().unwrap(), vec![feature, other_feature]);
    }

    #[test]
    fn it_works_for_string_features() {
        cleanup("string");

        let feature = "string_feature";
        let ident = "1240";
        test_for_ident(&feature, &ident);
    }

    #[test]
    fn it_works_for_int_features() {
        cleanup("int");

        let feature = "int_feature";
        let ident = 4444;
        test_for_ident(&feature, &ident);
    }
}
