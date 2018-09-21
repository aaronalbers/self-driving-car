use behavior::{Action, Behavior};
use collect::ExtendRotation3;
use eeg::{color, Drawable, EEG};
use maneuvers::GetToFlatGround;
use mechanics::{simple_yaw_diff, GroundAccelToLoc, QuickJumpAndDodge};
use nalgebra::Vector3;
use predict::estimate_intercept_car_ball_2;
use rlbot;
use simulate::rl;
use std::f32::consts::PI;
use utils::{
    enemy_goal_center, enemy_goal_left_post, enemy_goal_right_post, one_v_one, ExtendF32,
    ExtendPhysics, ExtendVector2, ExtendVector3,
};

pub struct GroundShot {
    min_distance: Option<f32>,
    finished: bool,
}

impl GroundShot {
    pub const MAX_BALL_Z: f32 = 120.0; // sloppy number

    pub fn new() -> GroundShot {
        GroundShot {
            min_distance: None,
            finished: false,
        }
    }

    pub fn good_angle(ball_loc: Vector3<f32>, car_loc: Vector3<f32>) -> bool {
        let angle_me_ball = car_loc.to_2d().angle_to(ball_loc.to_2d());
        let angle_ball_goal = ball_loc.to_2d().angle_to(enemy_goal_center());
        let goodness = (angle_me_ball - angle_ball_goal).normalize_angle().abs();
        goodness < 60.0_f32.to_radians()
    }
}

impl Behavior for GroundShot {
    fn name(&self) -> &'static str {
        stringify!(GroundShot)
    }

    fn execute(&mut self, packet: &rlbot::LiveDataPacket, eeg: &mut EEG) -> Action {
        if self.finished {
            return Action::Return;
        }

        let (me, _enemy) = one_v_one(packet);
        let intercept = estimate_intercept_car_ball_2(&me, &packet.GameBall, |t, &loc, vel| {
            loc.z < 110.0 && Self::good_angle(loc, me.Physics.loc())
        });

        if !Self::good_angle(intercept.ball_loc, me.Physics.loc()) {
            eeg.log(format!("Bad angle from {:?}", intercept.ball_loc));
            return Action::Return;
        }

        let target_loc = intercept.ball_loc.to_2d()
            + (intercept.ball_loc.to_2d() - enemy_goal_center()).normalize() * 150.0;
        let target_dist = (target_loc - me.Physics.loc().to_2d()).norm();

        // If the ball has moved further away, assume we hit it and we're done.
        match self.min_distance {
            Some(md) if target_dist >= md * 2.0 => return Action::Return,
            _ => self.min_distance = Some(target_dist),
        }

        eeg.draw(Drawable::print(
            format!("intercept_time: {:.2}", intercept.time),
            color::GREEN,
        ));
        eeg.draw(Drawable::print(
            format!("target_dist: {:.0}", target_dist),
            color::GREEN,
        ));
        eeg.draw(Drawable::GhostBall(intercept.ball_loc));

        // This behavior currently just operates in 2D
        if !GetToFlatGround::on_flat_ground(packet) {
            // TODO: this is not how this works…
            return Action::call(GetToFlatGround::new());
        }

        if target_dist <= 250.0 {
            self.finished = true;
            return shoot(packet, eeg);
        }

        // TODO: this is not how this works…
        let mut child =
            GroundAccelToLoc::new(target_loc, packet.GameInfo.TimeSeconds + intercept.time);
        child.execute(packet, eeg)
    }
}

fn shoot(packet: &rlbot::LiveDataPacket, eeg: &mut EEG) -> Action {
    let (me, _enemy) = one_v_one(packet);
    let angle = simple_yaw_diff(&me.Physics, packet.GameBall.Physics.loc().to_2d());
    if angle.abs() >= PI / 2.0 {
        eeg.log("Incorrect approach angle");
        return Action::Return;
    }

    return Action::call(QuickJumpAndDodge::begin(packet).yaw(angle));
}

#[cfg(test)]
mod integration_tests {
    use behavior::{runner::PUSHED, RootBehavior};
    use collect::ExtendRotation3;
    use integration_tests::helpers::{TestRunner, TestScenario};
    use maneuvers::{BounceShot, GroundShot};
    use nalgebra::{Rotation3, Vector3};
    use strategy::Runner2;

