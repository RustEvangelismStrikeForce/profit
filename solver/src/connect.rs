use std::cmp::Ordering;

use sim::{
    Building, CellKind, Combiner, Conveyor, Factory, Id, Mine, Pos, Rotation, Sim, SimRun,
    FACTORY_SIZE,
};
use smallvec::SmallVec;

use crate::{map_distances, DistanceMap, FactoryStats, ProductStats};

#[cfg(test)]
mod test;

struct Context<'a> {
    sim: &'a mut Sim,
    tree: ConnectionTree,
    distance_map: DistanceMap,
    factory_id: Id,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ChildrenId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NodeId(u32);

#[derive(Clone, Debug, PartialEq, Eq)]
struct ConnectionTreeNode {
    building: ConnectionBuilding,
    start_pos: Pos,
    end_pos: Pos,
    state: State,
}

impl ConnectionTreeNode {
    fn new(building: ConnectionBuilding, start_pos: Pos, end_pos: Pos, state: State) -> Self {
        Self {
            building,
            start_pos,
            end_pos,
            state,
        }
    }

    fn uninitialized() -> Self {
        Self {
            building: ConnectionBuilding::Mine(Mine::new((i8::MIN, i8::MIN), Rotation::Right)),
            start_pos: Pos::new(i8::MIN, i8::MIN),
            end_pos: Pos::new(i8::MIN, i8::MIN),
            state: State::Stopped,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ConnectionBuilding {
    Mine(Mine),
    Conveyor(Conveyor),
    Combiner(Combiner),
}

impl ConnectionBuilding {
    fn to_building(&self) -> Building {
        match self {
            ConnectionBuilding::Mine(m) => Building::Mine(*m),
            ConnectionBuilding::Conveyor(c) => Building::Conveyor(*c),
            ConnectionBuilding::Combiner(c) => Building::Combiner(*c),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum State {
    /// Search depth was exceeded, this path might be continued further
    Stopped,
    /// Connected to the factory
    Connected,
    /// Connected to the factory via an already existent path
    Merged,
    /// A list of children
    Children {
        /// Start index into the connection tree nodes
        start: ChildrenId,
        /// Length of the children array
        len: u16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PathStats {
    dist: u16,
    /// flipped depth -> lower is deeper
    depth: u8,
}

impl Ord for PathStats {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&other.dist, &self.dist).then(Ord::cmp(&self.depth, &other.depth))
    }
}

impl PartialOrd for PathStats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
) -> crate::Result<(Sim, SimRun)> {
    let product_type = product_stats.product_type;
    let factory = Building::Factory(Factory::new(factory_stats.pos, product_type));
    let factory_id = sim::place_building(sim, factory)?;
    let distance_map = map_distances(sim, factory_stats.pos, FACTORY_SIZE, FACTORY_SIZE);
    let mut ctx = Context {
        sim,
        distance_map,
        tree: ConnectionTree::new(),
        factory_id,
    };

    let mut non_improvements = 0;
    let mut errors = 0;
    let mut runs = Vec::new();
    // TODO: smarter selection order of deposits to connect
    for (i, d) in factory_stats.deposits_in_reach.iter().cycle().enumerate() {
        ctx.tree = ConnectionTree::new();
        if i % factory_stats.deposits_in_reach.len() == 0 {
            errors = 0;
        }

        let deposit_stats = &product_stats.deposit_stats[d.idx];
        let Building::Deposit(deposit) = &ctx.sim.buildings[deposit_stats.id] else { unreachable!("This should be a deposit") };
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
        let children_id = ctx.tree.alloc(max_children_len);
        let mut children_len = 0;

        let mut best = None;
        // place a mine somewhere around the deposit
        for x in 0..deposit_width {
            let pos = deposit_pos + (x, -1);
            if let Some(Some(_dist)) = ctx.distance_map.get(pos) {
                #[rustfmt::skip]
                let stats = place_mines(&mut ctx, pos, children_id, &mut children_len, search_depth);
                cmp_and_set(&mut best, stats);
            }
        }
        for y in 0..deposit_height {
            let pos = deposit_pos + (-1, y);
            if let Some(Some(_dist)) = ctx.distance_map.get(pos) {
                #[rustfmt::skip]
                let stats = place_mines(&mut ctx, pos, children_id, &mut children_len, search_depth);
                cmp_and_set(&mut best, stats);
            }
            let pos = deposit_pos + (deposit_width, y);
            if let Some(Some(_dist)) = ctx.distance_map.get(pos) {
                #[rustfmt::skip]
                let stats = place_mines(&mut ctx, pos, children_id, &mut children_len, search_depth);
                cmp_and_set(&mut best, stats);
            }
        }
        for x in 0..deposit_width {
            let pos = deposit_pos + (x, deposit_height);
            if let Some(Some(_dist)) = ctx.distance_map.get(pos) {
                #[rustfmt::skip]
                let stats = place_mines(&mut ctx, pos, children_id, &mut children_len, search_depth);
                cmp_and_set(&mut best, stats);
            }
        }

        let mut path = Vec::new();
        let res = loop {
            let Some((node_id, _stats)) = best else {
                break Err(crate::Error::NoPath(
                    deposit_stats.id,
                    deposit_pos,
                    factory_stats.pos,
                ));
            };
            path.push(node_id);

            let node = &ctx.tree[node_id];

            // TODO: maybe later
            // sim::place_building_unchecked(ctx.sim, node.building.to_building());
            let connector_id = sim::place_building(ctx.sim, node.building.to_building())
                .expect("connector to be valid");

            match node.state {
                State::Connected => {
                    break Ok(ctx.sim.clone());
                }
                State::Merged => {
                    break Ok(ctx.sim.clone());
                }
                State::Stopped => {
                    let end_pos = node.end_pos;
                    let end_dist = ctx.distance_map[node.end_pos].expect("should be valid");

                    #[rustfmt::skip]
                    let (state, stats) = place_children_connectors(&mut ctx, connector_id, node_id, end_pos, end_dist, search_depth);

                    ctx.tree[node_id].state = state;
                    best = stats;
                }
                State::Children { start, len } => {
                    #[rustfmt::skip]
                    let stats = continue_subtree(&mut ctx, start, len, search_depth);

                    best = stats;
                }
            }
        };

        match res {
            Ok(sim) => {
                let new = sim::run(&sim);
                if let Some((_, last)) = runs.last() {
                    // maybe don't break immediately
                    if &new < last {
                        non_improvements += 1;
                    }
                    if non_improvements == 12 {
                        break;
                    }
                }
                runs.push((sim, new));
            }
            Err(_) => {
                errors += 1;
                if let Some((last_sim, _)) = runs.last() {
                    ctx.sim.clone_from(last_sim);
                }
                if errors == factory_stats.deposits_in_reach.len() {
                    break;
                }
            }
        }
    }

    runs.pop().ok_or(crate::Error::NoSolution)
}

fn continue_subtree(
    ctx: &mut Context,
    children_id: ChildrenId,
    len: u16,
    search_depth: u8,
) -> Option<(NodeId, PathStats)> {
    let mut best = None;

    for i in 0..len {
        let node_id = NodeId(children_id.0 + i as u32);
        let node = &mut ctx.tree[node_id];
        match node.state {
            State::Connected => {
                // TODO: consider somehow storing a list of equally good paths.
                return Some((node_id, PathStats::new(0, search_depth)));
            }
            State::Merged => {
                let building_id = sim::place_building(ctx.sim, node.building.to_building()).ok()?;

                let node_end_pos = node.end_pos;
                let (_, stats) =
                    find_connection_around(ctx, node_id, building_id, node_end_pos, search_depth)
                        .expect("this path to be merged");
                cmp_and_set(&mut best, stats);

                sim::remove_building(ctx.sim, building_id);
            }
            State::Stopped => {
                let end_pos = node.end_pos;
                let end_dist = ctx.distance_map[node.end_pos].expect("should be valid");

                // TODO: maybe later
                // let building_id = sim::place_building_unchecked(ctx.sim, node.building.to_building());
                let building_id = sim::place_building(ctx.sim, node.building.to_building()).ok()?;

                #[rustfmt::skip]
                let (state, stats) = place_children_connectors(ctx, building_id, node_id, end_pos, end_dist, search_depth - 1);
                cmp_and_set(&mut best, stats.map(|(_, s)| (node_id, s)));

                sim::remove_building(ctx.sim, building_id);

                ctx.tree[node_id].state = state;
            }
            State::Children { start, len } => {
                // TODO: maybe later
                // let building_id = sim::place_building_unchecked(ctx.sim, node.building.to_building());
                let building_id = sim::place_building(ctx.sim, node.building.to_building()).ok()?;

                #[rustfmt::skip]
                let stats = continue_subtree(ctx, start, len, search_depth - 1);
                cmp_and_set(&mut best, stats.map(|(_, s)| (node_id, s)));

                sim::remove_building(ctx.sim, building_id);
            }
        }
    }

    best
}

#[rustfmt::skip]
#[inline(always)]
fn place_mines(
    ctx: &mut Context,
    start_pos: Pos,
    children_id: ChildrenId,
    len: &mut u16,
    search_depth: u8,
) -> Option<(NodeId, PathStats)> {
    let mut best = None;

    let stats = place_mine(ctx, start_pos, children_id, len, search_depth, Rotation::Right,    (1,  -1), (3,  0));
    cmp_and_set(&mut best, stats);
    let stats = place_mine(ctx, start_pos, children_id, len, search_depth, Rotation::Down, (0,   1), (0,  3));
    cmp_and_set(&mut best, stats);
    let stats = place_mine(ctx, start_pos, children_id, len, search_depth, Rotation::Left,  (-2,  0), (-3, 0));
    cmp_and_set(&mut best, stats);
    let stats = place_mine(ctx, start_pos, children_id, len, search_depth, Rotation::Up,  (-1, -2), (0, -3));
    cmp_and_set(&mut best, stats);

    best
}

fn place_mine(
    ctx: &mut Context,
    start_pos: Pos,
    children_id: ChildrenId,
    len: &mut u16,
    search_depth: u8,
    rotation: Rotation,
    pos_offset: impl Into<Pos>,
    end_offset: impl Into<Pos>,
) -> Option<(NodeId, PathStats)> {
    let end_pos = start_pos + end_offset;
    let end_dist = ctx.distance_map.get(end_pos)??;
    let mine = Mine::new(start_pos + pos_offset, rotation);
    let building_id = sim::place_building(ctx.sim, Building::Mine(mine)).ok()?;

    let node_id = increment_id(children_id, len);

    #[rustfmt::skip]
    let (state, stats) = place_children_connectors(ctx, building_id, node_id, end_pos, end_dist, search_depth - 1);

    sim::remove_building(ctx.sim, building_id);

    let building = ConnectionBuilding::Mine(mine);
    let node = ConnectionTreeNode::new(building, start_pos, end_pos, state);
    ctx.tree[node_id] = node;

    stats.map(|(_, s)| (node_id, s))
}

fn place_children_connectors(
    ctx: &mut Context,
    connector_id: Id,
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

    // check if we're already connected to another path leading to the factory
    if let Some(s) = find_connection_around(ctx, parent_id, connector_id, start_pos, search_depth) {
        return s;
    }

    if search_depth == 0 {
        return (
            State::Stopped,
            Some((parent_id, PathStats::new(start_dist, search_depth))),
        );
    }

    const DOCKING_POSITIONS: u16 = 3;
    const SMALL_CONVEYOR_CONFIGURATIONS: u16 = 3;
    const BIG_CONVEYOR_CONFIGURATIONS: u16 = 3;
    const COMBINER_CONFIGURATIONS: u16 = 5;
    const MAX_CHILDREN_LEN: u16 = DOCKING_POSITIONS
        * (SMALL_CONVEYOR_CONFIGURATIONS + BIG_CONVEYOR_CONFIGURATIONS + COMBINER_CONFIGURATIONS);

    let children_id = ctx.tree.alloc(MAX_CHILDREN_LEN);
    let mut len = 0;

    let mut best = None;

    #[rustfmt::skip]
    place_connectors(ctx, start_pos + (-1, 0), children_id, &mut len, &mut best, search_depth);
    #[rustfmt::skip]
    place_connectors(ctx, start_pos + (1, 0), children_id, &mut len, &mut best, search_depth);
    #[rustfmt::skip]
    place_connectors(ctx, start_pos + (0, 1), children_id, &mut len, &mut best, search_depth);
    #[rustfmt::skip]
    place_connectors(ctx, start_pos + (0, -1), children_id, &mut len, &mut best, search_depth);

    let state = State::Children {
        start: children_id,
        len,
    };

    (state, best)
}

#[inline(always)]
fn find_connection_around(
    ctx: &Context,
    parent_id: NodeId,
    connector_id: Id,
    start_pos: Pos,
    search_depth: u8,
) -> Option<(State, Option<(NodeId, PathStats)>)> {
    if let Some(s) = find_connection_at(
        ctx,
        parent_id,
        connector_id,
        start_pos + (-1, 0),
        search_depth,
    ) {
        return Some(s);
    }
    if let Some(s) = find_connection_at(
        ctx,
        parent_id,
        connector_id,
        start_pos + (1, 0),
        search_depth,
    ) {
        return Some(s);
    }
    if let Some(s) = find_connection_at(
        ctx,
        parent_id,
        connector_id,
        start_pos + (0, -1),
        search_depth,
    ) {
        return Some(s);
    }
    if let Some(s) = find_connection_at(
        ctx,
        parent_id,
        connector_id,
        start_pos + (0, 1),
        search_depth,
    ) {
        return Some(s);
    }

    None
}

#[inline(always)]
fn find_connection_at(
    ctx: &Context,
    parent_id: NodeId,
    connector_id: Id,
    pos: Pos,
    search_depth: u8,
) -> Option<(State, Option<(NodeId, PathStats)>)> {
    if let Some(Some(cell)) = ctx.sim.board.get(pos) {
        if cell.id != connector_id && cell.kind == CellKind::Input {
            if let Some(s) = find_connection(ctx, parent_id, connector_id, cell.id, search_depth) {
                return Some(s);
            }
        }
    }

    None
}

#[inline(always)]
fn find_connection(
    ctx: &Context,
    parent_id: NodeId,
    connector_id: Id,
    mut current_id: Id,
    mut search_depth: u8,
) -> Option<(State, Option<(NodeId, PathStats)>)> {
    let mut path = SmallVec::<[_; 15]>::new();
    let mut last_search_node = connector_id;

    'path: loop {
        path.push(current_id);

        if search_depth > 0 {
            last_search_node = current_id;
            search_depth -= 1;
        }

        for conn in ctx.sim.connections.iter() {
            if conn.output_id == current_id {
                if path.contains(&conn.input_id) {
                    // we're in a loop
                    return None;
                }

                current_id = conn.input_id;

                if current_id == ctx.factory_id {
                    let last_building = &ctx.sim.buildings[last_search_node];
                    let dist = match last_building {
                        Building::Deposit(_) => unreachable!(),
                        Building::Obstacle(_) => unreachable!(),
                        Building::Mine(mine) => {
                            let pos = mine.pos
                                + match mine.rotation {
                                    Rotation::Right => (2, 1),
                                    Rotation::Down => (0, 2),
                                    Rotation::Left => (-1, 0),
                                    Rotation::Up => (1, -1),
                                };
                            ctx.distance_map[pos].expect("this field to be valid")
                        }
                        Building::Conveyor(conveyor) => {
                            let pos = conveyor.pos
                                + if conveyor.big {
                                    match conveyor.rotation {
                                        Rotation::Right => (2, 0),
                                        Rotation::Down => (0, 2),
                                        Rotation::Left => (-1, 0),
                                        Rotation::Up => (0, -1),
                                    }
                                } else {
                                    match conveyor.rotation {
                                        Rotation::Right => (1, 0),
                                        Rotation::Down => (0, 1),
                                        Rotation::Left => (-1, 0),
                                        Rotation::Up => (0, -1),
                                    }
                                };
                            ctx.distance_map[pos].expect("this field to be valid")
                        }
                        Building::Combiner(combiner) => {
                            let pos = combiner.pos
                                + match combiner.rotation {
                                    Rotation::Right => (1, 0),
                                    Rotation::Down => (0, 1),
                                    Rotation::Left => (-1, 0),
                                    Rotation::Up => (0, -1),
                                };
                            ctx.distance_map[pos].expect("this field to be valid")
                        }
                        Building::Factory(_) => 0,
                    };
                    return Some((
                        State::Merged,
                        Some((parent_id, PathStats::new(dist, search_depth))),
                    ));
                }

                continue 'path;
            }
        }

        // the path ends here
        return None;
    }
}

/// Place conveyors or combiners
#[rustfmt::skip]
#[inline(always)]
fn place_connectors(
    ctx: &mut Context,
    start_pos: Pos,
    children_id: ChildrenId,
    len: &mut u16,
    best: &mut Option<(NodeId, PathStats)>,
    search_depth: u8,
) {
    // small conveyors
    let stats = place_conveyor(ctx, start_pos, children_id, len, search_depth, Rotation::Right, (1,  0), (2,  0), false);
    cmp_and_set(best, stats);
    let stats = place_conveyor(ctx, start_pos, children_id, len, search_depth, Rotation::Down,  (0,  1), (0,  2), false);
    cmp_and_set(best, stats);
    let stats = place_conveyor(ctx, start_pos, children_id, len, search_depth, Rotation::Left,  (-1, 0), (-2, 0), false);
    cmp_and_set(best, stats);
    let stats = place_conveyor(ctx, start_pos, children_id, len, search_depth, Rotation::Up,    (0, -1), (0, -2), false);
    cmp_and_set(best, stats);

    // big conveyors
    let stats = place_conveyor(ctx, start_pos, children_id, len, search_depth, Rotation::Right, (1,  0), (3,  0), true);
    cmp_and_set(best, stats);
    let stats = place_conveyor(ctx, start_pos, children_id, len, search_depth, Rotation::Down,  (0,  1), (0,  3), true);
    cmp_and_set(best, stats);
    let stats = place_conveyor(ctx, start_pos, children_id, len, search_depth, Rotation::Left,  (-2, 0), (-3, 0), true);
    cmp_and_set(best, stats);
    let stats = place_conveyor(ctx, start_pos, children_id, len, search_depth, Rotation::Up,    (0, -2), (0, -3), true);
    cmp_and_set(best, stats);

    // combiners
    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Right, (1,  1), (2,  1));
    cmp_and_set(best, stats);
    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Right, (1,  0), (2,  0));
    cmp_and_set(best, stats);
    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Right, (1, -1), (2, -1));
    cmp_and_set(best, stats);

    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Down, (1,  1), (1,  2));
    cmp_and_set(best, stats);
    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Down, (0,  1), (0,  2));
    cmp_and_set(best, stats);
    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Down, (-1, 1), (-1, 2));
    cmp_and_set(best, stats);

    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Left, (-1,  1), (-2,  1));
    cmp_and_set(best, stats);
    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Left, (-1,  0), (-2,  0));
    cmp_and_set(best, stats);
    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Left, (-1, -1), (-2, -1));
    cmp_and_set(best, stats);

    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Up, (1,  -1), (1,  -2));
    cmp_and_set(best, stats);
    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Up, (0,  -1), (0,  -2));
    cmp_and_set(best, stats);
    let stats = place_combiner(ctx, start_pos, children_id, len, search_depth, Rotation::Up, (-1, -1), (-1, -2));
    cmp_and_set(best, stats);
}

