use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    prelude::*,
    reflect::TypePath,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{sampler, texture_2d},
            *,
        },
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
        RenderApp,
    },
};
use binding_types::{storage_buffer_read_only_sized, uniform_buffer_sized};
use std::{num::NonZero, process::exit};

pub mod prelude {
    pub use super::{GridBindlessMaterial, GridMapping, GridMaterialPlugin};
}

const SHADER_ASSET_PATH: &str = "shaders/texture_binding_array.wgsl";

/// Describe the mapping that is used to update the grid material
///
/// * When this component changes it will update the material that is used on the grid
#[derive(Component, Deref, DerefMut)]
pub struct GridMapping(pub Vec<u32>);

/// Describe the material that is used on the grid to show textures as tiles
#[derive(Asset, TypePath, Debug, Clone, Default)]
pub struct GridBindlessMaterial {
    /// The size of the final texture
    size: UVec2,
    /// The textures to use
    textures: Vec<Handle<Image>>,
    /// The mapping of the textures to use in each tile
    mapping: Vec<u32>,
}

/// System set for the grid material plugin
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GridMaterialSet;

/// Plugin to handle grid material
pub struct GridMaterialPlugin;

impl Plugin for GridMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<GridBindlessMaterial>::default(),
            GpuFeatureSupportChecker,
        ))
        .add_systems(Update, update_material.in_set(GridMaterialSet));
    }
}

fn update_material(
    q_material: Query<(&MeshMaterial3d<GridBindlessMaterial>, &GridMapping), Changed<GridMapping>>,
    mut materials: ResMut<Assets<GridBindlessMaterial>>,
) {
    for (bindless, GridMapping(mapping)) in q_material.iter() {
        if let Some(material) = materials.get_mut(bindless) {
            material.mapping = mapping.to_vec();
        }
    }
}

const MAX_TEXTURE_COUNT: usize = 128;

impl GridBindlessMaterial {
    pub fn new(size: UVec2, textures: Vec<Handle<Image>>) -> Self {
        Self {
            size,
            textures,
            mapping: vec![0; (size.x * size.y) as usize],
        }
    }
}

impl AsBindGroup for GridBindlessMaterial {
    type Data = ();

    type Param = (SRes<RenderAssets<GpuImage>>, SRes<FallbackImage>);

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        (image_assets, fallback_image): &mut SystemParamItem<'_, '_, Self::Param>,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let mut images = vec![];
        for handle in self.textures.iter().take(MAX_TEXTURE_COUNT) {
            match image_assets.get(handle) {
                Some(image) => images.push(image),
                None => return Err(AsBindGroupError::RetryNextUpdate),
            }
        }

        let fallback_image = &fallback_image.d2;

        let textures = vec![&fallback_image.texture_view; MAX_TEXTURE_COUNT];

        let mut textures: Vec<_> = textures.into_iter().map(|texture| &**texture).collect();

        for (id, image) in images.into_iter().enumerate() {
            textures[id] = &*image.texture_view;
        }

        let mapping = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("bindless_material_mapping"),
            contents: &self
                .mapping
                .iter()
                .flat_map(|kind| bytemuck::bytes_of(kind).to_vec())
                .collect::<Vec<u8>>(),
            usage: BufferUsages::STORAGE,
        });

        let size = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("bindless_material_size"),
            contents: &bytemuck::bytes_of(&self.size).to_vec(),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = render_device.create_bind_group(
            "bindless_material_bind_group",
            layout,
            &BindGroupEntries::sequential((
                &textures[..],
                &fallback_image.sampler,
                mapping.as_entire_binding(),
                size.as_entire_binding(),
            )),
        );

        Ok(PreparedBindGroup {
            bindings: vec![],
            bind_group,
            data: (),
        })
    }

    fn unprepared_bind_group(
        &self,
        _layout: &BindGroupLayout,
        _render_device: &RenderDevice,
        _param: &mut SystemParamItem<'_, '_, Self::Param>,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        // we implement as_bind_group directly because
        panic!("bindless texture arrays can't be owned")
        // or rather, they can be owned, but then you can't make a `&'a [&'a TextureView]` from a vec of them in get_binding().
    }

    fn bind_group_layout_entries(_: &RenderDevice) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        BindGroupLayoutEntries::with_indices(
            // The layout entries will only be visible in the fragment stage
            ShaderStages::FRAGMENT,
            (
                // Screen texture
                //
                // @group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
                (
                    0,
                    texture_2d(TextureSampleType::Float { filterable: true })
                        .count(NonZero::<u32>::new(MAX_TEXTURE_COUNT as u32).unwrap()),
                ),
                // @group(2) @binding(1) var nearest_sampler: sampler;
                (1, sampler(SamplerBindingType::Filtering)),
                // @group(2) @binding(2) var<storage, read> mapping: array<u32>;
                (2, storage_buffer_read_only_sized(false, None)),
                // @group(2) @binding(3) var<uniform> size: vec2<u32>;
                (3, uniform_buffer_sized(false, None)),
            ),
        )
        .to_vec()
    }
}

impl Material for GridBindlessMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

impl IntoIterator for GridBindlessMaterial {
    type Item = u32;
    type IntoIter = std::vec::IntoIter<u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.mapping.into_iter()
    }
}

impl<'a> IntoIterator for &'a GridBindlessMaterial {
    type Item = &'a u32;
    type IntoIter = std::slice::Iter<'a, u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.mapping.iter()
    }
}

impl<'a> IntoIterator for &'a mut GridBindlessMaterial {
    type Item = &'a mut u32;
    type IntoIter = std::slice::IterMut<'a, u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.mapping.iter_mut()
    }
}

struct GpuFeatureSupportChecker;

impl Plugin for GpuFeatureSupportChecker {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let render_device = render_app.world().resource::<RenderDevice>();

        // Check if the device support the required feature. If not, exit the example.
        // In a real application, you should setup a fallback for the missing feature
        if !render_device
            .features()
            .contains(WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
        {
            error!(
                "Render device doesn't support feature \
SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, \
which is required for texture binding arrays"
            );
            exit(1);
        }
    }
}
