use std::sync::{Arc, Mutex, MutexGuard};

use crate::utils::join_paths;

use super::{WalkerEntity, WalkerLeaf, WalkerTree};

type SyncNode<T> = Arc<Mutex<T>>;

/// `Walker container`
/// It contains all tree context to avoid errors with mutable borrowings.
pub struct WalkerContainer {
    output_dir: String,
    root: Option<usize>,
    tree: Vec<SyncNode<WalkerTree>>,
    leaf: Vec<SyncNode<WalkerLeaf>>,
}

impl WalkerContainer {
    /// Create new tree context
    pub fn new<O>(output_dir: O) -> WalkerContainer
    where
        O: AsRef<str>,
    {
        WalkerContainer {
            output_dir: output_dir.as_ref().to_string(),
            root: None,
            tree: vec![],
            leaf: vec![],
        }
    }

    /// Get the output directory cloned
    pub fn get_output_dir(&self) -> String {
        self.output_dir.clone()
    }

    /// Create root node.
    /// This will overwrite the root
    pub fn create_root(&mut self) -> SyncNode<WalkerTree> {
        let mut root = WalkerTree::new();
        root.id = self.id_tree();
        root.output_path = join_paths("_index", &self.output_dir).into();
        root.is_root = true;
        self.root = Some(root.id);

        let root = Arc::new(Mutex::new(root));
        self.tree.push(root.clone());

        root
    }

    /// Returns the next id for tree
    pub fn id_tree(&self) -> usize {
        self.tree.len() + 1
    }

    /// Returns the next id for leaf
    pub fn id_leaf(&self) -> usize {
        self.leaf.len() + 1
    }

    /// Add a tree node to container
    pub fn add_tree(&mut self, mut new_node: WalkerTree) -> usize {
        let new_id = self.id_tree();
        new_node.set_id(new_id.clone());
        self.tree.push(Arc::new(Mutex::new(new_node)));
        new_id
    }

    /// Add a leaf node to container
    pub fn add_leaf(&mut self, mut new_node: WalkerLeaf) -> usize {
        let new_id = self.id_leaf();
        new_node.set_id(new_id.clone());
        self.leaf.push(Arc::new(Mutex::new(new_node)));
        new_id
    }

    /// Try to get tree node from an id.
    /// Cases:
    /// - `Some(node)`: Expected.
    /// - `None`: The node doesn't exists or id is invalid.
    /// ```
    /// # use densky_core::walker::{WalkerContainer, WalkerTree};
    /// #
    /// let mut container = WalkerContainer::new("output_dir");
    /// container.add_tree(WalkerTree::new());
    ///
    /// assert!(container.get_tree(1).is_some()); // Expected node
    /// assert!(container.get_tree(0).is_none()); // Id must be greater than zero
    /// assert!(container.get_tree(2).is_none()); // Node doesn't exist
    /// ```
    pub fn get_tree(&self, id: usize) -> Option<SyncNode<WalkerTree>> {
        self.tree.get(id.checked_sub(1)?).cloned()
    }

    /// Try to get tree node from an id and lock it.
    /// Cases:
    /// - `Some(node)`: Expected.
    /// - `None`: The node doesn't exists, id is negative or it's already locked.
    /// ```
    /// # use densky_core::walker::{WalkerContainer, WalkerTree};
    /// #
    /// let mut container = WalkerContainer::new("output_dir");
    /// container.add_tree(WalkerTree::new());
    ///
    /// let node = container.get_tree_locked(1); // Lock it
    /// assert!(node.is_some()); // Expected node
    /// assert!(container.get_tree_locked(0).is_none()); // Id must be greater than zero
    /// assert!(container.get_tree_locked(2).is_none()); // Node doesn't exist
    /// assert!(container.get_tree_locked(1).is_none()); // It's already locked
    /// ```
    pub fn get_tree_locked(&self, id: usize) -> Option<MutexGuard<'_, WalkerTree>> {
        let arc = self.tree.get(id.checked_sub(1)?)?;
        match arc.try_lock() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    /// Try to get leaf node from an id.
    /// Cases:
    /// - `Some(node)`: Expected.
    /// - `None`: The node doesn't exists or id is invalid.
    /// ```
    /// # use densky_core::walker::{WalkerContainer, WalkerLeaf};
    /// #
    /// let mut container = WalkerContainer::new("output_dir");
    /// container.add_leaf(WalkerLeaf::new("path".into(), "file_path", "output_path"));
    ///
    /// assert!(container.get_leaf(1).is_some()); // Expected node
    /// assert!(container.get_leaf(0).is_none()); // Id must be greater than zero
    /// assert!(container.get_leaf(2).is_none()); // Node doesn't exist
    /// ```
    pub fn get_leaf(&self, id: usize) -> Option<SyncNode<WalkerLeaf>> {
        self.leaf.get(id.checked_sub(1)?).cloned()
    }

