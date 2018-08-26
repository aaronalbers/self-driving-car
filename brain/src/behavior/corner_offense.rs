use behavior::{Action, Behavior};
use eeg::{color, Drawable, EEG};
use mechanics::{simple_steer_towards, GroundAccelToLoc, QuickJumpAndDodge};
use predict::intercept::estimate_intercept_car_ball;
use rlbot;
use utils::{enemy_goal_center, one_v_one, ExtendPhysics};

pub struct CornerOffense;

impl CornerOffense {
    pub fn new() -> CornerOffense {
        CornerOffense
    }
}

impl Behavior for CornerOffense {
    fn execute(&mut self, packet: &rlbot::LiveDataPacket, eeg: &mut EEG) -> Action {
        Action::Yield(Default::default())
    }
}

#[cfg(test)]
mod integration_tests {
    use behavior::CornerOffense;
    use collect::ExtendRotation3;
    use integration_tests::helpers::{TestRunner, TestScenario};
    use nalgebra::{Rotation3, Vector3};

    #[test]
    #[ignore]
    fn little_pop() {
        let test = TestRunner::start(
            CornerOffense::new(),
            TestScenario {
                ball_loc: Vector3::new(2935.073, 3494.3137, 93.65),
                ball_vel: Vector3::new(-805.1208, 831.31323, 1.0785761),
                car_loc: Vector3::new(3706.8596, 3888.5242, 126.83613),
                car_rot: Rotation3::from_unreal_angles(0.39174035, 0.24965537, -2.3729725),
                car_vel: Vector3::new(436.61905, 1689.2134, 17.572618),
                ..Default::default()
            },
        );

        test.sleep_millis(5000);

        assert!(test.has_scored());
    }
}