fn place_conveyor(
    ctx: &mut Context,
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
    let end_dist = ctx.distance_map.get(end_pos).flatten()?;
    let conveyor = Conveyor::new(start_pos + pos_offset, rotation, big);
    let building_id = sim::place_building(ctx.sim, Building::Conveyor(conveyor)).ok()?;

    let node_id = increment_id(children_id, len);

    #[rustfmt::skip]
    let (state, stats) = place_children_connectors(ctx, building_id, node_id, end_pos, end_dist, search_depth - 1);

    sim::remove_building(ctx.sim, building_id);

    let building = ConnectionBuilding::Conveyor(conveyor);
    let node = ConnectionTreeNode::new(building, start_pos, end_pos, state);
    ctx.tree[node_id] = node;

    stats.map(|(_, s)| (node_id, s))
}

fn place_combiner(
    ctx: &mut Context,
    start_pos: Pos,
    children_id: ChildrenId,
    len: &mut u16,
    search_depth: u8,
    rotation: Rotation,
    pos_offset: impl Into<Pos>,
    end_offset: impl Into<Pos>,
) -> Option<(NodeId, PathStats)> {
    let end_pos = start_pos + end_offset;
    let end_dist = ctx.distance_map.get(end_pos).flatten()?;
    let combiner = Combiner::new(start_pos + pos_offset, rotation);
    let building_id = sim::place_building(ctx.sim, Building::Combiner(combiner)).ok()?;

    let node_id = increment_id(children_id, len);

    #[rustfmt::skip]
    let (state, stats) = place_children_connectors(ctx, building_id, node_id, end_pos, end_dist, search_depth - 1);

    sim::remove_building(ctx.sim, building_id);

    let building = ConnectionBuilding::Combiner(combiner);
    let node = ConnectionTreeNode::new(building, start_pos, end_pos, state);
    ctx.tree[node_id] = node;

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
