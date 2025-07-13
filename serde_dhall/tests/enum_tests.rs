mod test_enum {
    use serde::{Deserialize, Serialize};
    use serde_dhall::{SimpleType, StaticType};

    use std::collections::HashMap;

    #[derive(
        Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, StaticType,
    )]
    pub struct FrameUid {
        pub ephemeris_id: i32,
        pub orientation_id: i32,
    }

    #[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
    pub enum VectorExpr {
        Fixed {
            x: f64,
            y: f64,
            z: f64,
        }, // Unitless vector, for arbitrary computations
        Position {
            from_frame: FrameUid,
            to_frame: FrameUid,
        },
        Velocity {
            from_frame: FrameUid,
            to_frame: FrameUid,
        },
        CrossProduct {
            a: Box<VectorExpr>,
            b: Box<VectorExpr>,
        },
    }

    // Manual implementation of StaticType for the recursive Vector enum.
    #[allow(unconditional_recursion)]
    impl StaticType for VectorExpr {
        // This function defines the Dhall type that corresponds to our Rust type
        fn static_type() -> SimpleType {
            let mut fields = HashMap::new();
            fields.insert(
                "Fixed".to_string(),
                // The type for the `Fixed` variant is a record.
                Some(SimpleType::Record(
                    [
                        ("x".to_string(), SimpleType::Natural), // Using Natural as a stand-in for f64
                        ("y".to_string(), SimpleType::Natural),
                        ("z".to_string(), SimpleType::Natural),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                )),
            );
            fields.insert(
                "Position".to_string(),
                Some(SimpleType::Record(
                    [
                        ("from_frame".to_string(), FrameUid::static_type()),
                        ("to_frame".to_string(), FrameUid::static_type()),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                )),
            );
            fields.insert(
                "Velocity".to_string(),
                Some(SimpleType::Record(
                    [
                        ("from_frame".to_string(), FrameUid::static_type()),
                        ("to_frame".to_string(), FrameUid::static_type()),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                )),
            );
            fields.insert(
                "CrossProduct".to_string(),
                Some(SimpleType::Record(
                    [
                        ("a".to_string(), Self::static_type()),
                        ("b".to_string(), Self::static_type()),
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                )),
            );
            SimpleType::Union(fields)
        }
    }

    fn build_type() -> SimpleType {
        let ty: SimpleType = serde_dhall::from_str(
            r#"
let FrameUid = {ephemeris_id: Integer, orientation_id: Integer }

let VectorExpr =
        < Fixed : { x : Double, y : Double, z : Double }
        | Position : { from_frame : FrameUid, to_frame : FrameUid }
        | Velocity : { from_frame : FrameUid, to_frame : FrameUid }
        >       
        in VectorExpr"#,
        )
        .parse()
        .unwrap();
        ty
    }

    #[test]
    fn test_vector_expr_fixed() {
        let v = VectorExpr::Fixed {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let v_str = serde_dhall::serialize(&v)
            .type_annotation(&build_type())
            .to_string()
            .unwrap();
        println!("{v_str:?}");
        let v_deser: VectorExpr =
            serde_dhall::from_str(&v_str).parse().unwrap();
        assert_eq!(v_deser, v); // This fails because I am not serializing it correctly.
    }

    #[test]
    fn test_vector_expr_state() {
        let pos = VectorExpr::Position {
            from_frame: FrameUid {
                ephemeris_id: 399,
                orientation_id: 0,
            },
            to_frame: FrameUid {
                ephemeris_id: 301,
                orientation_id: 0,
            },
        };

        let pos_str = serde_dhall::serialize(&pos)
            .type_annotation(&build_type())
            .to_string()
            .unwrap();
        println!("{pos_str:?}");
        let v_deser: VectorExpr =
            serde_dhall::from_str(&pos_str).parse().unwrap();
        assert_eq!(v_deser, pos);
    }

    #[test]
    fn test_vector_expr_cross() {
        let pos = VectorExpr::Position {
            from_frame: FrameUid {
                ephemeris_id: 399,
                orientation_id: 0,
            },
            to_frame: FrameUid {
                ephemeris_id: 301,
                orientation_id: 0,
            },
        };

        let vel = VectorExpr::Velocity {
            from_frame: FrameUid {
                ephemeris_id: 399,
                orientation_id: 0,
            },
            to_frame: FrameUid {
                ephemeris_id: 301,
                orientation_id: 0,
            },
        };

        let h_vec = VectorExpr::CrossProduct {
            a: Box::new(pos),
            b: Box::new(vel),
        };

        let h_vec_str = serde_dhall::serialize(&h_vec)
            .type_annotation(&build_type())
            .to_string()
            .unwrap();
        println!("{h_vec_str:?}");
        let v_deser: VectorExpr =
            serde_dhall::from_str(&h_vec_str).parse().unwrap();
        assert_eq!(v_deser, h_vec);
    }
}
