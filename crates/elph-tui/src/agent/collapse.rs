/// Tracks which transcript detail blocks are expanded (by entry index).
#[derive(Debug, Clone, Default)]
pub struct CollapseState {
    expanded: Vec<usize>,
}

impl CollapseState {
    pub fn is_expanded(&self, index: usize) -> bool {
        self.expanded.contains(&index)
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(pos) = self.expanded.iter().position(|&i| i == index) {
            self.expanded.remove(pos);
        } else {
            self.expanded.push(index);
        }
    }

    pub fn toggle_newest(&mut self, entries_len: usize) {
        if entries_len > 0 {
            self.toggle(entries_len - 1);
        }
    }
}
