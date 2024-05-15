use json::{object, JsonValue};

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

    // @Think: Should I accept an option? Only if I have a legitimate use case.
    pub fn set(&mut self, pos: (usize, usize), current: T) {
        let index = index(pos, self.get_size());
        self.tiles[index].item = Some(current);
    }

    pub fn get_size(&self) -> (usize, usize) {
        return (self.rows, self.cols);
    }
}

fn index(pos: (usize, usize), size: (usize, usize)) -> usize {
    let (x, y) = pos;
    let (rows, cols) = size;
    assert!(x < cols && y < rows, "check bounds x: {x} y: {y}");
    // Who would win in a fight? me or this index?
    // return x + y * rows;
    // return x * cols + y;
    return x + y * cols;
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

            new_grid.set(pos, T::from_json(value).expect("Valid Value"))
        });

        // TODO? reuse as list of rows?
        // source["list"].members().enumerate().for_each(|(i, val)| {
        //     new_grid.tiles[i].item = T::from_json(val.clone());
        // });

        return Some(new_grid);
    }
}
