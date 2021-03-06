use common::{halfway_house, prelude::*, rotation};
use nalgebra::UnitQuaternion;
use std::error::Error;

const PHYSICS_TPS: f32 = 120.0;

pub fn get_packet_and_inject_rigid_body_tick(
    rlbot: &rlbot::RLBot,
    rigid_body_tick: rlbot::flat::RigidBodyTick<'_>,
) -> Result<common::halfway_house::LiveDataPacket, Box<dyn Error>> {
    let packet = rlbot
        .interface()
        .update_live_data_packet_flatbuffer()
        .ok_or_else(|| Box::<dyn Error>::from("packet not returned"))?;
    let mut packet = halfway_house::deserialize_game_tick_packet(packet);
    physicsify(&mut packet, rigid_body_tick);
    Ok(packet)
}

pub fn physicsify(
    packet: &mut common::halfway_house::LiveDataPacket,
    physics: rlbot::flat::RigidBodyTick<'_>,
) {
    let ball = physics.ball().unwrap();
    let ball_state = ball.state().unwrap();
    packet.GameInfo.TimeSeconds = ball_state.frame() as f32 / PHYSICS_TPS;
    set_physics(&mut packet.GameBall.Physics, ball_state);
    for i in 0..packet.NumCars as usize {
        let player = physics.players().unwrap().get(i);
        set_physics(&mut packet.GameCars[i].Physics, player.state().unwrap());
    }
}

fn set_physics(dest: &mut common::halfway_house::Physics, source: rlbot::flat::RigidBodyState<'_>) {
    dest.Location = vector3(source.location().unwrap());
    dest.Rotation = rotator(source.rotation().unwrap());
    dest.Velocity = vector3(source.velocity().unwrap());
    dest.AngularVelocity = vector3(source.angularVelocity().unwrap());
}

fn vector3(v: &rlbot::flat::Vector3) -> common::halfway_house::Vector3 {
    common::halfway_house::Vector3 {
        X: v.x(),
        Y: v.y(),
        Z: v.z(),
    }
}

fn rotator(q: &rlbot::flat::Quaternion) -> common::halfway_house::Rotator {
    let quat = UnitQuaternion::xyzw(q.x(), q.y(), q.z(), q.w());
    let (pitch, yaw, roll) = rotation::convert_quat_to_pyr(&quat);
    assert!(!pitch.is_nan());
    common::halfway_house::Rotator {
        Pitch: pitch,
        Yaw: yaw,
        Roll: roll,
    }
}
