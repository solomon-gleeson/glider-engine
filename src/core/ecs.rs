use bevy::prelude::*;

pub struct EcsPlugin;

impl Plugin for EcsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<EngineState>();
        app.add_systems(
            Update,
            transition_to_running.run_if(in_state(EngineState::Loading)),
        );
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum EngineState {
    #[default]
    Loading,
    Running,
}

fn transition_to_running(mut next_state: ResMut<NextState<EngineState>>) {
    // Transition immediately to the Running state for now.
    next_state.set(EngineState::Running);
    info!("Engine transitioned to Running state");
}
