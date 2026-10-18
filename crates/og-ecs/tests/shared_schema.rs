//! Two engine modules may each declare the same reflected record (og-physics
//! and og-anim both declare `og.Transform`): the second Rust type of the
//! same name and layout shares the first's schema, so both read and write
//! the same rows; a type of the same name and another layout is refused.

use og_core::ErrorKind;
use og_core::math::Vec3;
use og_ecs::{Reflect, World, WorldDesc};

mod physics {
    use super::*;
    #[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
    #[reflect(name = "test.Place")]
    pub struct Place {
        pub position: Vec3,
        pub turn: f32,
    }
}

mod anim {
    use super::*;
    #[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
    #[reflect(name = "test.Place")]
    pub struct Place {
        pub position: Vec3,
        pub turn: f32,
    }
}

mod other {
    use super::*;
    #[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
    #[reflect(name = "test.Place")]
    pub struct Place {
        pub position: Vec3,
    }
}

#[test]
fn a_second_rust_type_of_the_same_layout_shares_the_rows() {
    let mut w = World::new(&WorldDesc::default()).unwrap();
    let a = w.register::<physics::Place>().unwrap();
    let b = w.register::<anim::Place>().unwrap();
    assert_eq!(a, b);
    let e = w
        .spawn((physics::Place {
            position: Vec3::new(1.0, 2.0, 3.0),
            turn: 0.5,
        },))
        .unwrap();
    // Written through one type, read through the other: the same row.
    assert_eq!(
        w.get::<anim::Place>(e).unwrap().position,
        Vec3::new(1.0, 2.0, 3.0)
    );
    w.insert(
        e,
        anim::Place {
            position: Vec3::new(4.0, 5.0, 6.0),
            turn: 1.0,
        },
    )
    .unwrap();
    assert_eq!(w.get::<physics::Place>(e).unwrap().turn, 1.0);
    let err = w.register::<other::Place>().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Schema);
}
