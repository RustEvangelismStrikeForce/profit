use profit_sim as sim;
use sim::{Building, Combiner, Conveyor, Factory, Mine, Pos, Rotation, Sim, FACTORY_SIZE};

use crate::{map_distances, DistanceMap, FactoryStats, ProductStats};

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
        self.nodes
            .resize_with(len + size as usize, ConnectionTreeNode::uninit);
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
    Connected,
    /// A list of children
    Children {
        /// Start index into the connection tree nodes
        start: NodeId,
        /// Length of the children array
        len: u16,
    },
}

pub(crate) fn connect_deposits_and_factory(
    sim: &mut Sim,
    product_stats: &ProductStats,
    factory_stats: &FactoryStats,
    search_depth: u8,
) -> sim::Result<()> {
    let product_type = product_stats.product_type;
    let factory = Building::Factory(Factory::new(factory_stats.pos, product_type));
    let building_id = sim::place_building(sim, factory)?;

    let factory_distance_map = map_distances(sim, factory_stats.pos, FACTORY_SIZE, FACTORY_SIZE);
    let mut tree = ConnectionTree::new();

    for d in factory_stats.deposits_in_reach.iter() {
        let stats = &product_stats.deposit_stats[d.idx];
        let Building::Deposit(deposit) = &sim.buildings[stats.id] else { unreachable!("This should be a deposit") };
        let deposit_pos = deposit.pos;
        let deposit_width = deposit.width as i8;
        let deposit_height = deposit.height as i8;

        const MINE_CORNER_POSITIONS: u16 = 4;
        const MINE_CORNER_CONFIGURATIONS: u16 = 3;
        const MINE_EDGE_CONFIGURATIONS: u16 = 2;
        let mine_edge_positions =
            2 * deposit.width.saturating_sub(1) + 2 * deposit.height.saturating_sub(1);
        let max_children_len = MINE_CORNER_POSITIONS * MINE_CORNER_CONFIGURATIONS
            + mine_edge_positions as u16 * MINE_EDGE_CONFIGURATIONS;
        let children_id = tree.alloc(max_children_len);
        let mut children_len = 0;

        // place a mine somewhere around the deposit
        for x in 0..deposit_width {
            let pos = deposit_pos + (x, -1);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                #[rustfmt::skip]
                place_mines(sim, &mut tree, &factory_distance_map, pos, children_id, &mut children_len, search_depth);
            }
        }
        for x in 0..deposit_width {
            let pos = deposit_pos + (x, deposit_height);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                #[rustfmt::skip]
                place_mines(sim, &mut tree, &factory_distance_map, pos, children_id, &mut children_len, search_depth);
            }
        }
        for y in 0..deposit_height {
            let pos = deposit_pos + (-1, y);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                #[rustfmt::skip]
                place_mines(sim, &mut tree, &factory_distance_map, pos, children_id, &mut children_len, search_depth);
            }
        }
        for y in 0..deposit_height {
            let pos = deposit_pos + (deposit_width, y);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                #[rustfmt::skip]
                place_mines(sim, &mut tree, &factory_distance_map, pos, children_id, &mut children_len, search_depth);
            }
        }
    }

    sim::remove_building(sim, building_id);

    Ok(())
}

#[rustfmt::skip]
#[inline(always)]
fn place_mines(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    node_id: NodeId,
    len: &mut u16,
    search_depth: u8,
) {
    println!("------------------------------");
    place_mine(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Up,    (1,  -1), (3,  0));
    place_mine(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Right, (0,   1), (0,  3));
    place_mine(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Down,  (-2,  0), (-3, 0));
    place_mine(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Left,  (-1, -2), (0, -3));
}

fn place_mine(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    node_id: NodeId,
    len: &mut u16,
    search_depth: u8,
    rotation: Rotation,
    pos_offset: impl Into<Pos>,
    end_offset: impl Into<Pos>,
) {
    let end_pos = start_pos + end_offset;
    let Some(end_dist) = distance_map.get(end_pos).flatten() else { return };
    let mine = Mine::new(start_pos + pos_offset, rotation);
    let Some(building_id) = sim::place_building(sim, Building::Mine(mine.clone())).ok() else { return };

    println!("mine {start_pos} {rotation:?}");
    println!("{:?}", sim.board);

    let state =
        place_connector_around(sim, tree, distance_map, end_pos, end_dist, search_depth - 1);

    sim::remove_building(sim, building_id);

    let building = ConnectionBuilding::Mine(mine);
    let node = ConnectionTreeNode::new(building, end_dist, state);
    tree.add_child(node_id, len, node);
}

