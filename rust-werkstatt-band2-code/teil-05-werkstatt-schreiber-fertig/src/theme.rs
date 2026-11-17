use iced::widget::container;
use iced::{Border, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ThemaModus {
    #[default]
    Dunkel,
    Hell,
}

impl ThemaModus {
    pub(crate) fn umschalten(&self) -> Self {
        match self {
            Self::Dunkel => Self::Hell,
            Self::Hell => Self::Dunkel,
        }
    }

    pub(crate) fn hintergrund(&self) -> Color {
        match self {
            Self::Dunkel => Color::from_rgb(0.06, 0.08, 0.11),
            Self::Hell => Color::from_rgb(0.97, 0.98, 0.99),
        }
    }

    pub(crate) fn seitenleiste(&self) -> Color {
        match self {
            Self::Dunkel => Color::from_rgb(0.10, 0.13, 0.18),
            Self::Hell => Color::from_rgb(0.92, 0.94, 0.97),
        }
    }

    pub(crate) fn editor_hintergrund(&self) -> Color {
        match self {
            Self::Dunkel => Color::from_rgb(0.08, 0.10, 0.14),
            Self::Hell => Color::from_rgb(1.0, 1.0, 1.0),
        }
    }

    pub(crate) fn text_haupt(&self) -> Color {
        match self {
            Self::Dunkel => Color::from_rgb(0.95, 0.96, 0.98),
            Self::Hell => Color::from_rgb(0.09, 0.12, 0.17),
        }
    }

    pub(crate) fn text_gedimmt(&self) -> Color {
        match self {
            Self::Dunkel => Color::from_rgb(0.58, 0.64, 0.72),
            Self::Hell => Color::from_rgb(0.40, 0.46, 0.54),
        }
    }

    pub(crate) fn akzent_gold(&self) -> Color {
        Color::from_rgb(0.96, 0.62, 0.07)
    }

    pub(crate) fn trennlinie(&self) -> Color {
        match self {
            Self::Dunkel => Color::from_rgb(0.18, 0.22, 0.29),
            Self::Hell => Color::from_rgb(0.82, 0.86, 0.90),
        }
    }
}

pub(crate) fn seitenleisten_stil(modus: ThemaModus) -> container::Style {
    container::Style {
        background: Some(modus.seitenleiste().into()),
        border: Border {
            color: modus.trennlinie(),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
