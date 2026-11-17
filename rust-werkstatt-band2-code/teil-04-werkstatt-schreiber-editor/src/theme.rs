use iced::widget::container;
use iced::{Border, Color};

pub(crate) const HINTERGRUND: Color =
    Color::from_rgb(0.06, 0.08, 0.11);
pub(crate) const SEITENLEISTE: Color =
    Color::from_rgb(0.10, 0.13, 0.18);
pub(crate) const EDITOR_HINTERGRUND: Color =
    Color::from_rgb(0.08, 0.10, 0.14);
pub(crate) const TEXT_HAUPT: Color =
    Color::from_rgb(0.95, 0.96, 0.98);
pub(crate) const TEXT_GEDIKTET: Color =
    Color::from_rgb(0.58, 0.64, 0.72);
pub(crate) const AKZENT_GOLD: Color =
    Color::from_rgb(0.96, 0.62, 0.07);
pub(crate) const TRENNLINIE: Color =
    Color::from_rgb(0.18, 0.22, 0.29);

pub(crate) fn seitenleisten_stil(_: &()) -> container::Style {
    container::Style {
        background: Some(SEITENLEISTE.into()),
        border: Border {
            color: TRENNLINIE,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
