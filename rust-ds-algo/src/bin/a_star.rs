use itertools::{enumerate, Itertools};
use rand::Rng;
use std::cmp::{max, Ordering};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::time::Duration;

#[derive(Debug, Clone, Eq, PartialEq)]
struct PriorityState {
    cost: usize,
    pos: (i32, i32),
}

impl PartialOrd<Self> for PriorityState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.pos.cmp(&other.pos))
    }
}

pub struct PathfindingResult {
    nodes_visited: usize,
    cost_of_path: usize,
    path: Vec<(i32, i32)>,
}

pub struct NeighborIterator<'a> {
    mid: (i32, i32),
    visited: u8,
    grid: &'a Vec<Vec<MapNode>>,
    size: (usize, usize),
}

impl<'a> NeighborIterator<'a> {
    pub fn new(mid: (i32, i32), grid: &'a Vec<Vec<MapNode>>, size: (usize, usize)) -> Self {
        NeighborIterator {
            mid,
            visited: 0,
            grid,
            size,
        }
    }
}

impl<'a> Iterator for NeighborIterator<'a> {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        let east = (self.mid.0, self.mid.1 + 1);
        let north = (self.mid.0 - 1, self.mid.1);
        let west = (self.mid.0, self.mid.1 - 1);
        let south = (self.mid.0 + 1, self.mid.1);

        let mut res = None;

        while self.visited < 4 {
            let mut current = (0, 0);
            match self.visited {
                0 => {
                    if east.1 >= self.size.1 as i32 {
                        self.visited += 1;
                        continue;
                    } else {
                        current = east;
                    }
                }
                1 => {
                    if north.0 < 0 {
                        self.visited += 1;
                        continue;
                    } else {
                        current = north;
                    }
                }
                2 => {
                    if west.1 < 0 {
                        self.visited += 1;
                        continue;
                    } else {
                        current = west;
                    }
                }
                3 => {
                    if south.0 >= self.size.0 as i32 {
                        self.visited += 1;
                        continue;
                    } else {
                        current = south;
                    }
                }
                _ => return None,
            }

            match self
                .grid
                .get(current.0 as usize)
                .and_then(|col| col.get(current.1 as usize))
            {
                Some(MapNode::Path(_)) => {
                    self.visited += 1;
                    return Some(current);
                }
                _ => {
                    self.visited += 1;
                }
            }
        }

        res
    }
}

#[derive(Clone, Debug)]
pub enum MapNode {
    VerticalObstacle,
    HorizontalObstacle,
    Path(i32),
    Custom(String),
}

