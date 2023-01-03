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
    pub fn alloc(&mut self, size: u16) -> ChildrenId {
        let len = self.nodes.len();
        self.nodes
            .resize_with(len + size as usize, ConnectionTreeNode::uninitialized);
        ChildrenId(len as u32)
    }
}

fn increment_id(children_id: ChildrenId, len: &mut u16) -> NodeId {
    let id = children_id.0 + *len as u32;
    *len += 1;
    NodeId(id)
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ChildrenId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NodeId(u32);

#[derive(Clone, PartialEq, Eq)]
struct ConnectionTreeNode {
    building: ConnectionBuilding,
    end_pos: Pos,
    state: State,
}

impl ConnectionTreeNode {
    fn new(building: ConnectionBuilding, end_pos: Pos, state: State) -> Self {
        Self {
            building,
            end_pos,
            state,
        }
    }

    fn uninitialized() -> Self {
        Self {
            building: ConnectionBuilding::Mine(Mine::new((i8::MIN, i8::MIN), Rotation::Up)),
            end_pos: Pos::new(i8::MIN, i8::MIN),
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

impl ConnectionBuilding {
    fn to_building(&self) -> Building {
        match self {
            ConnectionBuilding::Mine(m) => Building::Mine(m.clone()),
            ConnectionBuilding::Conveyor(c) => Building::Conveyor(c.clone()),
            ConnectionBuilding::Combiner(c) => Building::Combiner(c.clone()),
        }
    }
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
        start: ChildrenId,
        /// Length of the children array
        len: u16,
    },
}

#[derive(Clone, PartialEq, Eq, PartialOrd)]
struct PathStats {
    dist: u16,
    depth: u8,
}

impl Ord for PathStats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&other.dist, &self.dist).then(Ord::cmp(&other.depth, &self.depth))
    }
}

impl PathStats {
    fn new(dist: u16, depth: u8) -> Self {
        Self { dist, depth }
    }
}

pub(crate) fn connect_deposits_and_factory(
    sim: &mut Sim,
    product_stats: &ProductStats,
    factory_stats: &FactoryStats,
    search_depth: u8,
) -> crate::Result<()> {
    let product_type = product_stats.product_type;
    let factory = Building::Factory(Factory::new(factory_stats.pos, product_type));
    let building_id = sim::place_building(sim, factory)?;

    let factory_distance_map = map_distances(sim, factory_stats.pos, FACTORY_SIZE, FACTORY_SIZE);
    let mut tree = ConnectionTree::new();

    for d in factory_stats.deposits_in_reach.iter() {
        let deposit_stats = &product_stats.deposit_stats[d.idx];
        println!("Deposit {:?}", deposit_stats.id);
        let Building::Deposit(deposit) = &sim.buildings[deposit_stats.id] else { unreachable!("This should be a deposit") };
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

        let mut best = None;
        // place a mine somewhere around the deposit
        for x in 0..deposit_width {
            let pos = deposit_pos + (x, -1);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                #[rustfmt::skip]
                let stats = place_mines(sim, &mut tree, &factory_distance_map, pos, children_id, &mut children_len, search_depth);
                cmp_and_set(&mut best, stats);
            }
        }
        for y in 0..deposit_height {
            let pos = deposit_pos + (-1, y);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                #[rustfmt::skip]
                let stats = place_mines(sim, &mut tree, &factory_distance_map, pos, children_id, &mut children_len, search_depth);
                cmp_and_set(&mut best, stats);
            }
            let pos = deposit_pos + (deposit_width, y);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                #[rustfmt::skip]
                let stats = place_mines(sim, &mut tree, &factory_distance_map, pos, children_id, &mut children_len, search_depth);
                cmp_and_set(&mut best, stats);
            }
        }
        for x in 0..deposit_width {
            let pos = deposit_pos + (x, deposit_height);
            if let Some(Some(_dist)) = factory_distance_map.get(pos) {
                #[rustfmt::skip]
                let stats = place_mines(sim, &mut tree, &factory_distance_map, pos, children_id, &mut children_len, search_depth);
                cmp_and_set(&mut best, stats);
            }
        }

        let mut path = Vec::new();
        let res = loop {
            let Some((node_id, _)) = best else {
                break Err(crate::Error::NoPath(
                    deposit_stats.id,
                    deposit_pos,
                    factory_stats.pos,
                ));
            };
            path.push(node_id);
            println!("{path:?}");

            let node = &tree[node_id];
            let connector_id = sim::place_building_unchecked(sim, node.building.to_building());

            match node.state {
                State::Connected => {
                    println!("{:?}", sim.board);
                    break Ok(sim.clone());
                }
                State::Stopped => {
                    println!("continue new");
                    let end_pos = node.end_pos;
                    let end_dist = factory_distance_map[node.end_pos].expect("should be valid");
                    #[rustfmt::skip]
                    let (state, stats) = place_connectors_around(sim, &mut tree, &factory_distance_map, node_id, end_pos, end_dist, search_depth);

                    // TODO: decide whether we need to clean up these buildings afterwards
                    // sim::remove_building(sim, connector_id);

                    tree[node_id].state = state;
                    best = stats;
                }
                State::Children { start, len } => {
                    println!("continue existing");
                    #[rustfmt::skip]
                    let stats = continue_subtree(sim, &mut tree, &factory_distance_map, start, len, search_depth);

                    // TODO: decide whether we need to clean up these buildings afterwards
                    // sim::remove_building(sim, connector_id);

                    best = stats;
                }
            }
        };

        match res {
            Ok(_) => (),
            Err(e) => println!("{e}"),
        }
    }

    sim::remove_building(sim, building_id);

    Ok(())
}

