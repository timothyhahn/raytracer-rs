//! GPU renderer implementation

use super::buffers::*;
use super::pipeline::ComputePipeline;
use crate::core::color::Color;
use crate::rendering::camera::Camera;
use crate::rendering::canvas::Canvas;
use crate::rendering::world::World;
use bytemuck::{cast_slice, Pod, Zeroable};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;

#[cfg(test)]
use once_cell::sync::OnceCell;

#[cfg(test)]
static GPU_RENDERER_INSTANCE: OnceCell<Option<Arc<GpuRenderer>>> = OnceCell::new();

pub struct GpuRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl Clone for GpuRenderer {
    fn clone(&self) -> Self {
        Self {
            device: Arc::clone(&self.device),
            queue: Arc::clone(&self.queue),
        }
    }
}

impl GpuRenderer {
    pub fn try_new() -> Option<Self> {
        #[cfg(test)]
        {
            GPU_RENDERER_INSTANCE
                .get_or_init(|| match Self::new_sync() {
                    Ok(renderer) => Some(Arc::new(renderer)),
                    Err(err) => {
                        eprintln!("Failed to create GPU renderer: {}", err);
                        None
                    }
                })
                .as_ref()
                .map(|arc| (**arc).clone())
        }

        #[cfg(not(test))]
        {
            Self::new_sync().ok()
        }
    }

    pub fn render(&self, camera: &Camera, world: &World) -> Canvas {
        self.render_progressive(camera, world, 50, None)
    }

    pub fn render_with_progress(
        &self,
        camera: &Camera,
        world: &World,
        max_depth: u32,
        multi: Option<&MultiProgress>,
    ) -> Canvas {
        self.render_progressive(camera, world, max_depth, multi)
    }

    fn new_sync() -> Result<Self, Box<dyn std::error::Error>> {
        pollster::block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
                .ok_or("Failed to find suitable GPU adapter")?;

            let limits = wgpu::Limits {
                max_storage_buffers_per_shader_stage: 12,
                ..Default::default()
            };

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("Raytracer GPU Device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: limits,
                    },
                    None,
                )
                .await?;

