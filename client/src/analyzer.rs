use crate::geom::*;
use common::models::{GameState, PlayerState, BulletState, BULLET_SPEED};
use std::{collections::HashMap, time::{Instant, Duration}};

pub struct Analyzer {
    own_player_id: u32,
    players: HashMap<u32, Player>,
    bullets: Vec<Bullet>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            own_player_id: 0,
            players: HashMap::new(),
            bullets: Vec::new(),
        }
    }

    pub fn push_state(&mut self, state: &GameState, time: Instant) {
        let mut players = HashMap::new();
        for player_state in state.players.iter() {
            let player = if let Some(mut prev_player) = self.players.remove(&player_state.id) {
                prev_player.push_state(&player_state, &state.scoreboard, time);
                prev_player
            } else {
                Player::with_state(&player_state, &state.scoreboard, time)
            };
            players.insert(player.id, player);
        }
        self.players = players;

        self.bullets = state
            .bullets
            .iter()
            .map(|state| Bullet::new(&state))
            .collect();
    }

    pub fn player<'a>(&'a self, id: u32) -> &'a Player {
        self.players.get(&id).unwrap()
    }

    pub fn set_own_player_id(&mut self, id: u32) {
        self.own_player_id = id;
    }

    pub fn own_player<'a>(&'a self) -> &'a Player {
        self.player(self.own_player_id)
    }

    pub fn angle_to(&self, target: u32) -> Radian {
        self.own_player()
            .position
            .angle_to(&self.player(target).position)
    }

    pub fn bullets_to_collide(&self, _until: Instant) -> Vec<Bullet> {
        // TODO
        unimplemented!();
    }
}

pub struct Player {
    pub id: u32,
    pub angle: Radian,
    pub position: Point,
    pub trajectory: Trajectory,
    pub score_history: ScoreHistory,
    last_update: Instant,
}

impl Player {
    pub fn new() -> Self {
        Self {
            id: 0,
            angle: Radian::zero(),
            position: Point::zero(),
            trajectory: Trajectory::new(),
            score_history: ScoreHistory::new(),
            last_update: Instant::now(),
        }
    }

    pub fn with_state(state: &PlayerState, scoreboard: &HashMap<u32, u32>, time: Instant) -> Self {
        let position = Point::new(state.x, state.y);

        let mut trajectory = Trajectory::new();
        trajectory.push(position.clone(), time);

        let mut score_history = ScoreHistory::new();
        score_history.push(*scoreboard.get(&state.id).unwrap(), time);

        Self {
            id: state.id,
            angle: Radian::new(state.angle),
            position,
            trajectory,
            score_history,
            last_update: time,
        }
    }

    pub fn push_state(
        &mut self,
        state: &PlayerState,
        scoreboard: &HashMap<u32, u32>,
        time: Instant,
    ) {
        assert_eq!(self.id, state.id);

        self.angle = Radian::new(state.angle);
        self.position = Point::new(state.x, state.y);
        self.trajectory.push(self.position.clone(), time);
        self.score_history
            .push(*scoreboard.get(&state.id).unwrap(), time);
        self.last_update = time;
    }
}

pub struct Trajectory {
    pub positions: Vec<(Point, Instant)>,
}

impl Trajectory {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
        }
    }

    pub fn push(&mut self, position: Point, time: Instant) {
        self.positions.push((position, time));
    }

    pub fn last_position<'a>(&'a self) -> &'a Point {
        &self.positions.last().unwrap().0
    }

    pub fn last_velocity(&self) -> Vector {
        let (last_position, last_time) = self.positions.last().unwrap();
        if let Some((prev_position, prev_time)) = self.positions.get(self.positions.len() - 2) {
            prev_position.velocity_to(last_position, *last_time - *prev_time)
        } else {
            // No idea, just return zeros.
            Vector::zero()
        }
    }

    // Some indication of the player's desire to move.
    pub fn ave_abs_velocity(&self) -> Vector {
        let (items, sum) = self
            .positions
            .iter()
            .zip(self.positions.iter().skip(1))
            .map(|((prev_position, prev_time), (position, time))| {
                prev_position
                    .velocity_to(position, *time - *prev_time)
                    .abs()
            })
            .fold((0, Vector::zero()), |acc, next| (acc.0 + 1, acc.1 + next));

        if items == 0 {
            Vector::zero()
        } else {
            sum / items as f32
        }
    }
}

pub struct ScoreHistory {
    inner: Vec<(u32, Instant)>,
}

impl ScoreHistory {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, score: u32, time: Instant) {
        self.inner.push((score, time));
    }

    pub fn last_score(&self) -> u32 {
        self.inner.last().unwrap().0
    }

    pub fn score_since(&self, since: Instant) -> u32 {
        let start_score = self
            .inner
            .iter()
            .rev()
            .find_map(|(score, time)| if *time <= since { Some(*score) } else { None })
            .unwrap_or(0u32);
        self.last_score() - start_score
    }

    pub fn project(&self, after: Duration) -> u32 {
        let past_duration = Duration::from_secs(10); // configurable
        let past_score = self.score_since(Instant::now() - past_duration);
        self.last_score()
            + (past_score as f32 * (after.as_millis() as f32 / past_duration.as_millis() as f32))
                as u32
    }
}

pub struct Bullet {
    pub position: Point,
    pub velocity: Vector,
    pub player_id: u32,
}

impl Bullet {
    pub fn new(state: &BulletState) -> Self {
        Bullet {
            position: Point::new(state.x, state.y),
            velocity: Vector::with_angle(state.angle) * BULLET_SPEED,
            player_id: state.player_id,
        }
    }
}