use std::cmp::min;

use json::{object, JsonValue};

#[derive(Debug, Clone, Copy)]
pub struct Tile<T> {
    item: Option<T>,
}

#[derive(Debug, Clone)]
pub struct TileGrid<T> {
    pub rows: usize,
    pub cols: usize,
    tiles: Vec<Tile<T>>,
}

impl<T> TileGrid<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut result = TileGrid {
            rows,
            cols,
            tiles: Vec::with_capacity(rows * cols),
        };

        for _ in 0..rows * cols {
            result.tiles.push(Tile { item: None });
        }

        return result;
    }

    // important we return reference, T has no restraints
    pub fn get(&self, pos: (usize, usize)) -> &Option<T> {
        return &self.tiles[pos_to_index(pos, self.size())].item;
    }

    pub fn get_from_index(&self, index: usize) -> &Option<T> {
        assert!(index < self.tiles.len());
        return &self.tiles[index].item;
    }

    pub fn set(&mut self, pos: (usize, usize), current: Option<T>) {
        let index = pos_to_index(pos, self.size());
        self.tiles[index].item = current;
    }

    // Returns (rows, cols)
    pub fn size(&self) -> (usize, usize) {
        return (self.rows, self.cols);
    }
}
impl<T> TileGrid<T>
where
    T: Clone,
{
    pub fn resize(&mut self, new_rows: usize, new_cols: usize) {
        let mut new_grid = Self::new(new_rows, new_cols);

        for x in 0..min(self.cols, new_grid.cols) {
            for y in 0..min(self.rows, new_grid.rows) {
                let tile = self.get((x, y));
                if tile.is_some() {
                    new_grid.set((x, y), tile.clone())
                }
            }
        }

        *self = new_grid;
    }
}

pub fn pos_to_index(pos: (usize, usize), size: (usize, usize)) -> usize {
    let (x, y) = pos;
    let (rows, cols) = size;
    assert!(x < cols && y < rows, "check bounds x: {x} y: {y}");
    // Who would win in a fight? me or this index?
    // return x + y * rows;
    // return x * cols + y;
    return x + y * cols;
}

pub fn index_to_pos(index: usize, size: (usize, usize)) -> (usize, usize) {
    let (rows, cols) = size;
    assert!(index < rows * cols, "check bounds index: {index}");

    let pos = (index % cols, index / cols);

    assert!(pos_to_index(pos, size) == index, "check index: {index} -> {pos:?}");
    
    return pos;
}

pub trait ToAndFromJsonValue
where
    Self: Sized,
{
    fn to_json(&self) -> JsonValue;
    fn from_json(json: &JsonValue) -> Option<Self>;
}

impl<T> ToAndFromJsonValue for TileGrid<T>
where
    T: ToAndFromJsonValue,
{
    fn to_json(&self) -> JsonValue {
        let mut json_object = object! {
            version: "1.0",
            "rows": self.rows,
            "cols": self.cols,
            tiles: {},
            // list: [],
        };
        // why don't i put multiple different representations in here?
        // make it parse how you want?
        // Seems too theoretical. who would actually do that?
        // maybe just the sparse grid and a list of rows?

        for j in 0..self.rows {
            for i in 0..self.cols {
                if let Some(to_push) = self
                    .get((i, j))
                    .as_ref()
                    .and_then(|value| Some(value.to_json()))
                {
                    // Put space in here? easier to parse without?
                    let i_by_j = format!("({i},{j})");
                    json_object["tiles"][i_by_j] = to_push;
                }
            }
        }

        return json_object;
    }

    fn from_json(source: &JsonValue) -> Option<Self> {
        assert_eq!(source["version"], "1.0");

        let mut new_grid: TileGrid<T> = TileGrid::new(
            source["rows"].as_usize().expect("rows exists"),
            source["cols"].as_usize().expect("cols exists"),
        );

        source["tiles"].entries().for_each(|(key, value)| {
            let pos = key[1..key.len() - 1]
                .split_once(",")
                .map(|(x, y)| (x.parse().unwrap(), y.parse().unwrap()))
                .expect("Parse index's correctly");

            new_grid.set(pos, Some(T::from_json(value).expect("Valid Value")));
        });

        // TODO? reuse as list of rows?
        // source["list"].members().enumerate().for_each(|(i, val)| {
        //     new_grid.tiles[i].item = T::from_json(val.clone());
        // });

        return Some(new_grid);
    }
}
