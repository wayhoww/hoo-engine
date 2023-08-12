#[cfg(test)]
mod tests {
    use crate::{ObjectId, RcObject};

    #[test]
    pub fn static_assert() {
        assert_eq!(
            std::mem::size_of::<ObjectId>(),
            std::mem::size_of::<usize>()
        );
        assert_eq!(
            std::mem::size_of::<ObjectId>(),
            std::mem::size_of::<*mut i32>()
        );
    }

    #[test]
    fn equality() {
        // also: hash.
        let v1_p1 = RcObject::new(1i32);
        let v1_p2 = v1_p1.clone();
        let v1_p3 = v1_p2.clone().into_any();
        let v1_p4 = v1_p3.clone().try_downcast::<i32>().unwrap();

        let v2_p1 = RcObject::new(1);

        assert_ne!(v1_p1, v2_p1);

        assert_eq!(v1_p1, v1_p2);
        assert_eq!(v1_p1, v1_p4);
        assert_eq!(v1_p2, v1_p4);

        assert_eq!(v1_p1.id(), v1_p2.id());
        assert_eq!(v1_p1.id(), v1_p3.id());
        assert_eq!(v1_p1.id(), v1_p4.id());
        assert_eq!(v1_p2.id(), v1_p3.id());
        assert_eq!(v1_p2.id(), v1_p4.id());
        assert_eq!(v1_p3.id(), v1_p4.id());
    }

    #[test]
    fn downcast() {
        struct S;

        let s = RcObject::new(S);
        let t = s.clone().into_any();
        let back1 = t.clone().try_downcast::<S>();
        let back2 = t.try_downcast::<i32>();

        assert!(back1.is_ok());
        assert!(back2.is_err());
    }
}
