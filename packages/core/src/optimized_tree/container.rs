use std::{
    fmt,
    hash::Hash,
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use densky_adapter::{log_debug, log_error, utils::join_paths, AHashMap};

use super::OptimizedTreeNode;

type AsyncNode<T> = Arc<RwLock<T>>;

/// `Optimized tree container`
/// It contains all tree context to avoid errors with mutable borrowings.
pub struct OptimizedTreeContainer {
    output_dir: String,
    root: Option<u64>,
    pub nodes: SingleContainer<OptimizedTreeNode>,
    pub leafs: SingleContainer<()>,
    pub single_thorn: SingleThornContainer,
}

impl OptimizedTreeContainer {
    /// Create new tree context
    pub fn new<O>(output_dir: O) -> OptimizedTreeContainer
    where
        O: AsRef<str>,
    {
        OptimizedTreeContainer {
            output_dir: output_dir.as_ref().to_string(),
            root: None,
            nodes: SingleContainer::new(),
            leafs: SingleContainer::new(),
            single_thorn: SingleThornContainer::new(),
        }
    }

    /// Get the output directory cloned
    pub fn get_output_dir(&self) -> String {
        self.output_dir.clone()
    }

    /// Create root node.
    /// This will overwrite the root
    pub fn create_root(&mut self) -> AsyncNode<OptimizedTreeNode> {
        let mut root = OptimizedTreeNode::default();
        root.pathname = String::new();
        root.output_path = Some(join_paths("_index", &self.output_dir).into());
        root.is_root = true;

        let root_id = self.nodes.add(root);
        self.root = Some(root_id);

        self.nodes.get(root_id).unwrap()
    }

    /// Try to get root node.
    /// If the root isn't setted `None` is returned.
    /// ```
    /// # use densky_core::optimized_tree::WalkerContainer;
    /// #
    /// let mut container = WalkerContainer::new("output_dir");
    ///
    /// assert!(container.get_root().is_none()); // No root
    ///
    /// container.create_root();
    /// assert!(container.get_root().is_some()); // Expected node
    /// ```
    pub fn get_root(&self) -> Option<AsyncNode<OptimizedTreeNode>> {
        if let Some(root) = self.root {
            self.nodes.get(root)
        } else {
            None
        }
    }

    /// If the root isn't setted `None` is returned.
    pub fn get_root_id(&self) -> Option<u64> {
        self.root.clone()
    }

    pub fn iter(&self) -> impl Iterator<Item = AsyncNode<OptimizedTreeNode>> + '_ {
        self.nodes.inner.iter().map(|f| f.1).cloned()
    }

    pub fn transverse(&self) -> Vec<AsyncNode<OptimizedTreeNode>> {
        self.transverse_node(self.root.unwrap())
    }

    pub fn transverse_node(&self, id: u64) -> Vec<AsyncNode<OptimizedTreeNode>> {
        let root = self.nodes.get(id).unwrap();

        let dynamic_children = &root.read().unwrap().dynamic_children;
        let static_children = &root.read().unwrap().static_children;
        let mut out = Vec::with_capacity(1 + dynamic_children.len() + static_children.len());

        out.push(root.clone());

        for (_, id) in static_children.iter() {
            let a = self.transverse_node(*id);
            for b in a.into_iter() {
                out.push(b);
            }
        }

        for (_, id) in dynamic_children.iter() {
            let a = self.transverse_node(*id);
            for b in a.into_iter() {
                out.push(b);
            }
        }

        out
    }
}

impl IntoIterator for OptimizedTreeContainer {
    type Item = AsyncNode<OptimizedTreeNode>;
    type IntoIter = std::collections::hash_map::IntoValues<u64, Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.inner.into_values()
    }
}

pub struct SingleContainer<T> {
    pub(super) inner: AHashMap<u64, AsyncNode<T>>,
}

impl<T> SingleContainer<T> {
    pub fn new() -> SingleContainer<T> {
        SingleContainer {
            inner: AHashMap::new(),
        }
    }

    pub fn hash(&self, x: impl Hash) -> u64 {
        self.inner.hasher().hash_one(x)
    }

    pub fn insert(&mut self, id: u64, x: T) {
        let is_overwriting = self.inner.insert(id, Arc::new(RwLock::new(x))).is_some();
        if is_overwriting {
            panic!("{}", "Nodes should be unique. Overwriting: {id}");
        }
    }