fn continue_subtree(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    children_id: ChildrenId,
    len: u16,
    search_depth: u8,
) -> Option<(NodeId, PathStats)> {
    let mut best = None;

    for i in 0..len {
        let node_id = NodeId(children_id.0 + i as u32);
        let node = &tree[node_id];
        match node.state {
            State::Connected => {
                println!("{:?}", sim.board);
                // TODO: consider somehow storing a list of equally good paths.
                return Some((node_id, PathStats::new(0, search_depth)));
            }
            State::Stopped => {
                // TODO: decide whether we need to clean up these buildings afterwards
                let building_id = sim::place_building_unchecked(sim, node.building.to_building());

                let end_pos = node.end_pos;
                let end_dist = distance_map[node.end_pos].expect("should be valid");
                #[rustfmt::skip]
                let (state, stats) = place_connectors_around(sim, tree, distance_map, node_id, end_pos, end_dist, search_depth - 1);

                sim::remove_building(sim, building_id);

                tree[node_id].state = state;
                cmp_and_set(&mut best, stats);
            }
            State::Children { start, len } => {
                // TODO: decide whether we need to clean up these buildings afterwards
                let building_id = sim::place_building_unchecked(sim, node.building.to_building());

                #[rustfmt::skip]
                let stats = continue_subtree(sim, tree, distance_map, start, len, search_depth - 1);
                cmp_and_set(&mut best, stats);

                sim::remove_building(sim, building_id);
            }
        }
    }

    best
}

#[rustfmt::skip]
#[inline(always)]
fn place_mines(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    children_id: ChildrenId,
    len: &mut u16,
    search_depth: u8,
) -> Option<(NodeId, PathStats)> {
    // println!("------------------------------");
    let mut best = None;

    let stats = place_mine(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Up,    (1,  -1), (3,  0));
    cmp_and_set(&mut best, stats);
    let stats = place_mine(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Right, (0,   1), (0,  3));
    cmp_and_set(&mut best, stats);
    let stats = place_mine(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Down,  (-2,  0), (-3, 0));
    cmp_and_set(&mut best, stats);
    let stats = place_mine(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Left,  (-1, -2), (0, -3));
    cmp_and_set(&mut best, stats);

    best
}

