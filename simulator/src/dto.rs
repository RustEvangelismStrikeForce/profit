use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Object {
    #[serde(rename = "type")]
    kind: ObjectKind,
    #[serde(default)]
    subtype: u8,
    x: usize,
    y: usize,
    width: Option<usize>,
    height: Option<usize>,
    resources: [u16; 8],
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ObjectKind {
    Deposit,
    Obstacle,
    Conveyor,
    Combiner,
    Mine,
    Factory,
}

#[derive(Serialize, Deserialize)]
struct Product {
    #[serde(rename = "type")]
    subtype: u8,
    resources: [u16; 8],
    points: usize,
}

struct Game {
    width: u8,
    height: u8,
    objects: Vec<Object>,
    products: Vec<Product>,
}
