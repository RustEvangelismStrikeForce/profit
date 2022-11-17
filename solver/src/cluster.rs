use profit_sim as sim;
use sim::{pos, BuildingKind, Id, Pos, Sim, MAX_BOARD_SIZE};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cluster {
    pub buildings: Vec<Id>,
    pub cells: Vec<Pos>,
}

impl Cluster {
    pub fn new(buildings: Vec<Id>, cells: Vec<Pos>) -> Self {
        Self { buildings, cells }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Visited {
    width: i8,
    height: i8,
    cells: Vec<bool>,
}

impl<P: Into<Pos>> std::ops::Index<P> for Visited {
    type Output = bool;

    fn index(&self, pos: P) -> &Self::Output {
        let pos = pos.into();
        assert!(
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height,
            "Board index out of bounds"
        );

        &self.cells[pos.y as usize * self.width as usize + pos.x as usize]
    }
}

impl<P: Into<Pos>> std::ops::IndexMut<P> for Visited {
    fn index_mut(&mut self, pos: P) -> &mut Self::Output {
        let pos = pos.into();
        assert!(
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height,
            "Board index out of bounds"
        );

        &mut self.cells[pos.y as usize * self.width as usize + pos.x as usize]
    }
}

impl Visited {
    pub fn new(width: i8, height: i8) -> Self {
        let width = width.clamp(0, MAX_BOARD_SIZE);
        let height = height.clamp(0, MAX_BOARD_SIZE);
        Self {
            width,
            height,
            cells: vec![false; width as usize * height as usize],
        }
    }
}

pub fn find_clusters(sim: &Sim) -> Vec<Cluster> {
    let mut visited = Visited::new(sim.board.width, sim.board.height);
    let mut clusters = Vec::new();
    let mut position = pos(0, 0);

    loop {
        let mut current_cluster = Cluster::default();
        find_cluster(sim, &mut visited, &mut current_cluster, position);
        clusters.push(current_cluster);

        let mut all_visited = true;
        'outer: for y in 0..visited.height {
            for x in 0..visited.width {
                if !visited[(x, y)] {
                    position = pos(x, y);
                    all_visited = false;
                    break 'outer;
                }
            }
        }

        if all_visited {
            break;
        }
    }

    clusters
}

fn find_cluster(sim: &Sim, visited: &mut Visited, cluster: &mut Cluster, pos: Pos) {
    if pos.x < 0 || pos.x >= visited.width || pos.y < 0 || pos.y >= visited.height {
        return;
    }

    if let Some(c) = sim.board[pos] {
        let building = &sim.buildings[c.id];
        match &building.kind {
            BuildingKind::Deposit(deposit) => {
                if !cluster.buildings.contains(&c.id) {
                    cluster.buildings.push(c.id);
                }

                for y in 0..deposit.height as i8 {
                    for x in 0..deposit.width as i8 {
                        visited[building.pos + (x, y)] = true;
                    }
                }

                return;
            }
            BuildingKind::Obstacle(obstacle) => {
                for y in 0..obstacle.height as i8 {
                    for x in 0..obstacle.width as i8 {
                        visited[building.pos + (x, y)] = true;
                    }
                }

                return;
            }
            BuildingKind::Mine(_)
            | BuildingKind::Conveyor(_)
            | BuildingKind::Combiner(_)
            | BuildingKind::Factory(_) => todo!(),
        }
    } else {
        if !cluster.cells.contains(&pos) {
            cluster.cells.push(pos);
        }
    }

    if visited[pos] {
        return;
    }
    visited[pos] = true;

    find_cluster(sim, visited, cluster, pos + (0, -1));
    find_cluster(sim, visited, cluster, pos + (-1, 0));
    find_cluster(sim, visited, cluster, pos + (0, 1));
    find_cluster(sim, visited, cluster, pos + (1, 0));
}
