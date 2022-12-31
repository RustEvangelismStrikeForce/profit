use std::collections::HashMap;

use profit_sim as sim;
use sim::{
    Building, Combiner, Conveyor, Factory, Id, Mine, Pos, ProductType, ResourceType, Resources,
    Rotation, Sim, FACTORY_SIZE,
};

pub use distance::*;
pub use region::*;

mod distance;
mod region;
#[cfg(test)]
mod test;

struct ProductStats {
    product_type: ProductType,
    deposit_stats: Vec<DepositStats>,
    factory_stats: Vec<FactoryStats>,
}

struct DepositStats {
    id: Id,
    resource_type: ResourceType,
    resources: u16,
    weight: f32,
}

struct FactoryStats {
    pos: Pos,
    score: Score,
    resources_in_reach: Resources,
    /// indices into deposit_stats
    deposits_in_reach: Vec<DepositIdx>,
}

struct DepositIdx {
    idx: usize,
    dist: u16,
}

#[derive(Debug)]
struct WeightedDist {
    dist: f32,
    weighted: f32,
}

#[derive(Debug)]
struct Score {
    dist: f32,
    weighted: f32,
    max_products: f32,
}

// TODO: consider using a `bumpalo` to limit allocations
pub fn solve(sim: &Sim) -> sim::Result<()> {
    let regions = find_regions(sim);
    let deposit_distance_maps = map_deposit_distances(sim);

    for region in regions.iter() {
        let mut available_resources = Resources::default();
        for id in region.deposits.iter() {
            let Building::Deposit(deposit) = &sim.buildings[*id] else { continue };
            available_resources.values[deposit.resource_type as usize] += deposit.resources();
        }

        let product_stats = sim.products.iter()
            .enumerate()
            .filter_map(|(i, product)| {
                if product.points == 0 {
                    return None;
                }
                // filter out products that require more or different resources then there are in the
                // region
                if !available_resources.has_at_least(&product.resources) {
                    return None;
                }

                let product_type = ProductType::try_from(i as u8).unwrap();

                // calculate a weight for a deposit and filter out ones that don't provide any
                // resources needed for the current product
                let deposit_stats = region
                    .deposits
                    .iter()
                    .filter_map(|&id| {
                        let Building::Deposit(deposit) = &sim.buildings[id] else { unreachable!("This should be a deposit") };
                        let resource_type = deposit.resource_type;

                        let needed_resources = product.resources[resource_type];
                        if needed_resources == 0 {
                            return None;
                        }

                        // TODO: possibly factor in if there are other deposits of the same resource
                        // type in the region
                        let resources = deposit.resources();
                        let weight = needed_resources as f32 * resources as f32;

                        Some(DepositStats { id, resource_type, resources, weight })
                    })
                    .collect::<Vec<_>>();

                let mut factory_stats = region
                    .cells
                    .iter()
                    .filter_map(|&pos| {
                        // check if a factory could even be placed here
                        for y in 0..FACTORY_SIZE {
                            for x in 0..FACTORY_SIZE {
                                let p = pos + (x, y);
                                // out of bounds
                                let cell = sim.board.get(p)?;
                                // cell is non-empty
                                if cell.is_some() {
                                    return None;
                                }
                            }
                        }

                        let mut max = WeightedDist { dist: 0.0, weighted: 0.0 };
                        let mut sum = WeightedDist { dist: 0.0, weighted: 0.0 };
                        let mut resources_in_reach = available_resources;
                        let mut deposits_in_reach = Vec::with_capacity(region.deposits.len());
                        for (idx, ds) in deposit_stats.iter().enumerate() {
                            let map = &deposit_distance_maps[&ds.id];
                            // find the distance from the outer border of the factory
                            let mut dist = u16::MAX;
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (i, 0);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 0..FACTORY_SIZE {
                                let pos = pos + (i, FACTORY_SIZE - 1);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 1..FACTORY_SIZE - 1 {
                                let pos = pos + (0, i);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }
                            for i in 1..FACTORY_SIZE - 1 {
                                let pos = pos + (FACTORY_SIZE - 1, i);
                                if let Some(Some(d)) = map.get(pos) {
                                    dist = dist.min(d);
                                }
                            }

                            let deposit_idx = DepositIdx { idx, dist };
                            let dist = dist as f32;
                            let weighted = 1.0 / (dist + 1.0).ln() * ds.weight;

                            max.dist = max.dist.max(dist);
                            max.weighted = max.dist.max(weighted);
                            sum.dist += dist;
                            sum.weighted += weighted;

                            if dist == 0.0 {
                                return None;
                            } else if (dist as u32 / 4) + 2 < sim.turns {
                                deposits_in_reach.push(deposit_idx);
                            } else {
                                resources_in_reach[ds.resource_type] -= ds.resources;
                            }
                        }

                        // Filter out factory positions that can't reach all necessary resources
                        // in time
                        if !resources_in_reach.has_at_least(&product.resources) {
                            return None;
                        }

                        let len = deposit_stats.len() as f32;
                        let avg = WeightedDist { dist: sum.dist / len, weighted: sum.weighted / len };

                        // TODO: calculate some meaningful score
                        let max_products = (resources_in_reach / product.resources).iter().min().unwrap_or_default() as f32;
                        let score = Score {
                            dist: 1.0 / (avg.dist + 1.0).ln() * (max.dist + 1.0).ln(),
                            weighted: avg.weighted * (max.weighted + 1.0).ln(),
                            max_products: 1.0 / (max_products + 2.0).ln(),
                        };

                        Some(FactoryStats { pos, score, resources_in_reach, deposits_in_reach })
                    })
                    .collect::<Vec<_>>();

                // normalize score components
                let mut min_score = Score { dist: f32::MAX, weighted: f32::MAX, max_products: f32::MAX };
                let mut max_score = Score { dist: 0.0, weighted: 0.0, max_products: 0.0 };
                for d in factory_stats.iter() {
                    min_score.dist = min_score.dist.min(d.score.dist);
                    min_score.weighted = min_score.weighted.min(d.score.weighted);
                    min_score.max_products = min_score.max_products.min(d.score.max_products);
                    max_score.dist = max_score.dist.max(d.score.dist);
                    max_score.weighted = max_score.weighted.max(d.score.weighted);
                    max_score.max_products = max_score.max_products.max(d.score.max_products);
                }
                // increase the range by an epsilon to avoid `NaN`s when all scores are the same
                const EPSILON: f32 = 0.001;
                max_score.dist += EPSILON;
                max_score.weighted += EPSILON;
                max_score.max_products += EPSILON;
                factory_stats.iter_mut().for_each(|d| {
                    d.score.dist = (d.score.dist - min_score.dist) / (max_score.dist - min_score.dist);
                    d.score.weighted = (d.score.weighted - min_score.weighted) / (max_score.weighted - min_score.weighted);
                    d.score.max_products = (d.score.max_products - min_score.max_products) / (max_score.max_products - min_score.max_products);
                });

                // rank by score
                factory_stats.sort_by(|f1, f2| {
                    let score1 = f1.score.dist + f1.score.weighted + f1.score.max_products;
                    let score2 = f2.score.dist + f2.score.weighted + f2.score.max_products;
                    score2.total_cmp(&score1)
                });

                Some(ProductStats { product_type, deposit_stats, factory_stats })
            }).collect::<Vec<_>>();

        for (id, m) in deposit_distance_maps.iter() {
            println!("{id:?} {m:?}");
        }
        println!("------------------------------");
        println!("{:?}", region.deposits);
        println!("------------------------------");

        // TODO: calculate search depth dynamically based on some heuristic using time and board size
        let search_depth = 5;
        let mut current_sim = sim.clone();
        // TODO: try out some combinations of factories producing different products and rank those
        // combinations
        for product_stats in product_stats.iter() {
            for factory_stats in product_stats.factory_stats.iter() {
                current_sim.clone_from(sim);

                println!(
                    "{}: {:16}, {:16}, {:16}",
                    factory_stats.pos,
                    factory_stats.score.dist,
                    factory_stats.score.weighted,
                    factory_stats.score.max_products
                );

                let res = connect_deposits_and_factory(
                    &mut current_sim,
                    product_stats,
                    factory_stats,
                    search_depth,
                );
                if let Err(e) = res {
                    println!("{e}");
                }
            }
        }
    }

    Ok(())
}

struct ConnectionTree {
    nodes: Vec<ConnectionTreeNode>,
}

impl std::ops::Index<NodeId> for ConnectionTree {
    type Output = ConnectionTreeNode;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0 as usize]
    }
}

