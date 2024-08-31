use core_graphics::{base::CGFloat, geometry::CGRect};
use egui::Context;
use glam::{uvec2, UVec2};
use objc::{runtime::Object, *};
use std::marker::Sync;
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration};

use crate::{CaMetalLayer, UiViewObject};

pub struct Graphics {
    pub view: UiViewObject,
    pub scale_factor: f32,
    pub gpu: Gpu,
}
unsafe impl Sync for Graphics {}

impl Graphics {
    pub fn new(view: UiViewObject, metal_layer: CaMetalLayer) -> Self {
        let scale_factor = get_scale_factor(view.0);
        let s: CGRect = unsafe { msg_send![view.0, frame] };
        let mut physical = (
            (s.size.width as f32 * scale_factor) as u32,
            (s.size.height as f32 * scale_factor) as u32,
        );
        if physical.0 == 0 {
            physical.0 = 1;
        }
        if physical.1 == 0 {
            physical.1 = 1;
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..Default::default()
        });
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(metal_layer.0))
                .expect("Surface creation failed")
        };
        let (adapter, device, queue) = pollster::block_on(request_device(&instance, &surface));
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // CAMatalLayer's pixel format default value is MTLPixelFormatBGRA8Unorm.
            // https://developer.apple.com/documentation/quartzcore/cametallayer/1478155-pixelformat?language=objc
            // format: wgpu::TextureFormat::Bgra8Unorm,
            // format: surface.get_supported_formats(&adapter)[0],
            format: surface.get_capabilities(&adapter).formats[0],
            width: physical.0,
            height: physical.1,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::PostMultiplied,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);
        let gpu = Gpu {
            surface,
            surface_config,
            device: Arc::new(device),
            queue: Arc::new(queue),
        };
        Self {
            view,
            scale_factor,
            gpu,
        }
    }

    pub fn get_view_size(&self) -> (u32, u32) {
        let s: CGRect = unsafe { msg_send![self.view.0, frame] };
        (
            (s.size.width as f32 * self.scale_factor) as u32,
            (s.size.height as f32 * self.scale_factor) as u32,
        )
    }
}

fn get_scale_factor(obj: *mut Object) -> f32 {
    let s: CGFloat = unsafe { msg_send![obj, contentScaleFactor] };
    s as f32
}

pub struct Gpu {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
}
impl Gpu {
    pub fn update_config_format(&mut self, format: wgpu::TextureFormat) {
        self.surface_config.format = format;

        self.surface_config.view_formats =
            vec![format.add_srgb_suffix(), format.remove_srgb_suffix()];
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn resize(&mut self, new_size: UVec2) {
        self.surface_config.width = new_size[0];
        self.surface_config.height = new_size[1];
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&Default::default())
    }

    pub fn surface_size(&self) -> UVec2 {
        uvec2(self.surface_config.width, self.surface_config.height)
    }

    pub fn get_output(
        &self,
    ) -> Result<(wgpu::SurfaceTexture, wgpu::TextureView), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        Ok((output, view))
    }
}

async fn request_device(
    instance: &Instance,
    surface: &Surface<'static>,
) -> (Adapter, Device, Queue) {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::util::power_preference_from_env()
                .unwrap_or(wgpu::PowerPreference::HighPerformance),
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        })
        .await
        .expect("No suitable GPU adapters found on the system!");
    let adapter_info = adapter.get_info();
    let res = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: adapter.features(),
                required_limits: adapter.limits(),
            },
            None,
        )
        .await;
    match res {
        Err(err) => {
            panic!("request_device failed: {err:?}");
        }
        Ok((device, queue)) => (adapter, device, queue),
    }
}

pub struct Egui {
    pub graphics: Graphics,
    pub renderer: egui_wgpu::Renderer,
    pub ctx: egui::Context,
}
impl Egui {
    pub fn new(graphics: Graphics) -> Self {
        let gpu = &graphics.gpu;
        Self {
            renderer: egui_wgpu::Renderer::new(&gpu.device, gpu.surface_config.format, None, 1),
            ctx: egui::Context::default(),
            graphics,
        }
    }

    pub fn draw_frame(&mut self, mut input: egui::RawInput, f: impl FnOnce(&Context)) {
        let Graphics {
            gpu, scale_factor, ..
        } = &self.graphics;
        let (output, view) = match gpu.get_output() {
            Ok(v) => v,
            Err(err) => {
                log::error!("GPU surface error: {err:?}");
                return;
            }
        };
        let mut encoder = gpu.create_command_encoder();
        {
            // --- create scene ---
            let scale = crate::input::SCALE_FACTOR * scale_factor;
            let (w, h) = (gpu.surface_config.width, gpu.surface_config.height);

            let content_rect = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(w as f32 / scale, h as f32 / scale),
            );

            let viewport = input
                .viewports
                .get_mut(&egui::viewport::ViewportId::ROOT)
                .unwrap();
            viewport.native_pixels_per_point = Some(scale);
            viewport.inner_rect = Some(content_rect);
            input.screen_rect = Some(content_rect);
            let ctx = self.ctx.clone();
            let egui_output = ctx.run(input, |ctx| {
                f(ctx);
            });
            let egui_prims = self
                .ctx
                .tessellate(egui_output.shapes, egui_output.pixels_per_point);
            let screen_desc = egui_wgpu::ScreenDescriptor {
                size_in_pixels: gpu.surface_size().into(),
                pixels_per_point: egui_output.pixels_per_point,
            };

            // --- update buffers ---
            for (id, image) in egui_output.textures_delta.set {
                log::info!("Updating egui_renderer texture {id:?}");
                self.renderer
                    .update_texture(&gpu.device, &gpu.queue, id, &image);
            }
            self.renderer.update_buffers(
                &gpu.device,
                &gpu.queue,
                &mut encoder,
                &egui_prims,
                &screen_desc,
            );

            // --- render pass ---
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                occlusion_query_set: None,
                timestamp_writes: None,
                label: Some("#egui_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
            });
            self.renderer.render(&mut pass, &egui_prims, &screen_desc);
            std::mem::drop(pass);

            for id in egui_output.textures_delta.free {
                self.renderer.free_texture(&id);
            }

            gpu.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
    }
}
