use profit_sim as sim;
use sim::{pos, BuildingKind, Id, Pos, Sim, MAX_BOARD_SIZE};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Regions {
    pub buildings: Vec<Id>,
    pub cells: Vec<Pos>,
    pub indices: Vec<(usize, usize)>,
}

impl Regions {
    pub fn new_region(&mut self) {
        self.indices.push((self.buildings.len(), self.cells.len()));
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn get<'a>(&'a self, idx: usize) -> Region<'a> {
        let (b, c) = self.indices[idx];
        match self.indices.get(idx + 1) {
            Some(&(nb, nc)) => Region {
                buildings: &self.buildings[b..nb],
                cells: &self.cells[c..nc],
            },
            None => Region {
                buildings: &self.buildings[b..],
                cells: &self.cells[c..],
            },
        }
    }

    pub fn get_mut<'a>(&'a mut self, idx: usize) -> RegionMut<'a> {
        let (b, c) = self.indices[idx];
        match self.indices.get(idx + 1) {
            Some(&(nb, nc)) => RegionMut {
                buildings: &mut self.buildings[b..nb],
                cells: &mut self.cells[c..nc],
            },
            None => RegionMut {
                buildings: &mut self.buildings[b..],
                cells: &mut self.cells[c..],
            },
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Region<'a>> {
        (0..self.len()).map(|i| self.get(i))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Region<'a> {
    pub buildings: &'a [Id],
    pub cells: &'a [Pos],
}

#[derive(Debug, Default, PartialEq)]
pub struct RegionMut<'a> {
    pub buildings: &'a mut [Id],
    pub cells: &'a mut [Pos],
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

pub fn find_regions(sim: &Sim) -> Regions {
    let mut visited = Visited::new(sim.board.width, sim.board.height);
    let mut regions = Regions::default();
    let mut position = pos(0, 0);

    loop {
        regions.new_region();
        find_region(sim, &mut visited, &mut regions, position);

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

    regions
}

fn find_region(sim: &Sim, visited: &mut Visited, regions: &mut Regions, pos: Pos) {
    if pos.x < 0 || pos.x >= visited.width || pos.y < 0 || pos.y >= visited.height {
        return;
    }

    if let Some(c) = sim.board[pos] {
        let building = &sim.buildings[c.id];
        match &building.kind {
            BuildingKind::Deposit(deposit) => {
                if !regions.buildings.contains(&c.id) {
                    regions.buildings.push(c.id);
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
        if !regions.cells.contains(&pos) {
            regions.cells.push(pos);
        }
    }

    if visited[pos] {
        return;
    }
    visited[pos] = true;

    find_region(sim, visited, regions, pos + (0, -1));
    find_region(sim, visited, regions, pos + (-1, 0));
    find_region(sim, visited, regions, pos + (0, 1));
    find_region(sim, visited, regions, pos + (1, 0));
}