#[derive(Clone, Debug)]
struct Map {
    width: usize,
    height: usize,
    grid: Vec<Vec<MapNode>>,
    agent_pos: (i32, i32),
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut vec = vec![vec![MapNode::Path(0); width]; height];
        Map {
            width,
            height,
            grid: vec,
            agent_pos: (0, 0),
        }
    }

    pub fn generate_map(&mut self, obstacle_num: (usize, usize)) {
        let mut rng = rand::thread_rng();
        let obstacle_num = rng.gen_range(obstacle_num.0..obstacle_num.1);
        println!("Obstacles: {}", obstacle_num);

        for row in &mut self.grid {
            for i in 0..row.len() {
                row[i] = MapNode::Path(rng.gen_range(0..12));
            }
        }

        let mut placed_obstacle = 0;
        while placed_obstacle != obstacle_num {
            let is_horizontal = rng.gen_bool(0.5);
            let length = match is_horizontal {
                true => rng.gen_range(self.width / 10..self.width / 3),
                false => rng.gen_range(self.height / 10..self.height / 3),
            };
            let row = rng.gen_range(0..self.height);
            let col = rng.gen_range(0..self.width);
            match self.grid.get(row).and_then(|row| row.get(col)) {
                Some(MapNode::Path(_)) => {
                    if is_horizontal {
                        let length = if col + length > self.width - 1 {
                            self.width
                        } else {
                            col + length
                        };
                        for l in col..length {
                            if l <= self.width {
                                let mut grid_row = self.grid.get_mut(row).unwrap();
                                grid_row.remove(l);
                                grid_row.insert(l, MapNode::HorizontalObstacle);
                            }
                        }
                    } else {
                        let length = if row + length > self.height - 1 {
                            self.height
                        } else {
                            row + length
                        };
                        for l in row..length {
                            if l <= self.height {
                                let mut grid_row = self.grid.get_mut(l).unwrap();
                                grid_row.remove(col);
                                grid_row.insert(col, MapNode::VerticalObstacle);
                            }
                        }
                    }
                    placed_obstacle += 1;
                }
                _ => (),
            }
        }
    }

    pub fn print(&self) {
        let horizontal_space = self.width.to_string().len();
        let mut map = String::new();
        map.push_str(&" ".repeat(self.height.to_string().len() + 3));
        map.push_str(
            &(0..self.width)
                .map(|n| {
                    format!(
                        "{}{}",
                        n,
                        " ".repeat(horizontal_space - n.to_string().len() + 1)
                    )
                })
                .format(" ")
                .to_string(),
        );
        map.push_str("\n");
        map.push_str(&" ".repeat(self.height.to_string().len() + 2));
        map.push_str(
            &(0..self.width)
                .map(|n| "┈".repeat(horizontal_space))
                .format(&" ".repeat(horizontal_space))
                .to_string(),
        );
        map.push_str("\n");
        for (i, row) in self.grid.iter().enumerate() {
            if i != 0 {
                map.push_str("\n");
            }
            let space = self.height.to_string().len() - i.to_string().len();
            map.push_str(&format!("{}{}| ", i, " ".repeat(space + 1)));
            for (j, col) in row.iter().enumerate() {
                map.push_str(match col {
                    MapNode::VerticalObstacle => "┃",
                    MapNode::HorizontalObstacle => "━",
                    MapNode::Path(cost) => {
                        if self.agent_pos.0 == i as i32 && self.agent_pos.1 == j as i32 {
                            "╳"
                        } else {
                            if *cost <= 5 {
                                "·"
                            } else {
                                "☷"
                            }
                        }
                    }
                    MapNode::Custom(char) => char.as_str(),
                });
                if j != row.len() - 1 {
                    map.push_str(&" ".repeat(horizontal_space + 1));
                }
            }
        }

        println!("{}", map);
    }

    pub fn neighbors(&self, node: (i32, i32)) -> NeighborIterator {
        return NeighborIterator::new(node, &self.grid, (self.height, self.width));
    }

    pub fn draw(&mut self, node: (i32, i32), char: &str) {
        if let Some(mut col) = self.grid.get_mut(node.0 as usize) {
            col.remove(node.1 as usize);
            col.insert(node.1 as usize, MapNode::Custom(char.to_string()));
        };
    }

    pub fn render_path(&mut self, title: &str, path: &Vec<(i32, i32)>, fps: usize) {
        let sleep = (1.0 / (fps as f64) * 1000.0).floor();
        for (i, p) in path.iter().skip(1).enumerate() {
            print!("\x1B[2J");
            println!("{}", sleep);
            println!("{}", title);
            let char = if i == path.len() { "✯" } else { "☐" };
            self.draw(*p, char);
            self.print();
            std::thread::sleep(Duration::from_millis(sleep as u64));
        }
    }

    pub fn find_path_bfs(&self, goal: (i32, i32)) -> Option<PathfindingResult> {
        let mut frontier = VecDeque::new();
        frontier.push_front(self.agent_pos);
        let mut current = None;
        let mut history = HashMap::new();
        history.insert(self.agent_pos, self.agent_pos);

        while !frontier.is_empty() {
            current = frontier.pop_back();
            if current == Some(goal) {
                break;
            }

            if let Some(current) = current {
                for neighbor in self.neighbors(current) {
                    if !history.contains_key(&neighbor) {
                        frontier.push_front(neighbor);
                        history.insert(neighbor, current);
                    }
                }
            }
        }

        self.construct_path(goal, &history)
    }

    fn construct_path(
        &self,
        goal: (i32, i32),
        history: &HashMap<(i32, i32), (i32, i32)>,
    ) -> Option<PathfindingResult> {
        let mut path = Vec::new();
        let mut next = history.get(&goal);
        if next.is_none() {
            return None;
        }

        path.push(goal);

        while let Some(n) = next {
            path.push(*n);
            if *n == self.agent_pos {
                break;
            }
            next = history.get(n);
        }

        path.reverse();
        let mut cost_of_path = 0;
        for p in &path {
            cost_of_path += self
                .grid
                .get(p.0 as usize)
                .map(|col| match col.get(p.1 as usize) {
                    Some(MapNode::Path(cost)) => *cost as usize,
                    _ => 0,
                })
                .unwrap_or(0);
        }

        Some(PathfindingResult {
            cost_of_path,
            path,
            nodes_visited: history.len(),
        })
    }

    pub fn find_path_dijkstra(&self, goal: (i32, i32)) -> Option<PathfindingResult> {
        let mut frontier = BinaryHeap::new();
        frontier.push(PriorityState {
            cost: 0,
            pos: self.agent_pos,
        });
        let mut current = None;
        let mut history = HashMap::new();
        history.insert(self.agent_pos, self.agent_pos);
        let mut cost_until = HashMap::new();
        cost_until.insert(self.agent_pos, 0);

        while !frontier.is_empty() {
            current = frontier.pop();

            if let Some(current) = current {
                if current.pos == goal {
                    break;
                }
                for neighbor in self.neighbors(current.pos) {
                    let new_cost =
                        *cost_until.get(&current.pos).unwrap() + self.cost_adjacent(neighbor);
                    if !cost_until.contains_key(&neighbor)
                        || new_cost < *cost_until.get(&neighbor).unwrap()
                    {
                        cost_until.insert(neighbor, new_cost);
                        frontier.push(PriorityState {
                            cost: new_cost,
                            pos: neighbor,
                        });
                        history.insert(neighbor, current.pos);
                    }
                }
            }
        }

        self.construct_path(goal, &history)
    }

    pub fn find_path_gbfs(&self, goal: (i32, i32)) -> Option<PathfindingResult> {
        let mut frontier = BinaryHeap::new();
        frontier.push(PriorityState {
            cost: 0,
            pos: self.agent_pos,
        });
        let mut current = None;
        let mut history = HashMap::new();
        history.insert(self.agent_pos, self.agent_pos);

        while !frontier.is_empty() {
            current = frontier.pop();
            if let Some(current) = current {
                if current.pos == goal {
                    break;
                }
                for neighbor in self.neighbors(current.pos) {
                    if !history.contains_key(&neighbor) {
                        frontier.push(PriorityState {
                            cost: self.heuristic(goal, neighbor),
                            pos: neighbor,
                        });
                        history.insert(neighbor, current.pos);
                    }
                }
            }
        }

        self.construct_path(goal, &history)
    }

    pub fn find_path_a_star(&self, goal: (i32, i32)) -> Option<PathfindingResult> {
        let mut frontier = BinaryHeap::new();
        frontier.push(PriorityState {
            cost: 0,
            pos: self.agent_pos,
        });
        let mut current = None;
        let mut history = HashMap::new();
        history.insert(self.agent_pos, self.agent_pos);
        let mut cost_until = HashMap::new();
        cost_until.insert(self.agent_pos, 0);

        while !frontier.is_empty() {
            current = frontier.pop();

            if let Some(current) = current {
                if current.pos == goal {
                    break;
                }
                for neighbor in self.neighbors(current.pos) {
                    let new_cost =
                        *cost_until.get(&current.pos).unwrap() + self.cost_adjacent(neighbor);
                    if !cost_until.contains_key(&neighbor)
                        || new_cost < *cost_until.get(&neighbor).unwrap()
                    {
                        cost_until.insert(neighbor, new_cost);
                        frontier.push(PriorityState {
                            cost: new_cost + self.heuristic(goal, neighbor),
                            pos: neighbor,
                        });
                        history.insert(neighbor, current.pos);
                    }
                }
            }
        }

        self.construct_path(goal, &history)
    }

    fn cost_adjacent(&self, target: (i32, i32)) -> usize {
        let cost = self
            .grid
            .get(target.0 as usize)
            .map(|col| col.get(target.1 as usize))
            .flatten();

        if cost.is_none() {
            usize::MAX
        } else {
            match cost {
                Some(MapNode::Path(c)) => *c as usize,
                _ => usize::MAX,
            }
        }
    }

    fn heuristic(&self, a: (i32, i32), b: (i32, i32)) -> usize {
        ((a.0 - b.0).abs() + (a.1 - b.1).abs()) as usize
    }
}

fn main() {
    let mut res = HashMap::new();
    let mut map = Map::new(20, 20);
    map.generate_map((20, 50));
    map.print();

    let mut new_map = map.clone();
    res.insert("Breadth First Search", new_map.find_path_bfs((10, 10)));
    let mut new_map = map.clone();
    res.insert("Dijkstra", new_map.find_path_dijkstra((10, 10)));
    let mut new_map = map.clone();
    res.insert("Greedy Best First Search", new_map.find_path_gbfs((10, 10)));
    let mut new_map = map.clone();
    res.insert("A*", new_map.find_path_a_star((10, 10)));

    for (title, path_res) in &res {
        if let Some(path_res) = path_res {
            map.render_path(title, &path_res.path, 2);
        } else {
            println!("Unreachable goal");
            return;
        }
    }

    for (title, path_res) in &res {
        if let Some(path_res) = path_res {
            println!(
                "{} - Overall path cost: {}, Nodes covered: {}",
                title, path_res.cost_of_path, path_res.nodes_visited
            );
        } else {
            return;
        }
    }
}