    /// Try to get leaf node from an id.
    /// Cases:
    /// - `Some(node)`: Expected.
    /// - `None`: The node doesn't exists or id is invalid.
    /// ```
    /// # use densky_core::walker::{WalkerContainer, WalkerLeaf};
    /// #
    /// let mut container = WalkerContainer::new("output_dir");
    /// container.add_leaf(WalkerLeaf::new("path".into(), "file_path", "output_path"));
    ///
    /// let node = container.get_leaf_locked(1); // Lock it
    /// assert!(node.is_some()); // Expected node
    /// assert!(container.get_leaf_locked(0).is_none()); // Id must be greater than zero
    /// assert!(container.get_leaf_locked(2).is_none()); // Node doesn't exist
    /// assert!(container.get_leaf_locked(1).is_none()); // It's already locked
    /// ```
    pub fn get_leaf_locked(&self, id: usize) -> Option<MutexGuard<'_, WalkerLeaf>> {
        let arc = self.leaf.get(id.checked_sub(1)?)?;
        match arc.try_lock() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    /// Try to get root node.
    /// Cases:
    /// - `Some(node)`: Expected.
    /// - `None`: The root node is not specified.
    /// ```
    /// # use densky_core::walker::WalkerContainer;
    /// #
    /// let mut container = WalkerContainer::new("output_dir");
    ///
    /// assert!(container.get_root().is_none()); // No root
    ///
    /// container.create_root();
    /// assert!(container.get_root().is_some()); // Expected node
    /// ```
    pub fn get_root(&self) -> Option<SyncNode<WalkerTree>> {
        if let Some(root) = self.root {
            self.get_tree(root)
        } else {
            None
        }
    }

    /// Try to get root node.
    /// Cases:
    /// - `Some(node)`: Expected.
    /// - `None`: The root node is not specified.
    /// ```
    /// # use densky_core::walker::WalkerContainer;
    /// #
    /// let mut container = WalkerContainer::new("output_dir");
    ///
    /// assert!(container.get_root_locked().is_none()); // No root
    ///
    /// container.create_root();
    /// let root = container.get_root_locked(); // Lock it
    /// assert!(root.is_some()); // Expected node
    ///
    /// assert!(container.get_root_locked().is_none()); // It's already locked
    /// ```
    pub fn get_root_locked(&self) -> Option<MutexGuard<'_, WalkerTree>> {
        if let Some(root) = self.root {
            self.get_tree_locked(root)
        } else {
            None
        }
    }

    /// Get root id.
    /// Cases:
    /// - `Some(id)`: Expected
    /// - `None`: There's no root
    pub fn get_root_id(&self) -> Option<usize> {
        self.root.clone()
    }

    #[cfg(not(debug_assertions))]
    pub fn debug_tree(&self) {}

    #[cfg(debug_assertions)]
    pub fn debug_tree(&self) {
        println!(
            "[CONTAINER TREE VIEW DEBUG]\n{:#?}\n[/CONTAINER TREE VIEW DEBUG]",
            &self.tree
        );
    }

    #[cfg(not(debug_assertions))]
    pub fn debug_leaf(&self) {}

    #[cfg(debug_assertions)]
    pub fn debug_leaf(&self) {
        println!(
            "[CONTAINER LEAF VIEW DEBUG]\n{:#?}\n[/CONTAINER LEAF VIEW DEBUG]",
            &self.leaf
        );
    }
}
