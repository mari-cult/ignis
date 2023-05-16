use {
    serde::{Deserialize, Serialize},
    std::collections::BTreeSet,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Serialized {
    sources: BTreeSet<String>,
    parts: Vec<Part>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Part {
    uses: PartKind,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    depends: BTreeSet<String>,
    artifacts: BTreeSet<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PartKind {
    Rust,
    CCpp,
    Copy,
    Zig,
}

fn main() {
    let serialized = Serialized {
        sources: vec![
            "github:ka1mari/steam-for-linux".into(),
            "steam:client".into(),
        ]
        .into_iter()
        .collect(),
        parts: vec![
            Part {
                uses: PartKind::Rust,
                depends: vec![].into_iter().collect(),
                artifacts: vec!["bin steam".into()].into_iter().collect(),
            },
            Part {
                uses: PartKind::Copy,
                depends: vec!["fontconfig".into(), "sdl3".into()]
                    .into_iter()
                    .collect(),
                artifacts: vec!["bin steam".into()].into_iter().collect(),
            },
            Part {
                uses: PartKind::Copy,
                depends: vec!["sdl3".into(), "x11".into()].into_iter().collect(),
                artifacts: vec!["bin steam".into()].into_iter().collect(),
            },
            Part {
                uses: PartKind::CCpp,
                depends: vec!["sdl3".into(), "x11".into()].into_iter().collect(),
                artifacts: vec!["bin steam".into()].into_iter().collect(),
            },
        ],
    };

    println!("{}", serde_yaml::to_string(&serialized).unwrap());
}
