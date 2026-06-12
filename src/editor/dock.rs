pub struct DockTab {
    pub id: String,
    pub name: String,
    pub closeable: bool,
}

pub struct DockSlot {
    pub title: String,
    tabs: Vec<DockTab>,
    active: usize,
}

impl DockSlot {
    pub fn new(title: &str, tabs: Vec<DockTab>) -> Self {
        Self {
            title: title.to_string(),
            tabs,
            active: 0,
        }
    }

    pub fn from_single(title: &str, id: &str, name: &str) -> Self {
        Self::new(
            title,
            vec![DockTab {
                id: id.to_string(),
                name: name.to_string(),
                closeable: false,
            }],
        )
    }

    pub fn active_id(&self) -> Option<&str> {
        self.tabs.get(self.active).map(|t| t.id.as_str())
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn set_active_index(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active = idx;
        }
    }

    pub fn add_tab(&mut self, tab: DockTab) {
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
    }

    pub fn remove_tab(&mut self, id: &str) {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(idx);
            if self.active >= self.tabs.len() {
                self.active = self.tabs.len().saturating_sub(1);
            }
        }
    }
}
