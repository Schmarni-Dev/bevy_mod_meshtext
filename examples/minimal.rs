use bevy::prelude::*;
use bevy_mod_meshtext::{DepthLayout, HorizontalLayout, MeshText, MeshTextPlugin, VerticalLayout};
fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshTextPlugin,
            bevy_flycam::NoCameraPlayerPlugin,
        ))
        .insert_resource(bevy_flycam::MovementSettings {
            sensitivity: 0.00012, // default: 0.00012
            speed: 2.0,           // default: 12.0
        })
        .add_systems(Startup, setup)
        .add_systems(Update, draw_origin)
        .run()
}
fn setup(mut cmds: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let text = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    cmds.spawn((
        MeshMaterial3d(text.clone()),
        MeshText {
            text: "Hello, World!".into(),
            height: 0.1,
            depth: 0.01,
        },
        VerticalLayout::Centered,
        DepthLayout::Back,
        HorizontalLayout::Right,
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));
    cmds.spawn((
        MeshMaterial3d(text.clone()),
        MeshText {
            text: "Hello,\nWorld!".into(),
            height: 0.1,
            depth: 0.01,
        },
        VerticalLayout::Centered,
        DepthLayout::Front,
        HorizontalLayout::Centered
    ));
    cmds.spawn((
        MeshMaterial3d(text),
        MeshText {
            text: include_str!("lorem_ipsum.txt").into(),
            height: 0.1,
            depth: 0.01,
        },
        VerticalLayout::Centered,
        Transform::from_xyz(0.0, 0.0, -2.0),
    ));
    cmds.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.5, 0.5, -2.0).looking_at(Vec3::ZERO, Dir3::Y),
        bevy_flycam::FlyCam,
    ));
}
fn draw_origin(query: Query<&GlobalTransform, With<MeshText>>, mut gizmos: Gizmos) {
    for t in &query {
        gizmos.axes(*t, 0.1);
    }
}