    #[test]
    #[ignore] // TODO
    fn crossing_the_midfield() {
        let test = TestRunner::start(
            RootBehavior::new(),
            TestScenario {
                ball_loc: Vector3::new(-1794.4557, -681.9332, 99.93823),
                ball_vel: Vector3::new(-619.51764, 1485.6294, -12.806913),
                car_loc: Vector3::new(-3472.8125, -1983.225, 16.937647),
                car_rot: Rotation3::from_unreal_angles(-0.009779127, 2.4388378, -0.0011504856),
                car_vel: Vector3::new(-1599.1952, 1223.4504, 9.51471),
                ..Default::default()
            },
        );

        test.sleep_millis(4000);
        test.examine_eeg(|eeg| {
            assert!(
                eeg.log
                    .iter()
                    .any(|x| *x == format!("{} GroundShot", PUSHED))
            );
        });
        assert!(test.has_scored());
    }

    #[test]
    #[ignore] // TODO
    fn crossing_the_box() {
        let test = TestRunner::start(
            RootBehavior::new(),
            TestScenario {
                ball_loc: Vector3::new(-726.1142, -673.77716, 118.28892),
                ball_vel: Vector3::new(1032.4805, 1531.884, -72.43818),
                car_loc: Vector3::new(-45.566628, -1993.5394, 16.711021),
                car_rot: Rotation3::from_unreal_angles(-0.010258497, 0.60458016, 0.0013422332),
                car_vel: Vector3::new(1566.5747, 1017.1486, 13.497895),
                ..Default::default()
            },
        );

        test.sleep_millis(3000);
        test.examine_eeg(|eeg| {
            assert!(
                eeg.log
                    .iter()
                    .any(|x| *x == format!("{} GroundShot", PUSHED))
            );
        });
        assert!(test.has_scored());
    }

    #[test]
    #[ignore] // TODO
    fn high_bouncer() {
        let test = TestRunner::start(
            RootBehavior::new(),
            TestScenario {
                ball_loc: Vector3::new(-1725.8822, 4719.4307, 93.15),
                ball_vel: Vector3::new(1031.4242, 2151.6794, 0.0),
                car_loc: Vector3::new(-2374.2222, 3805.5469, 17.01),
                car_rot: Rotation3::from_unreal_angles(-0.009970875, 1.0610354, -0.0002876214),
                car_vel: Vector3::new(521.8343, 928.79755, 8.326952),
                ..Default::default()
            },
        );

        test.sleep_millis(4000);
        assert!(test.has_scored());
    }

    #[test]
    #[ignore] // TODO
    fn easy_open_net() {
        let test = TestRunner::start(
            GroundShot::new(),
            TestScenario {
                ball_loc: Vector3::new(999.651, 3636.9731, 93.14),
                ball_vel: Vector3::new(-271.7422, -1642.4099, 0.0),
                car_loc: Vector3::new(1981.3068, -3343.5154, 16.99),
                car_rot: Rotation3::from_unreal_angles(-0.00958738, 1.9184347, 0.0),
                car_vel: Vector3::new(-544.83453, 1537.2355, 8.53),
                boost: 0,
                ..Default::default()
            },
        );

        test.sleep_millis(4000);
        assert!(test.has_scored());
    }

    #[test]
    #[ignore] // TODO
    fn tight_angle_needs_correction() {
        let test = TestRunner::start0(TestScenario {
            ball_loc: Vector3::new(-2618.1267, 4567.453, 93.14),
            ball_vel: Vector3::new(204.82155, -438.9531, 0.0),
            car_loc: Vector3::new(-3850.746, 3749.8147, 16.319502),
            car_rot: Rotation3::from_unreal_angles(-0.15867114, -0.33191508, 0.005273059),
            car_vel: Vector3::new(1287.4675, -433.82834, -183.28568),
            ..Default::default()
        });
        test.set_behavior(Runner2::new());

        test.sleep_millis(2000);
        assert!(test.has_scored());
    }
}