impl std::ops::IndexMut<NodeId> for ConnectionTree {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.0 as usize]
    }
}

impl ConnectionTree {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// reserves size slots, and returns the starting index
    pub fn alloc(&mut self, size: u16) -> NodeId {
        let len = self.nodes.len();
        let new_len = len + size as usize;
        self.nodes.resize_with(new_len, ConnectionTreeNode::uninit);
        NodeId(len as u32)
    }

    pub fn add_child(&mut self, node_id: NodeId, len: &mut u16, node: ConnectionTreeNode) {
        let idx = node_id + *len;
        self[idx] = node;
        *len += 1;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct NodeId(u32);

impl std::ops::Add<u16> for NodeId {
    type Output = NodeId;

    fn add(self, rhs: u16) -> Self::Output {
        NodeId(self.0 + rhs as u32)
    }
}

#[derive(Clone, PartialEq, Eq)]
struct ConnectionTreeNode {
    building: ConnectionBuilding,
    end_dist: u16,
    state: State,
}

impl ConnectionTreeNode {
    fn new(building: ConnectionBuilding, end_dist: u16, state: State) -> Self {
        Self {
            building,
            end_dist,
            state,
        }
    }

    fn uninit() -> Self {
        Self {
            building: ConnectionBuilding::Mine(Mine::new((i8::MIN, i8::MIN), Rotation::Up)),
            end_dist: 0,
            state: State::Stopped,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum ConnectionBuilding {
    Mine(Mine),
    Conveyor(Conveyor),
    Combiner(Combiner),
}

#[derive(Clone, PartialEq, Eq)]
enum State {
    /// Search depth was exceeded, this path might be continued further
    Stopped,
    /// Connected to the factory
    Factory,
    /// A list of children
    Children {
        /// Start index into the connection tree nodes
        start: NodeId,
        /// Length of the children array
        len: u16,
    },
}

fn connect_deposits_and_factory(
    sim: &mut Sim,
    product_stats: &ProductStats,
    factory_stats: &FactoryStats,
    search_depth: u8,
) -> sim::Result<()> {
    let product_type = product_stats.product_type;
    let factory = Building::Factory(Factory::new(factory_stats.pos, product_type));
    sim::place_building(sim, factory)?;

    let factory_distance_map = map_distances(sim, factory_stats.pos, FACTORY_SIZE, FACTORY_SIZE);
    let mut tree = ConnectionTree::new();

    for d in factory_stats.deposits_in_reach.iter() {
        let stats = &product_stats.deposit_stats[d.idx];
        let Building::Deposit(deposit) = &sim.buildings[stats.id] else { unreachable!("This should be a deposit") };
        let deposit_pos = deposit.pos;
        let deposit_width = deposit.width as i8;
        let deposit_height = deposit.height as i8;

        const MINE_CORNER_POSITIONS: u16 = 4;
        const MINE_CORNER_ROTATIONS: u16 = 3;
        const MINE_CORNER_CONFIGURATIONS: u16 = MINE_CORNER_POSITIONS * MINE_CORNER_ROTATIONS;
        const MINE_EDGE_ROTATIONS: u16 = 2;
        let mine_edge_positions =
            2 * deposit.width.saturating_sub(1) + 2 * deposit.height.saturating_sub(1);
        let max_children_len =
            MINE_CORNER_CONFIGURATIONS + mine_edge_positions as u16 * MINE_EDGE_ROTATIONS;
        let children_id = tree.alloc(max_children_len);
        let mut children_len = 0;

        // place a mine somewhere around the deposit
        for x in 0..deposit_width {
            let pos = deposit_pos + (x, -1);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                place_mine(
                    sim,
                    &mut tree,
                    &factory_distance_map,
                    pos,
                    children_id,
                    &mut children_len,
                    search_depth,
                );
            }
        }
        for x in 0..deposit_width {
            let pos = deposit_pos + (x, deposit_height);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                place_mine(
                    sim,
                    &mut tree,
                    &factory_distance_map,
                    pos,
                    children_id,
                    &mut children_len,
                    search_depth,
                );
            }
        }
        for y in 0..deposit_height {
            let pos = deposit_pos + (-1, y);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                place_mine(
                    sim,
                    &mut tree,
                    &factory_distance_map,
                    pos,
                    children_id,
                    &mut children_len,
                    search_depth,
                );
            }
        }
        for y in 0..deposit_height {
            let pos = deposit_pos + (deposit_width, y);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                place_mine(
                    sim,
                    &mut tree,
                    &factory_distance_map,
                    pos,
                    children_id,
                    &mut children_len,
                    search_depth,
                );
            }
        }
    }

    Ok(())
}

fn place_mine(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    node_id: NodeId,
    len: &mut u16,
    search_depth: u8,
) {
    const DOCKING_POSITIONS: u16 = 3;
    const CONVEYOR_CONFIGURATIONS: u16 = 6;
    const COMBINER_CONFIGURATIONS: u16 = 0;
    const MAX_CHILDREN_SIZE: u16 =
        DOCKING_POSITIONS * (CONVEYOR_CONFIGURATIONS + COMBINER_CONFIGURATIONS);

    'block: {
        let end_pos = start_pos + (3, 0);
        let Some(end_dist) = distance_map.get(end_pos).flatten() else { break 'block };
        let mine = Mine::new(start_pos + (1, -1), Rotation::Up);
        let Some(mine_id) = sim::place_building(sim, Building::Mine(mine.clone())).ok() else { break 'block };

        let children_id = tree.alloc(MAX_CHILDREN_SIZE);
        let state =
            place_connector_around(sim, distance_map, end_pos, children_id, search_depth - 1);

        sim::remove_building(sim, mine_id);

        let building = ConnectionBuilding::Mine(mine);
        let node = ConnectionTreeNode::new(building, end_dist, state);
        tree.add_child(node_id, len, node);
    }
    'block: {
        let end_pos = start_pos + (0, 3);
        let Some(_dist) = distance_map.get(end_pos).flatten() else { break 'block };
        let mine = Mine::new(start_pos + (0, 1), Rotation::Right);
        todo!()
    }
    'block: {
        let end_pos = start_pos + (-3, 0);
        let Some(_dist) = distance_map.get(end_pos).flatten() else { break 'block };
        let mine = Mine::new(start_pos + (-2, 0), Rotation::Down);
        todo!()
    }
    'block: {
        let end_pos = start_pos + (0, -3);
        let Some(_dist) = distance_map.get(end_pos).flatten() else { break 'block };
        let mine = Mine::new(start_pos + (-1, -2), Rotation::Left);
        todo!()
    }
}

fn place_connector_around(
    sim: &mut Sim,
    distance_map: &DistanceMap,
    start_pos: Pos,
    node_id: NodeId,
    search_depth: u8,
) -> State {
    if search_depth == 0 {
        return State::Stopped;
    }

    let mut len = 0;

    // TODO: place connectors around, the last connectors end

    State::Children {
        start: node_id,
        len,
    }
}

// place conveyors or combiners
fn place_connector(
    sim: &mut Sim,
    factory_distance_map: &DistanceMap,
    start_pos: Pos,
    search_depth: u8,
) -> Option<()> {
    todo!("")
}
