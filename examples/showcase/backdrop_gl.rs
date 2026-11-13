use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use eframe::{
    egui_glow,
    glow::{self, HasContext},
};
use egui::{PaintCallback, PaintCallbackInfo};
use temivu::{GlassMode, GlassProfile, GlassRegion, GlassRenderer};

#[derive(Clone, Default)]
pub(super) struct BackdropGl {
    state: Arc<Mutex<Backend>>,
    failed: Arc<AtomicBool>,
}

#[derive(Default)]
struct Backend {
    context: usize,
    gpu: Option<Result<Gpu, String>>,
}

impl GlassRenderer for BackdropGl {
    fn available(&self) -> bool {
        !self.failed.load(Ordering::Relaxed)
    }

    fn callback(&self, region: GlassRegion) -> PaintCallback {
        let backend = self.clone();
        PaintCallback {
            rect: region.rect,
            callback: Arc::new(egui_glow::CallbackFn::new(move |info, painter| {
                backend.paint(info, painter, region);
            })),
        }
    }
}

impl BackdropGl {
    fn paint(&self, info: PaintCallbackInfo, painter: &egui_glow::Painter, region: GlassRegion) {
        let Some(capture) = Capture::new(&info, region) else {
            return;
        };
        let Ok(mut backend) = self.state.lock() else {
            return;
        };
        let gl = painter.gl().as_ref();
        let identity = gl as *const glow::Context as usize;
        if backend.context != identity {
            *backend = Backend {
                context: identity,
                gpu: None,
            };
        }
        // SAFETY: this callback is executed by the painter with its context current.
        // Every GL object below is owned by this adapter in that same context.
        unsafe {
            let gpu = backend.gpu.get_or_insert_with(|| {
                let result = Gpu::new(gl);
                if let Err(error) = &result {
                    self.failed.store(true, Ordering::Relaxed);
                    eprintln!("Backdrop renderer unavailable: {error}");
                }
                result
            });
            let Ok(gpu) = gpu else { return };
            if !gpu.paint(gl, painter.intermediate_fbo(), &info, capture, region) {
                gpu.destroy(gl);
                backend.gpu = Some(Err("framebuffer capture failed".to_owned()));
                self.failed.store(true, Ordering::Relaxed);
                eprintln!("Backdrop renderer disabled: framebuffer capture failed");
            }
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        if let Ok(mut backend) = self.state.lock()
            && backend.context == gl as *const glow::Context as usize
            && let Some(Ok(gpu)) = backend.gpu.take()
        {
            // SAFETY: eframe supplies the current rendering context during on_exit.
            unsafe { gpu.destroy(gl) };
        }
    }
}

#[derive(Clone, Copy)]
struct Capture {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Capture {
    fn new(info: &PaintCallbackInfo, region: GlassRegion) -> Option<Self> {
        let visible = info.viewport.intersect(info.clip_rect);
        if !visible.is_positive() {
            return None;
        }
        let optics = Optics::new(region);
        let padding = optics.sigma.into_iter().fold(0.0_f32, f32::max) * 2.8 + optics.lens + 4.0;
        let capture = egui::epaint::ViewportInPixels::from_points(
            &visible.expand(padding),
            info.pixels_per_point,
            info.screen_size_px,
        );
        let width = capture.width_px;
        let height = capture.height_px;
        if width < 2 || height < 2 {
            return None;
        }
        Some(Self {
            x: capture.left_px,
            y: capture.from_bottom_px,
            width,
            height,
        })
    }
}

struct Optics {
    sigma: [f32; 2],
    lens: f32,
    bevel: f32,
    polish: f32,
}

impl Optics {
    fn new(region: GlassRegion) -> Self {
        let (mut sigma, lens, bevel, polish) = match region.mode {
            GlassMode::Frost => ([15.5, 15.5], 0.0, 0.0, 0.0),
            GlassMode::SoftPrism => ([6.5, 6.5], 16.0, 20.0, 0.58),
            GlassMode::Prism => ([3.0, 3.0], 23.0, 23.0, 0.95),
        };
        if region.profile == GlassProfile::Dock {
            sigma[1] *= 1.25;
        }
        let tint = region.tint.clamp(0.0, 1.0);
        Self {
            sigma: sigma.map(|value| value * region.tuning.blur),
            lens: lens * (1.0 - 0.35 * tint) * region.tuning.refraction,
            bevel,
            polish: polish * region.tuning.rim,
        }
    }
}

#[derive(Clone, Copy)]
struct Surface {
    texture: glow::Texture,
    size: [i32; 2],
}

impl Surface {
    fn new(texture: glow::Texture) -> Self {
        Self {
            texture,
            size: [0; 2],
        }
    }

