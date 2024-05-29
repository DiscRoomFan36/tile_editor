pub struct MyIconServer<T> {
    pub assets: Vec<(String, T)>,
    selected: String,     // TODO: use str
    default_icon: String, // TODO: use str
}

impl<T> MyIconServer<T> {
	pub fn new(assets: Vec<(String, T)>) -> Self {
		assert!(assets.len() > 1);
		Self {
			default_icon: assets[0].0.clone(),
			selected: assets[1].0.clone(),
			assets,
		}
	}

    pub fn get_selected_name(&self) -> &str {
        return &self.selected;
    }

    pub fn get_default_name(&self) -> &str {
        return &self.default_icon;
    }

    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        self.assets
            .iter()
            .find(|(asset_name, _)| name == asset_name)
            .map(|(_, handle)| handle)
    }

    pub fn get_default_handle(&self) -> &T {
        self.get_by_name(&self.default_icon)
            .expect("self.default_icon is valid")
    }

    pub fn set_selected_by_name(&mut self, name: &str) {
        assert!(self.assets.iter().any(|(asset_name, _)| name == asset_name));
        self.selected = name.to_owned();
    }

    pub fn set_default_by_name(&mut self, name: &str) {
        assert!(self.assets.iter().any(|(asset_name, _)| name == asset_name));
        self.default_icon = name.to_owned();
    }

    fn cycle_icon(&self, name: &str, count: i32) -> &str {
        let index = self
            .assets
            .iter()
            .enumerate()
            .find(|(_, (asset_name, _))| *asset_name == name)
            .map(|(i, _)| i)
            .unwrap();

        let len = self.assets.len() as i32;

        let index = ((index as i32 + count) % len + len) % len;

        self.assets
            .get(index as usize)
            .map(|(name, _)| name)
            .unwrap()
    }

    pub fn cycle_selected(&mut self, count: i32) {
        self.selected = self.cycle_icon(&self.selected, count).to_owned();
    }

    pub fn cycle_default(&mut self, count: i32) {
        self.default_icon = self.cycle_icon(&self.default_icon, count).to_owned();
    }
}