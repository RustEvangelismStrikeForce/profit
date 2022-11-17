use std::array;
use std::cmp::Ordering;

use profit_sim as sim;
use sim::{pos, Board, Building, BuildingKind, Obstacle, Product, Resources, Sim};

use crate::{find_clusters, Cluster};

#[test]
fn find_two_clusters() {
    let products = array::from_fn(|i| Product::new(Resources::default(), i as u32));
    let mut sim = Sim::new(products, Board::new(6, 6));

    sim::place_building(
        &mut sim,
        Building::new(pos(3, 0), BuildingKind::Obstacle(Obstacle::new(1, 6))),
    )
    .unwrap();

    let mut clusters = find_clusters(&sim);

    for c in &mut clusters {
        c.cells.sort_by(|a, b| match a.y.cmp(&b.y) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => a.x.cmp(&b.x),
        });
    }

    #[rustfmt::skip]
    assert_eq!(
        clusters,
        vec![
            Cluster::new(Vec::new(), vec![
                 pos(0, 0), pos(1, 0), pos(2, 0),
                 pos(0, 1), pos(1, 1), pos(2, 1),
                 pos(0, 2), pos(1, 2), pos(2, 2),
                 pos(0, 3), pos(1, 3), pos(2, 3),
                 pos(0, 4), pos(1, 4), pos(2, 4),
                 pos(0, 5), pos(1, 5), pos(2, 5),
            ]),
            Cluster::new(Vec::new(), vec![
                 pos(4, 0), pos(5, 0),
                 pos(4, 1), pos(5, 1),
                 pos(4, 2), pos(5, 2),
                 pos(4, 3), pos(5, 3),
                 pos(4, 4), pos(5, 4),
                 pos(4, 5), pos(5, 5),
            ]),
        ]
    );
}
