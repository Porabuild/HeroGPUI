//! Gallery asset source — embeds the HeroGPUI icon set and sample image.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

pub struct Assets;

macro_rules! assets {
    ($($name:literal => $file:literal),* $(,)?) => {
        const EMBEDDED: &[(&str, &[u8])] = &[
            $(($name, include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $file)))),*
        ];

        impl AssetSource for Assets {
            fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
                match EMBEDDED.iter().find(|(p, _)| *p == path) {
                    Some((_, data)) => Ok(Some(Cow::Borrowed(*data))),
                    None => Ok(None),
                }
            }

            fn list(&self, path: &str) -> Result<Vec<SharedString>> {
                Ok(EMBEDDED
                    .iter()
                    .filter(|(p, _)| p.starts_with(path))
                    .map(|(p, _)| SharedString::from(*p))
                    .collect())
            }
        }
    };
}

assets! {
    "herogpui/icons/check.svg" => "herogpui/icons/check.svg",
    "herogpui/icons/calendar.svg" => "herogpui/icons/calendar.svg",
    "herogpui/icons/chevron_down.svg" => "herogpui/icons/chevron_down.svg",
    "herogpui/icons/chevron_up.svg" => "herogpui/icons/chevron_up.svg",
    "herogpui/icons/chevron_left.svg" => "herogpui/icons/chevron_left.svg",
    "herogpui/icons/chevron_right.svg" => "herogpui/icons/chevron_right.svg",
    "herogpui/icons/close.svg" => "herogpui/icons/close.svg",
    "herogpui/icons/search.svg" => "herogpui/icons/search.svg",
    "herogpui/icons/moon.svg" => "herogpui/icons/moon.svg",
    "herogpui/icons/sun.svg" => "herogpui/icons/sun.svg",
    "herogpui/icons/dots_vertical.svg" => "herogpui/icons/dots_vertical.svg",
    "herogpui/icons/plus.svg" => "herogpui/icons/plus.svg",
    "herogpui/icons/minus.svg" => "herogpui/icons/minus.svg",
    "herogpui/icons/eye.svg" => "herogpui/icons/eye.svg",
    "herogpui/icons/eye_off.svg" => "herogpui/icons/eye_off.svg",
    "herogpui/icons/copy.svg" => "herogpui/icons/copy.svg",
    "herogpui/icons/external_link.svg" => "herogpui/icons/external_link.svg",
    "herogpui/icons/arrow_left.svg" => "herogpui/icons/arrow_left.svg",
    "herogpui/icons/arrow_right.svg" => "herogpui/icons/arrow_right.svg",
    "herogpui/icons/ellipsis.svg" => "herogpui/icons/ellipsis.svg",
    "herogpui/icons/mail.svg" => "herogpui/icons/mail.svg",
    "herogpui/icons/key.svg" => "herogpui/icons/key.svg",
    "herogpui/icons/globe.svg" => "herogpui/icons/globe.svg",
    "herogpui/icons/heart.svg" => "herogpui/icons/heart.svg",
    "herogpui/icons/heart_fill.svg" => "herogpui/icons/heart_fill.svg",
    "herogpui/icons/close_circle.svg" => "herogpui/icons/close_circle.svg",
    "herogpui/icons/trash.svg" => "herogpui/icons/trash.svg",
    "herogpui/icons/gear.svg" => "herogpui/icons/gear.svg",
    "herogpui/icons/spinner.svg" => "herogpui/icons/spinner.svg",
    "herogpui/icons/tooltip_arrow.svg" => "herogpui/icons/tooltip_arrow.svg",
    "herogpui/icons/alert_triangle.svg" => "herogpui/icons/alert_triangle.svg",
    "herogpui/icons/info_circle.svg" => "herogpui/icons/info_circle.svg",
    "herogpui/icons/check_circle.svg" => "herogpui/icons/check_circle.svg",
    "herogpui/icons/circle_exclamation.svg" => "herogpui/icons/circle_exclamation.svg",
    "herogpui/icons/warning_triangle.svg" => "herogpui/icons/warning_triangle.svg",
    "herogpui/sample.png" => "herogpui/sample.png",
}
