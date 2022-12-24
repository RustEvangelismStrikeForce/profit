use core::fmt;
use std::collections::HashMap;

use profit_sim::{pos, Building, Id, Pos, Sim};

#[derive(Clone, PartialEq, Eq)]
pub struct DistanceMap {
    pub width: i8,
    pub height: i8,
    cells: Vec<Option<u16>>,
}

impl<P: Into<Pos>> std::ops::Index<P> for DistanceMap {
    type Output = Option<u16>;

    fn index(&self, pos: P) -> &Self::Output {
        let pos = pos.into();
        assert!(
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height,
            "Board index out of bounds: {pos}"
        );

        &self.cells[pos.y as usize * self.width as usize + pos.x as usize]
    }
}

impl<P: Into<Pos>> std::ops::IndexMut<P> for DistanceMap {
    fn index_mut(&mut self, pos: P) -> &mut Self::Output {
        let pos = pos.into();
        assert!(
            pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height,
            "Board index out of bounds: {pos}"
        );

        &mut self.cells[pos.y as usize * self.width as usize + pos.x as usize]
    }
}

impl fmt::Debug for DistanceMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\n\x1B[7;94m    \x1B[0m")?;
        for x in 0..self.width {
            write!(f, "\x1B[7;94m{x:3}\x1B[0m")?;
        }
        for y in 0..self.height {
            write!(f, "\n\x1B[1;7;94m{y:3}\x1B[0m ")?;
            for x in 0..self.width {
                match self[pos(x, y)] {
                    Some(d) => write!(f, "{d:3}")?,
                    None => write!(f, "  .",)?,
                }
            }
        }

        Ok(())
    }
}

impl DistanceMap {
    pub fn new(width: i8, height: i8) -> Self {
        Self {
            width,
            height,
            cells: vec![None; width as usize * height as usize],
        }
    }

    pub fn get(&self, pos: impl Into<Pos>) -> Option<Option<u16>> {
        let pos = pos.into();
        if pos.x < 0 || pos.x >= self.width {
            return None;
        }
        if pos.y < 0 || pos.y >= self.height {
            return None;
        }
        Some(self[pos])
    }

    pub fn get_mut(&mut self, pos: impl Into<Pos>) -> Option<&mut Option<u16>> {
        let pos = pos.into();
        if pos.x < 0 || pos.x >= self.width {
            return None;
        }
        if pos.y < 0 || pos.y >= self.height {
            return None;
        }
        Some(&mut self[pos])
    }
}

pub fn map_deposit_distances(sim: &Sim) -> HashMap<Id, DistanceMap> {
    sim.buildings
        .iter()
        .filter_map(|(i, b)| {
            let Building::Deposit(deposit) = b else { return None };
            let map = map_distances(sim, deposit.pos, deposit.width, deposit.height);
            Some((i, map))
        })
        .collect()
}

/// Generate a map of Manhattan distances to a rectangular object
pub fn map_distances(sim: &Sim, pos: Pos, width: u8, height: u8) -> DistanceMap {
    let mut map = DistanceMap::new(sim.board.width, sim.board.height);

    for i in 0..width as i8 {
        let pos = pos + (i, -1);
        map_distance(sim, &mut map, pos, 0);
    }
    for i in 0..width as i8 {
        let pos = pos + (i, height as i8);
        map_distance(sim, &mut map, pos, 0);
    }
    for i in 0..height as i8 {
        let pos = pos + (-1, i);
        map_distance(sim, &mut map, pos, 0);
    }
    for i in 0..height as i8 {
        let pos = pos + (height as i8, i);
        map_distance(sim, &mut map, pos, 0);
    }

    map
}

fn map_distance(sim: &Sim, map: &mut DistanceMap, pos: Pos, new_dist: u16) {
    // Out of bounds
    let Some(val) = map.get_mut(pos) else { return };

    // Occupied
    if sim.board[pos].is_some() {
        return;
    }

    match val {
        Some(current_dist) if *current_dist <= new_dist => return,
        Some(_) | None => *val = Some(new_dist),
    }

    map_distance(sim, map, pos + (0, -1), new_dist + 1);
    map_distance(sim, map, pos + (-1, 0), new_dist + 1);
    map_distance(sim, map, pos + (0, 1), new_dist + 1);
    map_distance(sim, map, pos + (1, 0), new_dist + 1);
}