fn place_connector_around(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    start_dist: u16,
    search_depth: u8,
) -> State {
    println!("------------------------------");
    if start_dist == 0 {
        return State::Connected;
    }
    if search_depth == 0 {
        return State::Stopped;
    }

    const DOCKING_POSITIONS: u16 = 3;
    const SMALL_CONVEYOR_CONFIGURATIONS: u16 = 3;
    const BIG_CONVEYOR_CONFIGURATIONS: u16 = 3;
    const COMBINER_CONFIGURATIONS: u16 = 5;
    const MAX_CHILDREN_LEN: u16 = DOCKING_POSITIONS
        * (SMALL_CONVEYOR_CONFIGURATIONS + BIG_CONVEYOR_CONFIGURATIONS + COMBINER_CONFIGURATIONS);

    let children_id = tree.alloc(MAX_CHILDREN_LEN);
    let mut len = 0;

    #[rustfmt::skip]
    place_connectors(sim, tree, distance_map, start_pos + (-1, 0), children_id, &mut len, search_depth);
    #[rustfmt::skip]
    place_connectors(sim, tree, distance_map, start_pos + (1, 0), children_id, &mut len, search_depth);
    #[rustfmt::skip]
    place_connectors(sim, tree, distance_map, start_pos + (0, 1), children_id, &mut len, search_depth);
    #[rustfmt::skip]
    place_connectors(sim, tree, distance_map, start_pos + (0, -1), children_id, &mut len, search_depth);

    State::Children {
        start: children_id,
        len,
    }
}

// place conveyors or combiners
#[rustfmt::skip]
#[inline(always)]
fn place_connectors(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    node_id: NodeId,
    len: &mut u16,
    search_depth: u8,
) {
    // small conveyors
    place_conveyor(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Up,    (1,  0), (2,  0),  false);
    place_conveyor(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Right, (0,  1), (0,  2),  false);
    place_conveyor(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Down,  (-1, 0), (-2, 0), false);
    place_conveyor(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Left,  (0, -1), (0, -2), false);
    
    // big conveyors
    place_conveyor(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Up,    (1,  0), (3,  0), true);
    place_conveyor(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Right, (0,  1), (0,  3), true);
    place_conveyor(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Down,  (-2, 0), (-3, 0), true);
    place_conveyor(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Left,  (0, -2), (0, -3), true);
    
    // combiners
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Up, (1,  1), (2,  1));
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Up, (1,  0), (2,  0));
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Up, (1, -1), (2, -1));
    
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Right, (1,  1), (1,  2));
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Right, (0,  1), (0,  2));
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Right, (-1, 1), (-1, 2));
    
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Down, (-1,  1), (-2,  1));
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Down, (-1,  0), (-2,  0));
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Down, (-1, -1), (-2, -1));
    
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Left, (1,  -1), (1,  -2));
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Left, (0,  -1), (0,  -2));
    place_combiner(sim, tree, distance_map, start_pos, node_id, len, search_depth, Rotation::Left, (-1, -1), (-1, -2));
}

fn place_conveyor(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    node_id: NodeId,
    len: &mut u16,
    search_depth: u8,
    rotation: Rotation,
    pos_offset: impl Into<Pos>,
    end_offset: impl Into<Pos>,
    big: bool,
) {
    let end_pos = start_pos + end_offset;
    let Some(end_dist) = distance_map.get(end_pos).flatten() else { return };
    let conveyor = Conveyor::new(start_pos + pos_offset, rotation, big);
    let Some(building_id) = sim::place_building(sim, Building::Conveyor(conveyor.clone())).ok() else { return };

    println!("conveyor {start_pos} {rotation:?} {big}");

    let state =
        place_connector_around(sim, tree, distance_map, end_pos, end_dist, search_depth - 1);

    sim::remove_building(sim, building_id);

    let building = ConnectionBuilding::Conveyor(conveyor);
    let node = ConnectionTreeNode::new(building, end_dist, state);
    tree.add_child(node_id, len, node);
}

fn place_combiner(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    node_id: NodeId,
    len: &mut u16,
    search_depth: u8,
    rotation: Rotation,
    pos_offset: impl Into<Pos>,
    end_offset: impl Into<Pos>,
) {
    let end_pos = start_pos + end_offset;
    let Some(end_dist) = distance_map.get(end_pos).flatten() else { return };
    let combiner = Combiner::new(start_pos + pos_offset, rotation);
    let Some(building_id) = sim::place_building(sim, Building::Combiner(combiner.clone())).ok() else { return };

    println!("combiner {start_pos} {rotation:?}");
    println!("{:?}", sim.board);

    let state =
        place_connector_around(sim, tree, distance_map, end_pos, end_dist, search_depth - 1);

    sim::remove_building(sim, building_id);

    let building = ConnectionBuilding::Combiner(combiner);
    let node = ConnectionTreeNode::new(building, end_dist, state);
    tree.add_child(node_id, len, node);
}
