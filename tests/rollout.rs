extern crate rollout;

// TODO: think about cleanup for redis db between tests
mod tests {
    use rollout::Flipper;

    #[test]
    fn it_works() {
        let f: Flipper = Flipper::new();
        let feature = "retweetable";
        let ident = "1240";

        assert_eq!(f.active(feature, ident), false);

        f.activate(feature, ident);

        assert_eq!(f.active(feature, ident), true);

        f.deactivate(feature, ident);

        assert_eq!(f.active(feature, ident), false);
    }
}