fn place_mine(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    children_id: ChildrenId,
    len: &mut u16,
    search_depth: u8,
    rotation: Rotation,
    pos_offset: impl Into<Pos>,
    end_offset: impl Into<Pos>,
) -> Option<(NodeId, PathStats)> {
    let end_pos = start_pos + end_offset;
    let end_dist = distance_map.get(end_pos)??;
    let mine = Mine::new(start_pos + pos_offset, rotation);
    let building_id = sim::place_building(sim, Building::Mine(mine.clone())).ok()?;

    // let indent = (10 - 2 * search_depth) as usize;
    // println!("{:indent$}mine {start_pos} {rotation:?}", "");

    let node_id = increment_id(children_id, len);

    #[rustfmt::skip]
    let (state, stats) = place_connectors_around(sim, tree, distance_map, node_id, end_pos, end_dist, search_depth - 1);

    sim::remove_building(sim, building_id);

    let building = ConnectionBuilding::Mine(mine);
    let node = ConnectionTreeNode::new(building, end_pos, state);
    tree[node_id] = node;

    stats.map(|(_, s)| (node_id, s))
}

fn place_connectors_around(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    parent_id: NodeId,
    start_pos: Pos,
    start_dist: u16,
    search_depth: u8,
) -> (State, Option<(NodeId, PathStats)>) {
    if start_dist == 0 {
        return (
            State::Connected,
            Some((parent_id, PathStats::new(0, search_depth))),
        );
    }
    if search_depth == 0 {
        return (
            State::Stopped,
            Some((parent_id, PathStats::new(start_dist, search_depth))),
        );
    }

    // let indent = (10 - 2 * search_depth) as usize;
    // println!("{:indent$}------------------------------", "");

    const DOCKING_POSITIONS: u16 = 3;
    const SMALL_CONVEYOR_CONFIGURATIONS: u16 = 3;
    const BIG_CONVEYOR_CONFIGURATIONS: u16 = 3;
    const COMBINER_CONFIGURATIONS: u16 = 5;
    const MAX_CHILDREN_LEN: u16 = DOCKING_POSITIONS
        * (SMALL_CONVEYOR_CONFIGURATIONS + BIG_CONVEYOR_CONFIGURATIONS + COMBINER_CONFIGURATIONS);

    let children_id = tree.alloc(MAX_CHILDREN_LEN);
    let mut len = 0;

    let mut best = None;

    #[rustfmt::skip]
    let stats = place_connectors(sim, tree, distance_map, start_pos + (-1, 0), children_id, &mut len, search_depth);
    cmp_and_set(&mut best, stats);

    #[rustfmt::skip]
    let stats = place_connectors(sim, tree, distance_map, start_pos + (1, 0), children_id, &mut len, search_depth);
    cmp_and_set(&mut best, stats);

    #[rustfmt::skip]
    let stats = place_connectors(sim, tree, distance_map, start_pos + (0, 1), children_id, &mut len, search_depth);
    cmp_and_set(&mut best, stats);

    #[rustfmt::skip]
    let stats = place_connectors(sim, tree, distance_map, start_pos + (0, -1), children_id, &mut len, search_depth);
    cmp_and_set(&mut best, stats);

    let state = State::Children {
        start: children_id,
        len,
    };

    (state, best)
}

