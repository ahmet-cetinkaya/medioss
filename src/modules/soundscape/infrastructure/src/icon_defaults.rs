/// Bundled-glyph key a sound gets by default: first a whole-word match on the
/// name, then the category slug. Shared by the built-in seeder and the icon
/// migration so fresh and existing instances agree.
pub fn default_icon_key(category_slug: &str, name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    let by_word = lower
        .split(|c: char| !c.is_alphanumeric())
        .find_map(|word| match word {
            "thunder" => Some("thunder"),
            "rain" => Some("rain"),
            "train" | "subway" | "traffic" | "highway" | "street" | "siren" => Some("city"),
            "wind" => Some("wind"),
            "fire" | "campfire" | "fireworks" => Some("fire"),
            "water" | "waterfall" | "waves" | "river" | "droplets" | "bubbles" | "underwater" => {
                Some("water")
            }
            "night" => Some("night"),
            "jungle" | "forest" | "leaves" | "trees" => Some("forest"),
            "bird" | "birds" => Some("birds"),
            "noise" | "cafe" | "restaurant" => Some("noise"),
            _ => None,
        });
    by_word.or(match category_slug {
        "rain" => Some("rain"),
        "nature" => Some("forest"),
        "animals" => Some("birds"),
        "binaural" => Some("heartbeat"),
        "noise" | "places" | "mechanical" => Some("noise"),
        "transport" | "urban" => Some("city"),
        "things" | "ambient" => Some("music"),
        _ => None,
    })
}

#[cfg(test)]
#[path = "icon_defaults.tests.rs"]
mod tests;
