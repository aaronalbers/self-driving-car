use common::{prelude::*, rl};
use nalgebra::{Point2, Point3};
use rlbot;

pub struct Game<'a> {
    packet: &'a rlbot::ffi::LiveDataPacket,
    pub team: Team,
    pub enemy_team: Team,
    boost_dollars: Box<[BoostPickup]>,
}

impl<'a> Game<'a> {
    pub fn new(
        field_info: &'a rlbot::ffi::FieldInfo,
        packet: &'a rlbot::ffi::LiveDataPacket,
    ) -> Self {
        Self {
            packet,
            team: Team::Blue,
            enemy_team: Team::Orange,
            boost_dollars: field_info
                .BoostPads
                .iter()
                .take(field_info.NumBoosts as usize)
                .filter(|info| info.FullBoost)
                .map(|info| BoostPickup {
                    loc: point3(info.Location).to_2d(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub fn enemy(&self) -> &rlbot::ffi::PlayerInfo {
        self.packet
            .cars()
            .find(|p| p.Team == self.enemy_team.to_ffi())
            .expect("No car found on enemy team")
    }

    pub fn own_goal(&self) -> &Goal {
        Goal::for_team(self.team)
    }

    pub fn enemy_goal(&self) -> &Goal {
        Goal::for_team(self.enemy_team)
    }

    pub fn boost_dollars(&self) -> &[BoostPickup] {
        &*self.boost_dollars
    }
}

fn point3(v: rlbot::ffi::Vector3) -> Point3<f32> {
    Point3::new(v.X, v.Y, v.Z)
}

#[derive(Copy, Clone)]
pub enum Team {
    Blue,
    Orange,
}

impl Team {
    fn to_ffi(&self) -> u8 {
        match self {
            Team::Blue => 0,
            Team::Orange => 1,
        }
    }
}

pub struct Goal {
    pub center_2d: Point2<f32>,
}

impl Goal {
    pub fn for_team(team: Team) -> &'static Self {
        match team {
            Team::Blue => &BLUE_GOAL,
            Team::Orange => &ORANGE_GOAL,
        }
    }

    pub fn ball_is_scored(&self, ball_loc: Point3<f32>) -> bool {
        // This is just an estimate, it doesn't take into account ball radius, etc.
        if self.center_2d.y < 0.0 {
            ball_loc.y < self.center_2d.y
        } else {
            ball_loc.y > self.center_2d.y
        }
    }
}

pub struct BoostPickup {
    pub loc: Point2<f32>,
}

lazy_static! {
    static ref BLUE_GOAL: Goal = Goal {
        center_2d: Point2::new(0.0, -rl::FIELD_MAX_Y)
    };
    static ref ORANGE_GOAL: Goal = Goal {
        center_2d: Point2::new(0.0, rl::FIELD_MAX_Y)
    };
    static ref BOOST_DOLLARS: Vec<BoostPickup> = vec![
        Point2::new(-3072.0, -4096.0),
        Point2::new(3072.0, -4096.0),
        Point2::new(-3584.0, 0.0),
        Point2::new(3584.0, 0.0),
        Point2::new(-3072.0, 4096.0),
        Point2::new(3072.0, 4096.0),
    ]
    .into_iter()
    .map(|loc| BoostPickup { loc })
    .collect();
}
