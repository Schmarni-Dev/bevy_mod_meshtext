use core::f32;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
};

use atomicow::CowArc;
use bevy::{
    math::Vec3A,
    prelude::*,
    render::{
        mesh::MeshAabb, primitives::Aabb, render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use meshtext::{MeshGenerator, TextSection};

pub struct MeshTextPlugin;

impl Plugin for MeshTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (update_meshes, attach_meshes).chain());
    }
}
/// This Inserts a Mesh3d onto the Entity with the component
#[derive(Component, Clone, Debug, PartialEq, Deref, DerefMut)]
#[require(
    MeshTextFont,
    MeshTextHash,
    VerticalLayout,
    DepthLayout,
    HorizontalLayout
)]
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
#[derive(Component)]
struct ComputeMeshText(Task<(Mesh, Option<Aabb>)>);

fn update_meshes(
    mut query: Query<(
        Entity,
        &MeshText,
        &MeshTextFont,
        &VerticalLayout,
        &DepthLayout,
        &HorizontalLayout,
        &mut MeshTextHash,
    )>,
    fonts: Res<Assets<Font>>,
    mut cmds: Commands,
) {
    let pool = AsyncComputeTaskPool::get();
    for (entity, text, font, vertical_layout, depth_layout, horizontal_layout, mut hash) in
        query.iter_mut()
    {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        font.hash(&mut hasher);
        let text_hash = hasher.finish();
        if hash.0 == text_hash {
            continue;
        }
        let Some(font_data) = fonts.get(&font.0).map(|data| data.data.deref().clone()) else {
            continue;
        };
        let text = text.clone();
        let vertical_layout = *vertical_layout;
        let depth_layout = *depth_layout;
        let horizontal_layout = *horizontal_layout;
        let task = pool.spawn(async move {
            // annoying, i am not even keeping this
            let mut generator = MeshGenerator::new(font_data);
            let mut positions: Vec<[f32; 3]> = Vec::new();
            let total_lines = text.lines().count();
            for (line_index, line) in text.lines().enumerate() {
                let text_mesh: meshtext::MeshText = generator
                    .generate_section(line, text.depth == 0.0, None)
                    .unwrap();

                let vertices = text_mesh.vertices;
                let width = vertices
                    .chunks(3)
                    .fold(f32::NEG_INFINITY, |v, p| v.max(p[0]))
                    * text.height;
                let y_offset = get_y_offset(&vertical_layout, text.height, total_lines, line_index);
                let z_offset = get_z_offset(&depth_layout, text.depth);
                let x_offset = get_x_offset(&horizontal_layout, width);
                positions.extend(vertices.chunks(3).map(|c| {
                    let vec = Vec3A::from_array([
                        (c[0] * text.height) + x_offset,
                        (c[1] * text.height) + y_offset,
                        (c[2] * text.depth) + z_offset,
                    ]);
                    let quat = Quat::from_rotation_y(f32::consts::PI);
                    let vec = quat * vec;
                    vec.to_array()
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
            let aabb = mesh.compute_aabb();
            (mesh, aabb)
        });
        cmds.entity(entity).insert(ComputeMeshText(task));
        hash.0 = text_hash;
    }
}
fn attach_meshes(
    mut cmds: Commands,
    mut mesh_tasks: Query<(Entity, &mut ComputeMeshText)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut task) in &mut mesh_tasks {
        if let Some((mesh, aabb)) = block_on(future::poll_once(&mut task.0)) {
            let mesh = meshes.add(mesh);
            cmds.entity(entity)
                .insert(Mesh3d(mesh))
                .remove::<ComputeMeshText>();
            if let Some(aabb) = aabb {
                cmds.entity(entity).insert(aabb);
            }
        }
    }
}
/// Where should the origin be placed
#[derive(Component, Clone, Debug, PartialEq, Default, Hash, Eq, Copy)]
pub enum DepthLayout {
    #[default]
    Centered,
    Front,
    Back,
}

const fn get_z_offset(layout: &DepthLayout, depth: f32) -> f32 {
    match layout {
        DepthLayout::Centered => 0.0,
        DepthLayout::Front => -depth * 0.5,
        DepthLayout::Back => depth * 0.5,
    }
}

/// Where should the origin be placed
#[derive(Component, Clone, Debug, PartialEq, Default, Hash, Eq, Copy)]
pub enum HorizontalLayout {
    Centered,
    #[default]
    Left,
    Right,
}

const fn get_x_offset(layout: &HorizontalLayout, line_width: f32) -> f32 {
    match layout {
        HorizontalLayout::Centered => -line_width * 0.5,
        HorizontalLayout::Left => 0.0,
        HorizontalLayout::Right => -line_width,
    }
}

/// Where should the origin be placed
#[derive(Component, Clone, Debug, PartialEq, Default, Hash, Eq, Copy)]
pub enum VerticalLayout {
    Centered,
    Top,
    #[default]
    Bottom,
}

const fn get_y_offset(
    layout: &VerticalLayout,
    line_height: f32,
    total_lines: usize,
    current_line: usize,
) -> f32 {
    match layout {
        VerticalLayout::Centered => {
            let total = total_lines as f32 * line_height;
            let top = (-line_height) * (current_line + 1) as f32;
            top + (total * 0.5)
        }
        VerticalLayout::Top => (-line_height) * (current_line + 1) as f32,
        VerticalLayout::Bottom => line_height * current_line as f32,
    }
}
