//! The editor state itself, sans causal snapshot system.
use crate::compute::Compute;
use crate::surface::Surface;
use crate::winit::{ModalContext, ModalEditor, ModalEvent};

#[derive(Default)]
pub struct Editor {
    close_requested: bool,
    num_frames: usize,
}

impl ModalEditor for Editor {
    type Compute = Compute;
    type Surface = Surface;

    fn event(&mut self, ev: ModalEvent, _: &mut ModalContext) {
        log::info!("{:?}", ev);
    }

    fn redraw_request(&mut self, surface: &mut Surface) {
        if let Err(err) = self.draw_to_surface(surface) {
            self.drawn_error(err, surface);
        }

        self.num_frames += 1;
        self.close_requested |= self.num_frames >= 500;
    }

    fn exit(&self) -> bool {
        self.close_requested
    }
}

impl Editor {
    pub fn draw_to_surface(&mut self, surface: &mut Surface) -> Result<(), wgpu::SurfaceError> {
        let start = std::time::Instant::now();
        let full_start = start;
        let mut texture = surface.get_current_texture()?;
        let end = std::time::Instant::now();
        log::warn!("Time get texture{:?}", end.saturating_duration_since(start));
        let start = end;
        surface.present_to_texture(&mut texture);
        let end = std::time::Instant::now();
        log::warn!(
            "Time rendering total {:?}",
            end.saturating_duration_since(start)
        );
        texture.present();
        let end = std::time::Instant::now();
        log::warn!(
            "Time present total {:?}",
            end.saturating_duration_since(full_start)
        );
        Ok(())
    }

    pub fn drawn_error(&mut self, err: wgpu::SurfaceError, surface: &mut Surface) {
        match err {
            wgpu::SurfaceError::Lost => surface.lost(),
            wgpu::SurfaceError::OutOfMemory => self.close_requested = true,
            e => log::warn!("{:?}", e),
        }
    }
}