    unsafe fn reserve(&mut self, gl: &glow::Context, width: i32, height: i32) {
        if width <= self.size[0] && height <= self.size[1] {
            return;
        }
        self.size = [width.max(self.size[0]), height.max(self.size[1])];
        // SAFETY: the callback owns the current context and texture.
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                self.size[0],
                self.size[1],
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            for parameter in [glow::TEXTURE_MIN_FILTER, glow::TEXTURE_MAG_FILTER] {
                gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, glow::LINEAR as i32);
            }
        }
    }
}

struct Gpu {
    program: glow::Program,
    vao: glow::VertexArray,
    target: glow::Framebuffer,
    read: glow::Framebuffer,
    full: Surface,
    reduced: [Surface; 2],
    filter: Filter,
    uniforms: [Option<glow::UniformLocation>; 7],
}

impl Gpu {
    unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            // Scaling a multisampled framebuffer is unsupported. The normal app
            // framebuffer is single-sampled, so an unusual host falls back cleanly.
            if gl.get_parameter_i32(glow::SAMPLES) > 0 {
                return Err("a single-sampled target is required".into());
            }
            let program = program(gl, FRAGMENT)?;
            let vao = gl
                .create_vertex_array()
                .inspect_err(|_| gl.delete_program(program))?;
            let target = gl.create_framebuffer().inspect_err(|_| {
                gl.delete_vertex_array(vao);
                gl.delete_program(program);
            })?;
            let read = gl.create_framebuffer().inspect_err(|_| {
                gl.delete_framebuffer(target);
                gl.delete_vertex_array(vao);
                gl.delete_program(program);
            })?;
            let mut textures = Vec::with_capacity(3);
            let setup = (|| {
                for _ in 0..3 {
                    textures.push(gl.create_texture()?);
                }
                Filter::new(gl)
            })();
            let filter = setup.inspect_err(|_| {
                for &texture in &textures {
                    gl.delete_texture(texture);
                }
                gl.delete_framebuffer(read);
                gl.delete_framebuffer(target);
                gl.delete_vertex_array(vao);
                gl.delete_program(program);
            })?;
            let uniforms = [
                "u_texture",
                "u_region",
                "u_corners",
                "u_capture",
                "u_texels",
                "u_effect",
                "u_strength",
            ]
            .map(|name| gl.get_uniform_location(program, name));
            Ok(Self {
                program,
                vao,
                target,
                read,
                full: Surface::new(textures[0]),
                reduced: [Surface::new(textures[1]), Surface::new(textures[2])],
                filter,
                uniforms,
            })
        }
    }

    unsafe fn paint(
        &mut self,
        gl: &glow::Context,
        destination: Option<glow::Framebuffer>,
        info: &PaintCallbackInfo,
        capture: Capture,
        region: GlassRegion,
    ) -> bool {
        unsafe {
            let scale = info.pixels_per_point;
            let optics = Optics::new(region);
            gl.active_texture(glow::TEXTURE0);
            gl.disable(glow::SCISSOR_TEST);
            let filtered = self.filter_backdrop(gl, destination, capture, &optics, scale);
            gl.bind_framebuffer(glow::FRAMEBUFFER, destination);
            gl.enable(glow::SCISSOR_TEST);
            let Some((surface, [w, h])) = filtered else {
                return false;
            };
            let viewport = info.viewport_in_pixels();
            gl.viewport(
                viewport.left_px,
                viewport.from_bottom_px,
                viewport.width_px,
                viewport.height_px,
            );
            gl.bind_texture(glow::TEXTURE_2D, Some(surface.texture));

            let [texture, bounds, corners, captured, texels, effect, strength] = &self.uniforms;
            let center = region.rect.center();
            gl.use_program(Some(self.program));
            gl.uniform_1_i32(texture.as_ref(), 0);
            gl.uniform_4_f32(
                bounds.as_ref(),
                center.x * scale,
                info.screen_size_px[1] as f32 - center.y * scale,
                region.rect.width() * scale * 0.5,
                region.rect.height() * scale * 0.5,
            );
            gl.uniform_4_f32(
                corners.as_ref(),
                region.corners.nw * scale,
                region.corners.ne * scale,
                region.corners.sw * scale,
                region.corners.se * scale,
            );
            gl.uniform_4_f32(
                captured.as_ref(),
                capture.x as f32,
                capture.y as f32,
                w as f32 / capture.width as f32,
                h as f32 / capture.height as f32,
            );
            gl.uniform_4_f32(
                texels.as_ref(),
                w as f32,
                h as f32,
                surface.size[0] as f32,
                surface.size[1] as f32,
            );
            gl.uniform_3_f32(
                effect.as_ref(),
                optics.lens * scale,
                optics.bevel * scale,
                optics.polish,
            );
            gl.uniform_1_f32(strength.as_ref(), region.strength);
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            // The painter restores its program, VAO and viewport after the callback.
            true
        }
    }

    unsafe fn filter_backdrop(
        &mut self,
        gl: &glow::Context,
        destination: Option<glow::Framebuffer>,
        capture: Capture,
        optics: &Optics,
        scale: f32,
    ) -> Option<(Surface, [i32; 2])> {
        unsafe {
            let mut extent = [capture.width, capture.height];
            self.full.reserve(gl, extent[0], extent[1]);
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.target));
            if !Self::attach(gl, glow::DRAW_FRAMEBUFFER, self.full.texture) {
                return None;
            }
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, destination);
            gl.blit_framebuffer(
                capture.x,
                capture.y,
                capture.x + extent[0],
                capture.y + extent[1],
                0,
                0,
                extent[0],
                extent[1],
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );

            let mut sigma = optics.sigma.map(|value| value * scale);
            if sigma[0].max(sigma[1]) < 0.55 {
                return (gl.get_error() == glow::NO_ERROR).then_some((self.full, extent));
            }

            // A half-size linear blit averages adjacent texels in both axes.
            // Build the reduction in steps so narrow details survive without aliasing.
            let mut source = self.full;
            let mut next = 0;
            while sigma[0].max(sigma[1]) > 4.25 && extent[0].min(extent[1]) >= 4 {
                let half = extent.map(|edge| (edge + 1) / 2);
                self.reduced[next].reserve(gl, half[0], half[1]);
                let target = self.reduced[next];
                gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.read));
                if !Self::attach(gl, glow::READ_FRAMEBUFFER, source.texture)
                    || !Self::attach(gl, glow::DRAW_FRAMEBUFFER, target.texture)
                {
                    return None;
                }
                gl.blit_framebuffer(
                    0,
                    0,
                    extent[0],
                    extent[1],
                    0,
                    0,
                    half[0],
                    half[1],
                    glow::COLOR_BUFFER_BIT,
                    glow::LINEAR,
                );
                for axis in 0..2 {
                    let ratio = half[axis] as f32 / extent[axis] as f32;
                    // Each downsample already smooths the image slightly.
                    sigma[axis] = (sigma[axis].powi(2) - 0.25).max(0.0).sqrt() * ratio;
                }
                source = target;
                extent = half;
                next = 1 - next;
            }
            self.reduced[next].reserve(gl, extent[0], extent[1]);
            let scratch = self.reduced[next];
            gl.disable(glow::BLEND);
            gl.viewport(0, 0, extent[0], extent[1]);
            gl.bind_vertex_array(Some(self.vao));
            let horizontal = Self::attach(gl, glow::DRAW_FRAMEBUFFER, scratch.texture);
            if horizontal {
                self.filter.prepare(gl, extent, source.size);
                self.filter
                    .pass(gl, source.texture, [1.0, 0.0], Kernel::new(sigma[0]));
            }
            let vertical = horizontal && Self::attach(gl, glow::DRAW_FRAMEBUFFER, source.texture);
            if vertical {
                self.filter.prepare(gl, extent, scratch.size);
                self.filter
                    .pass(gl, scratch.texture, [0.0, 1.0], Kernel::new(sigma[1]));
            }
            gl.enable(glow::BLEND);
            (vertical && gl.get_error() == glow::NO_ERROR).then_some((source, extent))
        }
    }

    unsafe fn attach(gl: &glow::Context, framebuffer: u32, texture: glow::Texture) -> bool {
        unsafe {
            gl.framebuffer_texture_2d(
                framebuffer,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );
            gl.check_framebuffer_status(framebuffer) == glow::FRAMEBUFFER_COMPLETE
        }
    }

    unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_framebuffer(self.target);
            gl.delete_framebuffer(self.read);
            gl.delete_texture(self.full.texture);
            for surface in self.reduced {
                gl.delete_texture(surface.texture);
            }
            gl.delete_program(self.filter.program);
            gl.delete_program(self.program);
        }
    }
}

