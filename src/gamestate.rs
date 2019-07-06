use std::ops::BitOr;

#[derive(Eq, Debug, PartialEq, Copy, Clone)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
    Countdown,
}

impl GameState {
    pub fn is_playing(self) -> bool {
        match self {
            GameState::Playing => true,
            _ => false,
        }
    }
    pub fn is_paused(self) -> bool {
        match self {
            GameState::Paused => true,
            _ => false,
        }
    }
    pub fn is_gameover(self) -> bool {
        match self {
            GameState::GameOver => true,
            _ => false,
        }
    }
    pub fn is_countdown(self) -> bool {
        match self {
            GameState::Countdown => true,
            _ => false,
        }
    }
    pub fn toggle_pause(&mut self) {
        if self.is_paused() {
            *self = GameState::Playing;
        } else {
            *self = GameState::Paused;
        }
    }
}

impl BitOr<Self> for GameState {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (GameState::GameOver, _) => GameState::GameOver,
            (_, GameState::GameOver) => GameState::GameOver,
            _ => rhs,
        }
    }
}
