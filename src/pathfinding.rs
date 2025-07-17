struct Pathfinder {
    costs: Vec<i32>,  // dist array for dijkstra, non-zero values are walkable cells
    prev: Vec<u16>,   // stores ancestor of each vertex, used to recover the path
    root: (u16, u16), // root location from where distance is calculated
    cardinal: u16,    // additional cost for cardinal movement
    diagonal: u16,    // additional cost for diagonal movement
}

impl Pathfinder {
    pub fn new(costs: Vec<i32>, root: (u16, u16)) -> Self {
        todo!()

        // dijkstra is calculated once here!!!
        // and results are reused everywhere else...
    }

    // returns shortest path from dest to root
    // reverse this iter in caller's code if needed
    pub fn path_from(dest: (u16, u16)) -> impl Iterator<Item = (u16, u16)> {
        std::iter::from_fn(|| todo!())
    }
}
