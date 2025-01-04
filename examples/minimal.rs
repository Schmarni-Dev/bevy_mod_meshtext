use bevy::prelude::*;
use bevy_mod_meshtext::{MeshText, MeshTextPlugin, VerticalLayout};
fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, MeshTextPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_origin)
        .run()
}
fn setup(mut cmds: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let text = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::GREEN,
        ..default()
    });
    cmds.spawn((
        MeshMaterial3d(text),
        MeshText {
            text: "Hello,\nWorld!".into(),
            height: 0.1,
            depth: 0.01,
        },
        VerticalLayout::Centered,
    ));
    cmds.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.5, 0.5, 2.0).looking_at(Vec3::ZERO, Dir3::Y),
    ));
}
fn draw_origin(query: Query<&GlobalTransform, With<MeshText>>, mut gizmos: Gizmos) {
    for t in &query {
        gizmos.axes(*t, 0.05);
    }
}
