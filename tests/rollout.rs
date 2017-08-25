extern crate rollout;

// TODO: think about cleanup for redis db between tests
mod tests {
    use std;
    use rollout::Flipper;

    fn test_for_ident<T: std::hash::Hash + std::fmt::Display>(ident: &T) {
        let f: Flipper = Flipper::new().expect("Couldn't create a flipper!?");
        let feature = "retweetable";

        assert_eq!(
            f.active(feature, ident).expect(
                "Could not check active status (first time)",
            ),
            false
        );

        f.activate(feature, ident).expect(
            "Could not activate feature",
        );

        assert_eq!(
            f.active(feature, ident).expect(
                "Could not check active status (second time)",
            ),
            true
        );

        f.deactivate(feature, ident).expect(
            "Could not deactivate feature",
        );

        assert_eq!(
            f.active(feature, ident).expect(
                "Could not check active status (third time)",
            ),
            false
        );
    }

    #[test]
    fn it_works() {
        let ident = "1240";
        test_for_ident(&ident);
    }

    #[test]
    fn it_allows_other_kinds_of_idents() {
        let ident = 4444;
        test_for_ident(&ident);
    }
}
