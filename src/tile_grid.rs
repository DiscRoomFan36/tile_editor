use json::{
    self, object,
    JsonValue::{self, Null},
};

#[derive(Debug, Clone, Copy)]
pub struct Tile<T> {
    _id: usize, // @Cleanup: Unneeded
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

        for i in 0..rows * cols {
            result.tiles.push(Tile { _id: i, item: None });
        }

        return result;
    }

    pub fn get(&self, pos: (usize, usize)) -> &Option<T> {
        return &self.tiles[index(pos, self.get_size())].item;
    }

    pub fn set(&mut self, pos: (usize, usize), current: T) {
        let index = index(pos, self.get_size());
        self.tiles[index].item = Some(current);
    }

    pub fn get_size(&self) -> (usize, usize) {
        return (self.rows, self.cols);
    }
}

impl<T> ToAndFromJsonValue for TileGrid<T>
where
    T: ToAndFromJsonValue,
{
    fn to_json(&self) -> JsonValue {
        let mut json_object = object! {
            version: "1.0",
            rows: self.rows,
            cols: self.cols,
            list: []
        };

        // TODO: Maybe use json object and index numbers for storing,
        // less error prone, and could make sparse grid
        // eg: list { (0,0): 3, (3,5): 2 }
        for j in 0..self.cols {
            for i in 0..self.rows {
                let to_push = self
                    .get((i, j))
                    .as_ref()
                    .and_then(|value| Some(value.to_json()))
                    .unwrap_or(Null);
                let _ = json_object["list"].push(to_push);
            }
        }

        return json_object;
    }

    fn from_json(source: JsonValue) -> Option<Self> {
        assert_eq!(source["version"], "1.0");

        let mut new_grid: TileGrid<T> = TileGrid::new(
            source["rows"].as_usize().expect("rows exists"),
            source["cols"].as_usize().expect("cols exists"),
        );

        source["list"].members().enumerate().for_each(|(i, val)| {
            new_grid.tiles[i].item = T::from_json(val.clone());
        });

        return Some(new_grid);
    }
}

pub trait ToAndFromJsonValue
where
    Self: Sized,
{
    fn to_json(&self) -> JsonValue;
    fn from_json(json: JsonValue) -> Option<Self>;
}

fn index(pos: (usize, usize), size: (usize, usize)) -> usize {
    let (x, y) = pos;
    let (rows, cols) = size;
    assert!(x < rows && y < cols, "check bounds x: {x} y: {y}");
    return x + y * rows;
}