            Ok(Self {
                device: Arc::new(device),
                queue: Arc::new(queue),
            })
        })
    }

    fn render_progressive(
        &self,
        camera: &Camera,
        world: &World,
        max_depth: u32,
        multi: Option<&MultiProgress>,
    ) -> Canvas {
        let width = camera.hsize;
        let height = camera.vsize;

        let total_samples = camera.aa_samples.max(1);
        let samples_per_axis = ((total_samples as f64).sqrt().ceil() as u32).max(1);
        let total_samples = samples_per_axis * samples_per_axis;

        const MAX_CHUNK_SIZE: u32 = 25;
        let chunks = calculate_chunks(total_samples, MAX_CHUNK_SIZE);

        let progress = multi.map(|m| {
            let pb = m.add(ProgressBar::new(chunks.len() as u64));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} chunks")
                    .unwrap()
                    .progress_chars("##-"),
            );
            pb
        });

        let mut accumulated_canvas = Canvas::new(width, height);

        for chunk in chunks {
            let mut scene_buffers =
                GpuSceneBuffers::from_scene_with_depth(camera, world, max_depth, chunk.random_seed);

            scene_buffers.camera.chunk_samples = chunk.chunk_samples;
            scene_buffers.camera.sample_offset = chunk.sample_offset;
            scene_buffers.camera.samples_per_axis = samples_per_axis;
            scene_buffers.camera.aa_samples = total_samples;

            let partial_canvas = self.render_chunk(&scene_buffers, width, height);

            accumulate_canvas(&mut accumulated_canvas, &partial_canvas, chunk.weight);

            if let Some(ref pb) = progress {
                pb.inc(1);
            }
        }

        if let Some(pb) = progress {
            pb.finish_with_message("Rendering complete");
        }

        accumulated_canvas
    }

    fn render_chunk(&self, scene_buffers: &GpuSceneBuffers, width: u32, height: u32) -> Canvas {
        let output_texture = self.create_output_texture(width, height);
        let pipeline = ComputePipeline::new(&self.device);

        let camera_buffer = self.create_uniform_buffer(&scene_buffers.camera);
        let spheres_buffer = self.create_storage_buffer(&scene_buffers.spheres);
        let materials_buffer = self.create_storage_buffer(&scene_buffers.materials);
        let planes_buffer = self.create_storage_buffer(&scene_buffers.planes);
        let cubes_buffer = self.create_storage_buffer(&scene_buffers.cubes);
        let cylinders_buffer = self.create_storage_buffer(&scene_buffers.cylinders);
        let cones_buffer = self.create_storage_buffer(&scene_buffers.cones);
        let triangles_buffer = self.create_storage_buffer(&scene_buffers.triangles);
        let transforms_buffer = self.create_storage_buffer(&scene_buffers.transforms);
        let bvh_nodes_buffer = self.create_storage_buffer(&scene_buffers.bvh_nodes);
        let bvh_indices_buffer = self.create_storage_buffer(&scene_buffers.bvh_triangle_indices);

        let light = scene_buffers.light.unwrap_or_default();
        let light_buffer = self.create_uniform_buffer(&light);

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Raytracer Bind Group"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &output_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: spheres_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: materials_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: planes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: cylinders_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: cones_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: transforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: triangles_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: bvh_nodes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: bvh_indices_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Raytracer Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&pipeline.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let workgroup_x = width.div_ceil(8);
            let workgroup_y = height.div_ceil(8);
            compute_pass.dispatch_workgroups(workgroup_x, workgroup_y, 1);
        }
        self.queue.submit(Some(encoder.finish()));

        self.download_texture(&output_texture, width, height)
    }

    fn create_uniform_buffer<T: Pod>(&self, data: &T) -> wgpu::Buffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<T>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buffer, 0, cast_slice(&[*data]));
        buffer
    }

    fn create_storage_buffer<T: Pod + Zeroable>(&self, data: &[T]) -> wgpu::Buffer {
        let element_size = std::mem::size_of::<T>();
        let size = if data.is_empty() {
            element_size.max(16) as u64
        } else {
            std::mem::size_of_val(data) as u64
        };

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Storage Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        if !data.is_empty() {
            self.queue.write_buffer(&buffer, 0, cast_slice(data));
        } else {
            let zero = T::zeroed();
            self.queue.write_buffer(&buffer, 0, cast_slice(&[zero]));
        }

        buffer
    }

    fn create_output_texture(&self, width: u32, height: u32) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn download_texture(&self, texture: &wgpu::Texture, width: u32, height: u32) -> Canvas {
        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

        let buffer_size = (padded_bytes_per_row * height) as u64;
        let download_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Download Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Download Encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &download_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = download_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);

        let data = buffer_slice.get_mapped_range();
        let mut canvas = Canvas::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let padded_offset = (y * padded_bytes_per_row + x * bytes_per_pixel) as usize;
                let r = data[padded_offset] as f64 / 255.0;
                let g = data[padded_offset + 1] as f64 / 255.0;
                let b = data[padded_offset + 2] as f64 / 255.0;
                canvas.write_pixel(x, y, &Color::new(r, g, b));
            }
        }

        drop(data);
        download_buffer.unmap();

        canvas
    }
}

#[derive(Debug, PartialEq)]
struct ChunkInfo {
    sample_offset: u32,
    chunk_samples: u32,
    random_seed: u32,
    weight: f64,
}

fn calculate_chunks(total_samples: u32, max_chunk_size: u32) -> Vec<ChunkInfo> {
    let mut chunks = Vec::new();
    let mut offset = 0;
    let mut chunk_index = 0u32;

    while offset < total_samples {
        let remaining = total_samples - offset;
        let chunk_samples = remaining.min(max_chunk_size);
        let weight = chunk_samples as f64 / total_samples as f64;

        chunks.push(ChunkInfo {
            sample_offset: offset,
            chunk_samples,
            random_seed: chunk_index.wrapping_mul(1103515245).wrapping_add(12345),
            weight,
        });

        offset += chunk_samples;
        chunk_index += 1;
    }

    chunks
}

