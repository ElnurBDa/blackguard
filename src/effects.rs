//! Lightweight, self-contained animations via `tachyonfx` (0.19, pinned to
//! ratatui 0.29).
//!
//! tachyonfx effects are *post-processing*: you render your widgets into the
//! buffer first, then `process()` an effect over the same region to mutate
//! those cells. `process`/`done` are *inherent* methods on `Effect` (no
//! `Shader` import needed), and `process` takes tachyonfx's own `Duration`, so
//! we convert the frame's `std::time::Duration` delta with `.into()`.

use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use tachyonfx::{Effect, EffectTimer, Interpolation, fx};

/// The transient effects overlaid on the base UI.
#[derive(Default)]
pub struct Fx {
    damage_flash: Option<Effect>,
    room_reveal: Option<Effect>,
}

impl Fx {
    /// Flash red over the HP readout when the player takes damage, resolving
    /// back to the real colours.
    pub fn flash_damage(&mut self) {
        self.damage_flash = Some(fx::fade_from(
            Color::Rgb(255, 90, 90),
            Color::Rgb(80, 20, 24),
            EffectTimer::from_ms(360, Interpolation::Linear),
        ));
    }

    /// Materialise a freshly dealt room (a "coalesce" reveal).
    pub fn reveal_room(&mut self) {
        self.room_reveal = Some(fx::coalesce(EffectTimer::from_ms(
            450,
            Interpolation::Linear,
        )));
    }

    /// Whether any effect is still animating — drives the frame tick rate.
    pub fn active(&self) -> bool {
        running(&self.damage_flash) || running(&self.room_reveal)
    }

    /// Advance and apply the effects onto `buf` over their target regions.
    pub fn process(&mut self, buf: &mut Buffer, hp_area: Rect, room_area: Rect, elapsed: Duration) {
        // tachyonfx::Duration under default features is a custom { ms: u32 }
        // type; convert the std delta (From<std::time::Duration> exists).
        let dt: tachyonfx::Duration = elapsed.into();
        if let Some(e) = self.damage_flash.as_mut() {
            if !e.done() {
                e.process(dt, buf, hp_area);
            }
        }
        if let Some(e) = self.room_reveal.as_mut() {
            if !e.done() {
                e.process(dt, buf, room_area);
            }
        }
    }

    /// Release finished effects so the loop can idle again.
    pub fn gc(&mut self) {
        if self.damage_flash.as_ref().is_some_and(|e| e.done()) {
            self.damage_flash = None;
        }
        if self.room_reveal.as_ref().is_some_and(|e| e.done()) {
            self.room_reveal = None;
        }
    }
}

fn running(e: &Option<Effect>) -> bool {
    e.as_ref().is_some_and(|e| !e.done())
}
