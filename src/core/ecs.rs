use bevy::prelude::*;

pub struct EcsPlugin;

impl Plugin for EcsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EngineState>();
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum EngineState {
    #[default]
    Loading,
    Running,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_state_default() {
        assert_eq!(EngineState::default(), EngineState::Loading);
    }
}