fn accumulate_canvas(base: &mut Canvas, partial: &Canvas, weight: f64) {
    for y in 0..base.height {
        for x in 0..base.width {
            let base_color = base.pixel_at(x, y);
            let partial_color = partial.pixel_at(x, y);

            let accumulated = Color::new(
                base_color.red + partial_color.red * weight,
                base_color.green + partial_color.green * weight,
                base_color.blue + partial_color.blue * weight,
            );

            base.write_pixel(x, y, &accumulated);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunks_evenly_divisible() {
        let chunks = calculate_chunks(100, 25);

        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[0].sample_offset, 0);
        assert_eq!(chunks[0].chunk_samples, 25);
        assert_eq!(chunks[0].weight, 0.25);

        assert_eq!(chunks[1].sample_offset, 25);
        assert_eq!(chunks[1].chunk_samples, 25);
        assert_eq!(chunks[1].weight, 0.25);

        assert_eq!(chunks[3].sample_offset, 75);
        assert_eq!(chunks[3].chunk_samples, 25);
        assert_eq!(chunks[3].weight, 0.25);
    }

    #[test]
    fn test_chunks_with_remainder() {
        let chunks = calculate_chunks(100, 30);

        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[0].chunk_samples, 30);
        assert_eq!(chunks[0].weight, 0.30);

        assert_eq!(chunks[1].chunk_samples, 30);
        assert_eq!(chunks[1].weight, 0.30);

        assert_eq!(chunks[2].chunk_samples, 30);
        assert_eq!(chunks[2].weight, 0.30);

        assert_eq!(chunks[3].sample_offset, 90);
        assert_eq!(chunks[3].chunk_samples, 10);
        assert_eq!(chunks[3].weight, 0.10);
    }

    #[test]
    fn test_chunks_single_chunk_when_small() {
        let chunks = calculate_chunks(50, 100);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].sample_offset, 0);
        assert_eq!(chunks[0].chunk_samples, 50);
        assert_eq!(chunks[0].weight, 1.0);
    }

    #[test]
    fn test_chunks_have_different_seeds() {
        let chunks = calculate_chunks(100, 25);

        let seed0 = chunks[0].random_seed;
        let seed1 = chunks[1].random_seed;
        let seed2 = chunks[2].random_seed;

        assert_ne!(seed0, seed1);
        assert_ne!(seed1, seed2);
        assert_ne!(seed0, seed2);
    }

    #[test]
    fn test_canvas_accumulation() {
        let mut base = Canvas::new(2, 2);
        let mut partial = Canvas::new(2, 2);

        partial.write_pixel(0, 0, &Color::new(1.0, 0.0, 0.0));
        partial.write_pixel(1, 1, &Color::new(0.0, 1.0, 0.0));

        accumulate_canvas(&mut base, &partial, 0.5);

        let pixel_00 = base.pixel_at(0, 0);
        assert!((pixel_00.red - 0.5).abs() < 1e-5);
        assert!((pixel_00.green - 0.0).abs() < 1e-5);
        assert!((pixel_00.blue - 0.0).abs() < 1e-5);

        let pixel_11 = base.pixel_at(1, 1);
        assert!((pixel_11.red - 0.0).abs() < 1e-5);
        assert!((pixel_11.green - 0.5).abs() < 1e-5);
        assert!((pixel_11.blue - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_canvas_accumulation_multiple_passes() {
        let mut base = Canvas::new(1, 1);
        let mut pass1 = Canvas::new(1, 1);
        let mut pass2 = Canvas::new(1, 1);

        pass1.write_pixel(0, 0, &Color::new(0.8, 0.4, 0.2));
        pass2.write_pixel(0, 0, &Color::new(0.2, 0.6, 0.8));

        accumulate_canvas(&mut base, &pass1, 0.5);
        accumulate_canvas(&mut base, &pass2, 0.5);

        let pixel = base.pixel_at(0, 0);
        assert!((pixel.red - 0.5).abs() < 1e-5);
        assert!((pixel.green - 0.5).abs() < 1e-5);
        assert!((pixel.blue - 0.5).abs() < 1e-5);
    }
}
