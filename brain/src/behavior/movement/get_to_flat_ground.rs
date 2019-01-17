use crate::{
    behavior::{
        higher_order::Chain,
        movement::{dodge::Dodge, drive_towards::drive_towards, land::Land, yielder::Yielder},
    },
    eeg::{color, Drawable},
    strategy::{Action, Behavior, Context, Priority},
};
use common::prelude::*;
use nalgebra::Vector3;
use nameof::name_of_type;
use simulate::linear_interpolate;
use std::f32::consts::PI;

pub struct GetToFlatGround;

impl GetToFlatGround {
    pub fn new() -> GetToFlatGround {
        GetToFlatGround
    }

    pub fn on_flat_ground(car: &rlbot::ffi::PlayerInfo) -> bool {
        car.OnGround
            && car.Physics.rot().pitch().abs() < 15.0_f32.to_radians()
            && car.Physics.rot().roll().abs() < 15.0_f32.to_radians()
    }
}

impl Behavior for GetToFlatGround {
    fn name(&self) -> &str {
        name_of_type!(GetToFlatGround)
    }

    fn execute_old(&mut self, ctx: &mut Context<'_>) -> Action {
        if Self::on_flat_ground(ctx.me()) {
            return Action::Return;
        }

        let me = ctx.me();

        if !me.OnGround {
            return Action::tail_call(Land::new());
        }

        if me.Physics.roof_axis().angle(&-Vector3::z_axis()) < PI / 10.0 {
            // We're probably upside down under the ceiling of a goal
            ctx.eeg.log(self.name(), "jumping while upside-down");
            return Action::tail_call(Yielder::new(
                rlbot::ffi::PlayerInput {
                    Jump: true,
                    ..Default::default()
                },
                0.1,
            ));
        }

        let backup_cutoff = linear_interpolate(
            &[0.0, 2000.0],
            &[PI / 4.0, PI / 6.0],
            me.Physics.vel().dot(&me.Physics.forward_axis()),
        );
        if me.Physics.forward_axis().angle(&Vector3::z_axis()) < backup_cutoff {
            // Our nose is pointed towards the sky. It's quicker to jump down than to drive.
            ctx.eeg
                .draw(Drawable::print("nose pointed upwards", color::GREEN));
            return backup_down_the_wall(ctx);
        }

        ctx.eeg
            .draw(Drawable::print("driving down the wall", color::GREEN));
        let target_loc =
            (me.Physics.loc() + me.Physics.rot() * Vector3::new(500.0, 0.0, 250.0)).to_2d();
        ctx.eeg
            .draw(Drawable::ghost_car_ground(target_loc, me.Physics.rot()));
        Action::Yield(drive_towards(ctx, target_loc))
    }
}

fn backup_down_the_wall(ctx: &mut Context<'_>) -> Action {
    let me = ctx.me();

    if me.Physics.vel().z >= 0.0 || me.Physics.loc().z >= 1000.0 {
        // Phase one of the reverse dismount: back up so we don't jump into the sky
        ctx.eeg.draw(Drawable::print("backing up", color::GREEN));
        return Action::Yield(rlbot::ffi::PlayerInput {
            Throttle: -1.0,
            ..Default::default()
        });
    } else {
        // Phase two of the reverse dismount: jump. Eventually we'll make our way to
        // `Land` and we'll land on our wheels.
        ctx.eeg
            .log(name_of_type!(GetToFlatGround), "jumping off the wall");
        let mut inputs = Vec::<Box<dyn Behavior>>::with_capacity(3);

        inputs.push(Box::new(Yielder::new(
            rlbot::ffi::PlayerInput {
                Pitch: 1.0,
                Jump: true,
                ..Default::default()
            },
            0.2,
        )));
        inputs.push(Box::new(Yielder::new(
            rlbot::ffi::PlayerInput {
                Pitch: 1.0,
                Jump: false,
                ..Default::default()
            },
            0.1,
        )));

        if Land::defensiveness(ctx) < 0.5 {
            // We're probably way out of the game. Dodge towards our goal to get back to
            // defense quicker.
            ctx.eeg.log(
                name_of_type!(GetToFlatGround),
                "... and dodging to get back quick",
            );

            let forward = me.Physics.forward_axis().unwrap();
            let roof = me.Physics.roof_axis().unwrap();
            // The roof is meant to add some instability so we don't end up with weird flips
            // that somehow put us exactly vertical.
            let angle = (forward + roof / 3.0)
                .to_2d()
                .rotation_to(&(ctx.game.own_back_wall_center() - ctx.me().Physics.loc_2d()));

            inputs.push(Box::new(Dodge::new().angle(angle)));
        }

        return Action::tail_call(Chain::new(Priority::Idle, inputs));
    }
}
