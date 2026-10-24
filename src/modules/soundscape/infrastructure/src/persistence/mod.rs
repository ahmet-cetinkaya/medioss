pub mod category_repository;
pub mod mix_repository;
pub mod sound_repository;
pub use category_repository::SeaOrmCategoryRepository;
pub use mix_repository::SeaOrmMixRepository;
pub use sound_repository::SeaOrmSoundRepository;

pub(crate) fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in name.trim().chars().flat_map(|c| c.to_lowercase()) {
        let c = match c {
            'ç' => 'c',
            'ğ' => 'g',
            'ı' => 'i',
            'ö' => 'o',
            'ş' => 's',
            'ü' => 'u',
            x if x.is_ascii_alphanumeric() => x,
            _ => '-',
        };
        if c == '-' {
            if !out.is_empty() && !dash {
                out.push(c);
                dash = true;
            }
        } else {
            out.push(c);
            dash = false;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "category".into()
    } else {
        out
    }
}