#[derive(Clone, Copy)]
struct Kernel {
    center: f32,
    offsets: [f32; 4],
    weights: [f32; 4],
}

impl Kernel {
    fn new(sigma: f32) -> Self {
        let factor = -0.5 / sigma.max(0.7).powi(2);
        let mut normalization = 1.0;
        let mut offsets = [0.0; 4];
        let mut weights = [0.0; 4];
        for (pair, (offset, weight)) in offsets.iter_mut().zip(&mut weights).enumerate() {
            let first = (2 * pair + 1) as f32;
            let a = (factor * first.powi(2)).exp();
            let b = (factor * (first + 1.0).powi(2)).exp();
            *weight = a + b;
            // Linear sampling evaluates both adjacent Gaussian terms in one read.
            *offset = first + b / *weight;
            normalization += 2.0 * *weight;
        }
        Self {
            center: normalization.recip(),
            offsets,
            weights: weights.map(|weight| weight / normalization),
        }
    }
}

struct Filter {
    program: glow::Program,
    uniforms: [Option<glow::UniformLocation>; 6],
}

impl Filter {
    unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            let program = program(gl, FILTER)?;
            let uniforms = [
                "u_texture",
                "u_texels",
                "u_axis",
                "u_center",
                "u_offsets",
                "u_weights",
            ]
            .map(|name| gl.get_uniform_location(program, name));
            Ok(Self { program, uniforms })
        }
    }

    unsafe fn prepare(&self, gl: &glow::Context, samples: [i32; 2], texture_size: [i32; 2]) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_1_i32(self.uniforms[0].as_ref(), 0);
            gl.uniform_4_f32(
                self.uniforms[1].as_ref(),
                samples[0] as f32,
                samples[1] as f32,
                texture_size[0] as f32,
                texture_size[1] as f32,
            );
        }
    }

    unsafe fn pass(
        &self,
        gl: &glow::Context,
        source: glow::Texture,
        axis: [f32; 2],
        kernel: Kernel,
    ) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(source));
            gl.uniform_2_f32(self.uniforms[2].as_ref(), axis[0], axis[1]);
            gl.uniform_1_f32(self.uniforms[3].as_ref(), kernel.center);
            gl.uniform_4_f32_slice(self.uniforms[4].as_ref(), &kernel.offsets);
            gl.uniform_4_f32_slice(self.uniforms[5].as_ref(), &kernel.weights);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}

