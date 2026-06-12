use bevy::asset::RenderAssetUsages;
use bevy::camera::{ClearColorConfig, Projection, RenderTarget};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use super::editor_state::EditorState;
use super::theme::EditorTheme;

pub const GAME_VIEW_SIZE: UVec2 = UVec2::new(1280, 720);

#[derive(Resource)]
#[allow(dead_code)]
pub struct GameView {
    pub image: Handle<Image>,
}

#[derive(Component)]
pub struct GameViewCamera;

pub fn setup_game_view(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    already_setup: Option<Res<GameView>>,
    theme: Res<EditorTheme>,
) {
    if already_setup.is_some() {
        return;
    }

    let size = Extent3d {
        width: GAME_VIEW_SIZE.x,
        height: GAME_VIEW_SIZE.y,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let handle = images.add(image);

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,

            clear_color: ClearColorConfig::Custom(theme.colors.viewport_bg),
            ..default()
        },
        RenderTarget::from(handle.clone()),
        GameViewCamera,
    ));

    commands.insert_resource(GameView { image: handle });
}

pub fn apply_viewport_zoom(
    state: Res<EditorState>,
    mut cameras: Query<&mut Projection, With<GameViewCamera>>,
) {
    for mut projection in &mut cameras {
        if let Projection::Orthographic(ortho) = &mut *projection
            && (ortho.scale - state.viewport_zoom).abs() > f32::EPSILON
        {
            ortho.scale = state.viewport_zoom;
        }
    }
}
