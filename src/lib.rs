use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
};

use atomicow::CowArc;
use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use meshtext::{MeshGenerator, TextSection};

pub struct MeshTextPlugin;

impl Plugin for MeshTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_meshes);
    }
}
/// This Inserts a Mesh3d onto the Entity with the component
#[derive(Component, Clone, Debug, PartialEq, Deref, DerefMut)]
#[require(MeshTextFont, MeshTextHash)]
pub struct MeshText {
    #[deref]
    pub text: CowArc<'static, str>,
    /// the height in meters
    pub height: f32,
    /// the depth in meters
    pub depth: f32,
}
#[derive(Component, Clone, Debug, PartialEq, Default, Deref, DerefMut, Hash)]
pub struct MeshTextFont(pub Handle<Font>);
#[derive(Component, Debug, Default)]
struct MeshTextHash(u64);

impl Hash for MeshText {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
        state.write_u32(self.height.to_bits());
        state.write_u32(self.depth.to_bits());
    }
}

fn update_meshes(
    mut query: Query<(
        Entity,
        &MeshText,
        &MeshTextFont,
        &VerticalLayout,
        &mut MeshTextHash,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    fonts: Res<Assets<Font>>,
    mut cmds: Commands,
) {
    for (entity, text, font, vertical_layout, mut hash) in query.iter_mut() {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        font.hash(&mut hasher);
        let text_hash = hasher.finish();
        if hash.0 == text_hash {
            continue;
        }
        let Some(font_data) = fonts.get(&font.0) else {
            continue;
        };

        // annoying, i am not even keeping this
        let mut generator = MeshGenerator::new(font_data.data.deref().clone());
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let total_lines = text.lines().count();
        for (line_index, line) in text.lines().enumerate() {
            info!("line {line_index}");
            let text_mesh: meshtext::MeshText = generator
                .generate_section(line, text.depth == 0.0, None)
                .unwrap();

            let vertices = text_mesh.vertices;
            let y_offset = get_y_offset(vertical_layout, text.height, total_lines, line_index);
            positions.extend(vertices.chunks(3).map(|c| {
                [
                    c[0] * text.height,
                    (c[1] * text.height) + y_offset,
                    c[2] * text.depth,
                ]
            }));
        }
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        let uvs = vec![[0f32, 0f32]; positions.len()];
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.compute_flat_normals();
        let mesh = meshes.add(mesh);
        cmds.entity(entity).insert(Mesh3d(mesh));
        hash.0 = text_hash;
    }
}

/// Where should the origin be placed
#[derive(Component, Clone, Debug, PartialEq, Default, Hash, Eq)]
pub enum VerticalLayout {
    #[default]
    Centered,
    Top,
    Bottom,
}

fn get_y_offset(
    layout: &VerticalLayout,
    line_height: f32,
    total_lines: usize,
    current_line: usize,
) -> f32 {
    match layout {
        VerticalLayout::Centered => {
            let total = total_lines as f32 * line_height;

            let fraction = match total_lines {
                1 => 0.5,
                _ => (current_line) as f32 / total_lines as f32,
            };
            let out = total * -fraction;
            info!(total, fraction, out);
            out
        }
        VerticalLayout::Top => (-line_height) * (current_line + 1) as f32,
        VerticalLayout::Bottom => line_height * current_line as f32,
    }
}