unsafe fn program(gl: &glow::Context, fragment: &str) -> Result<glow::Program, String> {
    unsafe {
        let header = if gl.version().is_embedded {
            "#version 300 es\nprecision highp float;\n"
        } else {
            "#version 330 core\n"
        };
        let program = gl.create_program()?;
        let result = (|| {
            for (kind, source) in [
                (glow::VERTEX_SHADER, VERTEX),
                (glow::FRAGMENT_SHADER, fragment),
            ] {
                let shader = gl.create_shader(kind)?;
                gl.shader_source(shader, &format!("{header}{source}"));
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    let error = gl.get_shader_info_log(shader);
                    gl.delete_shader(shader);
                    return Err(error);
                }
                gl.attach_shader(program, shader);
                gl.delete_shader(shader);
            }
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(gl.get_program_info_log(program));
            }
            Ok(program)
        })();
        if result.is_err() {
            gl.delete_program(program);
        }
        result
    }
}

const VERTEX: &str = "
void main() {
    vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}
";

const FRAGMENT: &str = include_str!("backdrop_gl.frag");

const FILTER: &str = include_str!("backdrop_blur.frag");

#[cfg(test)]
mod tests {
    use super::Kernel;

    #[test]
    fn blur_kernel_keeps_uniform_color_and_has_no_secondary_peaks() {
        for sigma in [0.0, 1.5, 3.0, 4.0, 8.0] {
            let kernel = Kernel::new(sigma);
            let mut previous = kernel.center;
            let mut total = kernel.center;
            for (pair, (&offset, &weight)) in kernel.offsets.iter().zip(&kernel.weights).enumerate()
            {
                let first = (2 * pair + 1) as f32;
                let fraction = offset - first;
                assert!((0.0..=1.0).contains(&fraction));
                let near = weight * (1.0 - fraction);
                let far = weight * fraction;
                assert!(previous >= near && near >= far);
                previous = far;
                total += 2.0 * weight;
            }
            assert!((total - 1.0).abs() < 1e-6);
        }
    }
}