/// Place conveyors or combiners
#[rustfmt::skip]
#[inline(always)]
fn place_connectors(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    children_id: ChildrenId,
    len: &mut u16,
    search_depth: u8,
) -> Option<(NodeId, PathStats)> {
    let mut best = None;

    // small conveyors
    let stats = place_conveyor(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Up,    (1,  0), (2,  0),  false);
    cmp_and_set(&mut best, stats);
    let stats = place_conveyor(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Right, (0,  1), (0,  2),  false);
    cmp_and_set(&mut best, stats);
    let stats = place_conveyor(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Down,  (-1, 0), (-2, 0), false);
    cmp_and_set(&mut best, stats);
    let stats = place_conveyor(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Left,  (0, -1), (0, -2), false);
    cmp_and_set(&mut best, stats);

    // big conveyors
    let stats = place_conveyor(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Up,    (1,  0), (3,  0), true);
    cmp_and_set(&mut best, stats);
    let stats = place_conveyor(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Right, (0,  1), (0,  3), true);
    cmp_and_set(&mut best, stats);
    let stats = place_conveyor(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Down,  (-2, 0), (-3, 0), true);
    cmp_and_set(&mut best, stats);
    let stats = place_conveyor(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Left,  (0, -2), (0, -3), true);
    cmp_and_set(&mut best, stats);

    // combiners
    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Up, (1,  1), (2,  1));
    cmp_and_set(&mut best, stats);
    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Up, (1,  0), (2,  0));
    cmp_and_set(&mut best, stats);
    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Up, (1, -1), (2, -1));
    cmp_and_set(&mut best, stats);

    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Right, (1,  1), (1,  2));
    cmp_and_set(&mut best, stats);
    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Right, (0,  1), (0,  2));
    cmp_and_set(&mut best, stats);
    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Right, (-1, 1), (-1, 2));
    cmp_and_set(&mut best, stats);

    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Down, (-1,  1), (-2,  1));
    cmp_and_set(&mut best, stats);
    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Down, (-1,  0), (-2,  0));
    cmp_and_set(&mut best, stats);
    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Down, (-1, -1), (-2, -1));
    cmp_and_set(&mut best, stats);

    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Left, (1,  -1), (1,  -2));
    cmp_and_set(&mut best, stats);
    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Left, (0,  -1), (0,  -2));
    cmp_and_set(&mut best, stats);
    let stats = place_combiner(sim, tree, distance_map, start_pos, children_id, len, search_depth, Rotation::Left, (-1, -1), (-1, -2));
    cmp_and_set(&mut best, stats);

    best
}

fn place_conveyor(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    children_id: ChildrenId,
    len: &mut u16,
    search_depth: u8,
    rotation: Rotation,
    pos_offset: impl Into<Pos>,
    end_offset: impl Into<Pos>,
    big: bool,
) -> Option<(NodeId, PathStats)> {
    let end_pos = start_pos + end_offset;
    let end_dist = distance_map.get(end_pos).flatten()?;
    let conveyor = Conveyor::new(start_pos + pos_offset, rotation, big);
    let building_id = sim::place_building(sim, Building::Conveyor(conveyor.clone())).ok()?;

    // let indent = (10 - 2 * search_depth) as usize;
    // println!("{:indent$}conveyor {start_pos} {rotation:?} {big}", "");

    let node_id = increment_id(children_id, len);

    #[rustfmt::skip]
    let (state, stats) = place_connectors_around(sim, tree, distance_map, node_id, end_pos, end_dist, search_depth - 1);

    sim::remove_building(sim, building_id);

    let building = ConnectionBuilding::Conveyor(conveyor);
    let node = ConnectionTreeNode::new(building, end_pos, state);
    tree[node_id] = node;

    stats.map(|(_, s)| (node_id, s))
}

fn place_combiner(
    sim: &mut Sim,
    tree: &mut ConnectionTree,
    distance_map: &DistanceMap,
    start_pos: Pos,
    children_id: ChildrenId,
    len: &mut u16,
    search_depth: u8,
    rotation: Rotation,
    pos_offset: impl Into<Pos>,
    end_offset: impl Into<Pos>,
) -> Option<(NodeId, PathStats)> {
    let end_pos = start_pos + end_offset;
    let end_dist = distance_map.get(end_pos).flatten()?;
    let combiner = Combiner::new(start_pos + pos_offset, rotation);
    let building_id = sim::place_building(sim, Building::Combiner(combiner.clone())).ok()?;

    // let indent = (10 - 2 * search_depth) as usize;
    // println!("{:indent$}combiner {start_pos} {rotation:?}", "");

    let node_id = increment_id(children_id, len);

    #[rustfmt::skip]
    let (state, stats) = place_connectors_around(sim, tree, distance_map, node_id, end_pos, end_dist, search_depth - 1);

    sim::remove_building(sim, building_id);

    let building = ConnectionBuilding::Combiner(combiner);
    let node = ConnectionTreeNode::new(building, end_pos, state);
    tree[node_id] = node;

    stats.map(|(_, s)| (node_id, s))
}

#[inline(always)]
fn cmp_and_set(best: &mut Option<(NodeId, PathStats)>, other: Option<(NodeId, PathStats)>) {
    if let Some((_, other_stats)) = &other {
        match best {
            None => *best = other,
            Some((_, best_stats)) => {
                if other_stats > best_stats {
                    *best = other;
                }
            }
        }
    }
}
