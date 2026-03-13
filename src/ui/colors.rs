//! Gruvebox Material color palette constants.
//!
//! All values are expressed as `bevy::prelude::Color` using sRGB u8 components.
//! This module is constants-only — no public functions, so no `#[tracing::instrument]`
//! annotations are needed.

use bevy::prelude::Color;

// ── Backgrounds ──────────────────────────────────────────────────────────────

pub const BG_DIM: Color = Color::srgb_u8(0x1B, 0x1B, 0x1B);
pub const BG0: Color = Color::srgb_u8(0x28, 0x28, 0x28);
pub const BG1: Color = Color::srgb_u8(0x32, 0x30, 0x2F);
pub const BG2: Color = Color::srgb_u8(0x32, 0x30, 0x2F);
pub const BG3: Color = Color::srgb_u8(0x45, 0x40, 0x3D);
pub const BG4: Color = Color::srgb_u8(0x45, 0x40, 0x3D);
pub const BG5: Color = Color::srgb_u8(0x5A, 0x52, 0x4C);

// ── Statusline backgrounds ────────────────────────────────────────────────────

pub const BG_STATUSLINE1: Color = Color::srgb_u8(0x32, 0x30, 0x2F);
pub const BG_STATUSLINE2: Color = Color::srgb_u8(0x3A, 0x37, 0x35);
pub const BG_STATUSLINE3: Color = Color::srgb_u8(0x50, 0x49, 0x45);

// ── Special backgrounds ───────────────────────────────────────────────────────

pub const BG_CURRENT_WORD: Color = Color::srgb_u8(0x3C, 0x38, 0x36);

// ── Diff backgrounds ─────────────────────────────────────────────────────────

pub const BG_DIFF_RED: Color = Color::srgb_u8(0x40, 0x21, 0x20);
pub const BG_DIFF_GREEN: Color = Color::srgb_u8(0x34, 0x38, 0x1B);
pub const BG_DIFF_BLUE: Color = Color::srgb_u8(0x0E, 0x36, 0x3E);

// ── Visual selection backgrounds ─────────────────────────────────────────────

pub const BG_VISUAL_RED: Color = Color::srgb_u8(0x4C, 0x34, 0x32);
pub const BG_VISUAL_GREEN: Color = Color::srgb_u8(0x3B, 0x44, 0x39);
pub const BG_VISUAL_BLUE: Color = Color::srgb_u8(0x37, 0x41, 0x41);
pub const BG_VISUAL_YELLOW: Color = Color::srgb_u8(0x4F, 0x42, 0x2E);
pub const BG_VISUAL_PURPLE: Color = Color::srgb_u8(0x44, 0x38, 0x40);

// ── Foregrounds ───────────────────────────────────────────────────────────────

pub const FG0: Color = Color::srgb_u8(0xD4, 0xBE, 0x98);
pub const FG1: Color = Color::srgb_u8(0xDD, 0xC7, 0xA1);

// ── Accent colors ─────────────────────────────────────────────────────────────

pub const RED: Color = Color::srgb_u8(0xEA, 0x69, 0x62);
pub const GREEN: Color = Color::srgb_u8(0xA9, 0xB6, 0x65);
pub const BLUE: Color = Color::srgb_u8(0x7D, 0xAE, 0xA3);
pub const YELLOW: Color = Color::srgb_u8(0xD8, 0xA6, 0x57);
pub const PURPLE: Color = Color::srgb_u8(0xD3, 0x86, 0x9B);
pub const ORANGE: Color = Color::srgb_u8(0xE7, 0x8A, 0x4E);
pub const AQUA: Color = Color::srgb_u8(0x89, 0xB4, 0x82);

// ── Greys ─────────────────────────────────────────────────────────────────────

pub const GREY0: Color = Color::srgb_u8(0x7C, 0x6F, 0x64);
pub const GREY1: Color = Color::srgb_u8(0x92, 0x83, 0x74);
pub const GREY2: Color = Color::srgb_u8(0xA8, 0x99, 0x84);

// ── Background accent tones ───────────────────────────────────────────────────

pub const BG_RED: Color = Color::srgb_u8(0xEA, 0x69, 0x62);
pub const BG_GREEN: Color = Color::srgb_u8(0xA9, 0xB6, 0x65);
pub const BG_YELLOW: Color = Color::srgb_u8(0xD8, 0xA6, 0x57);
