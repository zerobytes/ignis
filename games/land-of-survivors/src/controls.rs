//! Game bindings over the native PlayerView/input contract.
use ignis::{
    physics::Input,
    player_view::{Config, Mode, ThirdPerson},
};
use winit::event::MouseButton;

#[derive(Debug)]
pub struct Controls {
    pub view: Config,
    pub heading_degrees: f32,
    left: bool,
    right: bool,
}
impl Default for Controls {
    fn default() -> Self {
        Self {
            view: Config {
                actor: "player0".into(),
                yaw_degrees: 0.,
                pitch_degrees: -12.,
                eye_height: 1.25,
                fov_degrees: 55.,
                mode: Mode::ThirdPerson,
                third_person: ThirdPerson {
                    shoulder_offset: 0.2,
                    ..Default::default()
                },
            },
            heading_degrees: 0.,
            left: false,
            right: false,
        }
    }
}
impl Controls {
    pub fn dragging(&self) -> bool {
        self.left || self.right
    }
    pub fn button(&mut self, button: MouseButton, pressed: bool, blocked: bool) -> bool {
        if pressed && blocked {
            return false;
        }
        match button {
            MouseButton::Left => self.left = pressed,
            MouseButton::Right => {
                self.right = pressed;
                if pressed {
                    self.heading_degrees = self.view.yaw_degrees;
                }
            }
            _ => return false,
        }
        true
    }
    pub fn motion(&mut self, dx: f64, dy: f64) -> bool {
        if !self.dragging() || !dx.is_finite() || !dy.is_finite() {
            return false;
        }
        self.view.yaw_degrees =
            (self.view.yaw_degrees + (dx as f32 * 0.12).clamp(-180., 180.)).rem_euclid(360.);
        self.view.pitch_degrees =
            (self.view.pitch_degrees - (dy as f32 * 0.12).clamp(-180., 180.)).clamp(-75., 75.);
        if self.right {
            self.heading_degrees = self.view.yaw_degrees;
        }
        true
    }
    pub fn zoom(&mut self, steps: f32) {
        if steps.is_finite() {
            self.view.third_person.distance_meters =
                (self.view.third_person.distance_meters - steps * 0.5).clamp(1., 20.);
        }
    }
    pub fn release(&mut self) {
        self.left = false;
        self.right = false;
    }
    pub fn world_input(&self, input: Input) -> Input {
        let mut direction = self.view.clone();
        direction.yaw_degrees = self.heading_degrees;
        direction.world_input(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn orbit_turn_ui_capture_and_focus_have_distinct_effects() {
        let mut c = Controls::default();
        let forward = Input {
            x: 0.,
            z: -1.,
            ..Default::default()
        };
        assert!(!c.button(MouseButton::Left, true, true));
        assert!(!c.motion(500., 100.));
        c.button(MouseButton::Left, true, false);
        assert!(c.motion(750., 0.));
        assert_eq!(c.view.yaw_degrees, 90.);
        assert_eq!(c.heading_degrees, 0.);
        assert_eq!(c.world_input(forward).z, -1.);
        c.button(MouseButton::Right, true, false);
        assert_eq!(c.heading_degrees, 90.);
        let input = c.world_input(forward);
        assert!((input.x - 1.).abs() < 0.00001 && input.z.abs() < 0.00001);
        c.motion(20., 20.);
        assert_eq!(c.heading_degrees, c.view.yaw_degrees);
        c.button(MouseButton::Right, false, true);
        let heading = c.heading_degrees;
        c.motion(100., 100.);
        assert_eq!(heading, c.heading_degrees);
        c.release();
        assert!(!c.dragging() && !c.motion(100., 100.));
        c.zoom(1000.);
        assert_eq!(c.view.third_person.distance_meters, 1.);
        c.view.validate().unwrap();
    }
}
