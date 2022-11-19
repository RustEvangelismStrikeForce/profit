use std::cmp::Ordering;

use profit_sim as sim;
use sim::{place_building, pos, Board, Building, Product, Products, ResourceType, Resources, Sim};

use crate::{find_regions, possible_products_per_region, Regions};

#[test]
fn find_two_clusters() {
    let mut sim = Sim::new(Products::default(), Board::new(6, 6));

    place_building(&mut sim, Building::obstacle(pos(3, 0), 1, 6)).unwrap();

    let mut regions = find_regions(&sim);

    for i in 0..regions.len() {
        let r = regions.get_mut(i);
        r.cells.sort_by(|a, b| match a.y.cmp(&b.y) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => a.x.cmp(&b.x),
        });
    }

    #[rustfmt::skip]
    let expected = Regions {
        buildings: vec![],
        cells: vec![
            // first
            pos(0, 0), pos(1, 0), pos(2, 0),
            pos(0, 1), pos(1, 1), pos(2, 1),
            pos(0, 2), pos(1, 2), pos(2, 2),
            pos(0, 3), pos(1, 3), pos(2, 3),
            pos(0, 4), pos(1, 4), pos(2, 4),
            pos(0, 5), pos(1, 5), pos(2, 5),
            // second
            pos(4, 0), pos(5, 0),
            pos(4, 1), pos(5, 1),
            pos(4, 2), pos(5, 2),
            pos(4, 3), pos(5, 3),
            pos(4, 4), pos(5, 4),
            pos(4, 5), pos(5, 5),
        ],
        indices: vec![(0, 0), (0, 18)],
    };

    assert_eq!(regions, expected,);
}

#[test]
fn one_possible_product() {
    let mut products = Products::default();
    products[0] = Product::new(Resources::new([4, 0, 0, 0, 0, 0, 0, 0]), 10);
    let mut sim = Sim::new(products, Board::new(20, 6));
    place_building(&mut sim, Building::obstacle((3, 0), 1, 6)).unwrap();
    place_building(
        &mut sim,
        Building::deposit((5, 0), ResourceType::Type0, 2, 2),
    )
    .unwrap();

    let regions = find_regions(&sim);

    let possible_products = possible_products_per_region(&sim, &regions);
    assert_eq!(possible_products, [0x00, 0b0000_0001]);
}