    pub fn get(&self, id: u64) -> Option<AsyncNode<T>> {
        self.inner.get(&id).cloned()
    }

    pub fn get_by(&self, x: impl Hash) -> Option<AsyncNode<T>> {
        self.get(self.hash(x))
    }

    pub fn get_reader(&self, id: u64) -> Option<RwLockReadGuard<T>> {
        self.inner.get(&id).and_then(|x| x.read().ok())
    }

    pub fn get_writer(&self, id: u64) -> Option<RwLockWriteGuard<T>> {
        self.inner.get(&id).and_then(|x| x.write().ok())
    }

    pub fn remove(&mut self, id: u64) {
        log_debug!(["SingleContainer"] "Node deleted: {id}");
        self.inner.remove(&id);
    }
}

impl<T: Hash> SingleContainer<T> {
    pub fn add(&mut self, x: T) -> u64 {
        let id = self.hash(&x);
        self.insert(id, x);
        id
    }
}

impl<T: fmt::Debug> SingleContainer<T> {
    #[cfg(not(debug_assertions))]
    pub fn debug(&self) {}

    #[cfg(debug_assertions)]
    pub fn debug(&self) {
        log_debug!(["optimized_tree::SingleContainer"] "{self:#?}");
    }
}

impl<T: fmt::Debug> fmt::Debug for SingleContainer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

#[derive(Debug)]
pub struct SingleThornContainer {
    inner: AHashMap<String, AHashMap<String, u64>>,
}

impl SingleThornContainer {
    pub fn new() -> Self {
        Self {
            inner: AHashMap::new(),
        }
    }

    pub fn has(&self, name: &String, path: &String) -> bool {
        self.get(name, path).is_some()
    }

    pub fn get(&self, name: &String, path: &String) -> Option<u64> {
        if let Some(thorns) = self.inner.get(path) {
            thorns.get(name).copied()
        } else {
            None
        }
    }

    pub fn get_all(&self, name: &String, path: &String) -> Vec<u64> {
        let mut out = Vec::new();

        if let Some(id) = self.get(name, path) {
            out.push(id);
        }

        let mut path: PathBuf = path.into();
        while path.pop() {
            if let Some(id) = self.get(name, &path.display().to_string()) {
                out.push(id);
            }
        }

        out
    }

    pub fn get_all_on(&self, path: &String) -> AHashMap<String, u64> {
        let mut out = AHashMap::new();

        if let Some(thorns) = self.inner.get(path) {
            for (name, id) in thorns {
                out.insert(name.clone(), *id);
            }
        }

        out
    }

    pub fn get_all_of(&self, path: &String) -> AHashMap<String, Vec<u64>> {
        let mut out = AHashMap::new();

        if let Some(thorns) = self.inner.get(path) {
            for (name, id) in thorns {
                let vec = vec![*id];
                out.insert(name.clone(), vec);
            }
        }

        let mut path: PathBuf = path.into();
        while path.pop() {
            if let Some(thorns) = self.inner.get(&path.display().to_string()) {
                for (name, id) in thorns {
                    if let Some(vec) = out.get_mut(name) {
                        vec.push(*id);
                    } else {
                        let vec = vec![*id];
                        out.insert(name.clone(), vec);
                    }
                }
            }
        }

        out
    }

    pub fn insert(&mut self, name: String, path: String, node: u64) -> bool {
        let thorns = if let Some(thorns) = self.inner.get_mut(&path) {
            thorns
        } else {
            self.inner.insert(path.clone(), AHashMap::new());
            self.inner.get_mut(&path).unwrap()
        };

        let old_node = thorns.insert(name.clone(), node);
        if let Some(old_node) = old_node {
            log_error!(["OptimizedTree"] "Conflicting thorn {name:?}. Inserting {node} when {old_node} already is {name:?} thorn.");
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod test {
    use super::SingleThornContainer;

    #[test]
    fn single_thorn_container() {
        let mut container = SingleThornContainer::new();
        container.insert("middleware".into(), "a/b/c".into(), 1);
        container.insert("middleware".into(), "a".into(), 3);
        container.insert("middleware".into(), "a/b".into(), 2);
        container.insert("fallback".into(), "a/b".into(), 4);

        assert!(container.inner.len() == 3);

        let middlewares = container.get_all(&"middleware".into(), &"a/b/c".into());
        assert_eq!(middlewares, [1, 2, 3]);
    }
}
