extern crate rollout;

// TODO: think about cleanup for redis db between tests
mod tests {
    use rollout::Flipper;

    #[test]
    fn it_works() {
        let f: Flipper = Flipper::new().expect("Couldn't create a flipper!?");
        let feature = "retweetable";
        let ident = "1240";

        assert_eq!(
            f.active(feature, ident).expect(
                "Could not check active status",
            ),
            false
        );

        f.activate(feature, ident).expect(
            "Could not activate feature",
        );

        assert_eq!(
            f.active(feature, ident).expect(
                "Could not check active status",
            ),
            true
        );

        f.deactivate(feature, ident).expect(
            "Could not deactivate feature",
        );

        assert_eq!(
            f.active(feature, ident).expect(
                "Could not check active status",
            ),
            false
        );
    }
}
