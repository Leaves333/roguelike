use crate::gamemap::{coords_to_idx, idx_to_coords};
use std::{collections::BinaryHeap, u32};

pub struct Pathfinder {
    width: u16,       // width of the board
    height: u16,      // height of the board
    costs: Vec<u32>,  // multiplier for edges that move into this cell
    dists: Vec<u32>,  // distance dp for dijkstra
    prev: Vec<usize>, // stores ancestor of each vertex, used to recover the path
    root: (u16, u16), // root location from where distance is calculated
    cardinal: u32,    // additional cost for cardinal movement
    diagonal: u32,    // additional cost for diagonal movement
}

fn in_bounds(x: i16, y: i16, width: u16, height: u16) -> bool {
    return 0 <= x && x < width as i16 && 0 <= y && y < height as i16;
}

impl Pathfinder {
    pub fn new(
        costs: Vec<u32>,
        root: (u16, u16),
        width: u16,
        height: u16,
        cardinal: u32,
        diagonal: u32,
    ) -> Self {
        assert_eq!(
            costs.len(),
            (width * height) as usize,
            "costs len does not match up with board dimensions!"
        );

        let mut pathfinder = Pathfinder {
            width,
            height,
            costs,
            dists: Vec::new(),
            prev: Vec::new(),
            root,
            cardinal,
            diagonal,
        };
        pathfinder.dijkstra();
        pathfinder
    }

    // returns shortest path from root to dest
    // last element is always dest, first element is tile adjacent to root
    pub fn path_to(&self, dest: (u16, u16)) -> Vec<(u16, u16)> {
        if self.dists[coords_to_idx(dest.0, dest.1, self.width)] == u32::MAX {
            return vec![(490, 490)];
        }

        let mut path = Vec::new();
        let mut cur = dest;
        while cur != self.root {
            path.push(cur);
            cur = idx_to_coords(
                self.prev[coords_to_idx(cur.0, cur.1, self.width)],
                self.width,
            );
        }

        path.reverse();
        path
    }

    fn dijkstra(&mut self) {
        // dijkstra is calculated once here!!!
        // and results are reused everywhere else...
        self.dists.resize(self.costs.len(), u32::MAX);
        self.prev.resize(self.costs.len(), usize::MAX);

        let mut heap = BinaryHeap::new();

        heap.push(std::cmp::Reverse((0, self.root)));
        let cardinal_dirs = vec![(1, 0), (0, 1), (-1, 0), (0, -1)];
        let diagonal_dirs = vec![(1, 1), (-1, 1), (1, -1), (-1, -1)];

        while let Some(std::cmp::Reverse((cost, (x, y)))) = heap.pop() {
            // this is not the current best distance
            if cost > self.dists[coords_to_idx(x, y, self.width)] {
                continue;
            }

            for (dx, dy) in cardinal_dirs.iter().chain(diagonal_dirs.iter()) {
                if !in_bounds(x as i16 + dx, y as i16 + dy, self.width, self.height) {
                    continue; // destination is not in bounds
                }

                let (target_x, target_y) = ((x as i16 + dx) as u16, (y as i16 + dy) as u16);
                let target_idx = coords_to_idx(target_x, target_y, self.width);
                let cur_idx = coords_to_idx(x, y, self.width);

                if self.costs[target_idx] <= 0 {
                    continue;
                }

                let step_cost = if dx.abs() + dy.abs() == 1 {
                    self.cardinal
                } else {
                    self.diagonal
                };
                let target_dist = cost + self.costs[target_idx] * step_cost;

                if self.dists[target_idx] > target_dist {
                    self.dists[target_idx] = target_dist;
                    self.prev[target_idx] = cur_idx;
                    heap.push(std::cmp::Reverse((target_dist, (target_x, target_y))));
                }
            }
        }
    }
}
