use hashbrown::HashMap;

pub struct Unique<V>
where
    V: Clone,
{
    set: HashMap<V, usize>,
    all: Vec<Option<V>>,

    available: Vec<usize>,
}

impl<V> Unique<V>
where
    V: Clone + Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            set: HashMap::new(),
            all: Vec::new(),

            available: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: V) {
        if self.set.contains_key(&value) {
            return;
        }

        match self.available.pop() {
            Some(index) => {
                self.set.insert(value.clone(), index);
                self.all[index] = Some(value);
            }
            None => {
                let index = self.all.len();
                self.set.insert(value.clone(), index);
                self.all.push(Some(value));
            }
        }
    }

    pub fn remove(&mut self, value: &V) {
        if let Some(index) = self.set.remove(value) {
            self.available.push(index);
            self.all[index] = None;
        }
    }

    pub fn get_all(&self) -> Vec<V> {
        self.all.iter().filter_map(|v| v.clone()).collect()
    }
}
