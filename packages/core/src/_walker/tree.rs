use std::{
    fmt,
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::utils::{join_paths, Fmt};

use super::{container::WalkerContainer, WalkerEntity};

#[derive(Debug, Clone)]
pub struct WalkerTree {
    pub id: usize,
    /// The absolute path (url) for this leaf
    pub path: String,
    /// The path (url) relative to parent.
    pub rel_path: String,
    pub output_path: PathBuf,

    /// Children identifiers
    /// ```
    /// # use densky_core::walker::{WalkerLeaf, WalkerContainer};
    /// # let mut container = WalkerContainer::new("output_path");
    /// # let root = container.create_root();
    /// # let child_id = container.add_leaf(WalkerLeaf::new("test".into(), "a.ts", "b.ts"));
    /// # let mut root = root.lock().unwrap();
    /// # root.add_child(child_id, &mut container);
    /// #
    /// for child in &root.children {
    ///     let child = container.get_tree_locked(*child).unwrap();
    ///
    ///     assert_eq!(Some(root.id), child.parent);
    /// }
    /// ```
    pub children: Vec<usize>,

    /// Leaf id
    /// ```
    /// # use densky_core::walker::{WalkerLeaf, WalkerContainer};
    /// # let mut container = WalkerContainer::new("output_path");
    /// # let root = container.create_root();
    /// # let child_id = container.add_leaf(WalkerLeaf::new("/_index".into(), "a.ts", "b.ts"));
    /// # let mut root = root.lock().unwrap();
    /// # root.add_child(child_id, &mut container);
    /// #
    /// if let Some(leaf) = &root.leaf {
    ///     let leaf = container.get_leaf_locked(*leaf).unwrap();
    ///
    ///     assert_eq!(root.id, leaf.owner);
    /// } else {
    ///     panic!("There's no leaf");
    /// }
    /// ```
    pub leaf: Option<usize>,

    /// Middleware id
    /// ```
    /// # use densky_core::walker::{WalkerLeaf, WalkerContainer};
    /// # let mut container = WalkerContainer::new("output_path");
    /// # let root = container.create_root();
    /// # let child_id = container.add_leaf(WalkerLeaf::new("/_middleware".into(), "a.ts", "b.ts"));
    /// # let mut root = root.lock().unwrap();
    /// # root.add_child(child_id, &mut container);
    /// #
    /// if let Some(middleware) = &root.middleware {
    ///     let middleware = container.get_leaf_locked(*middleware).unwrap();
    ///
    ///     assert_eq!(root.id, middleware.owner);
    /// } else {
    ///     panic!("There's no middleware");
    /// }
    /// ```
    pub middleware: Option<usize>,

    /// Fallback id
    /// ```
    /// # use densky_core::walker::{WalkerLeaf, WalkerContainer};
    /// # let mut container = WalkerContainer::new("output_path");
    /// # let root = container.create_root();
    /// # let child_id = container.add_leaf(WalkerLeaf::new("/_fallback".into(), "a.ts", "b.ts"));
    /// # let mut root = root.lock().unwrap();
    /// # root.add_child(child_id, &mut container);
    /// #
    /// if let Some(fallback) = &root.fallback {
    ///     let fallback = container.get_leaf_locked(*fallback).unwrap();
    ///
    ///     assert_eq!(root.id, fallback.owner);
    /// } else {
    ///     panic!("There's no fallback");
    /// }
    /// ```
    pub fallback: Option<usize>,

    /// Parent id
    /// ```
    /// # use densky_core::walker::{WalkerLeaf, WalkerContainer};
    /// # let mut container = WalkerContainer::new("output_path");
    /// # let root = container.create_root();
    /// # let child_id = container.add_leaf(WalkerLeaf::new("/a".into(), "a.ts", "b.ts"));
    /// # let mut root = root.lock().unwrap();
    /// # root.add_child(child_id, &mut container);
    /// #
    /// assert!(root.parent.is_none());
    ///
    /// for child in &root.children {
    ///     let child = container.get_tree_locked(*child).unwrap();
    ///
    ///     assert_eq!(Some(root.id), child.parent);
    /// }
    /// ```
    pub parent: Option<usize>,

    /// Cached middlewares.
    ///
    /// > Note: It's just a cache, update it with `.get_middlewares()`
    pub middlewares: Vec<usize>,

    pub has_index: bool,
    pub is_container: bool,
    pub is_root: bool,
    pub is_fallback: bool,
    pub is_middleware: bool,
}

impl WalkerEntity for WalkerTree {
    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}

impl WalkerTree {
    pub fn new() -> WalkerTree {
        WalkerTree {
            id: 0,
            path: "/".to_owned(),
            rel_path: "/".to_owned(),
            output_path: "/".into(),
            children: vec![],
            leaf: None,
            middleware: None,
            middlewares: vec![],
            fallback: None,
            parent: None,
            has_index: false,
            is_container: false,
            is_root: false,
            is_fallback: false,
            is_middleware: false,
        }
    }

    pub fn new_detailed<P, R, O>(path: P, rel_path: R, output_path: O) -> WalkerTree
    where
        P: AsRef<str>,
        R: AsRef<str>,
        O: AsRef<Path>,
    {
        WalkerTree {
            id: 0,
            path: path.as_ref().to_owned(),
            rel_path: rel_path.as_ref().to_owned(),
            output_path: output_path.as_ref().to_path_buf(),
            children: vec![],
            leaf: None,
            middleware: None,
            middlewares: vec![],
            fallback: None,
            parent: None,
            has_index: false,
            is_container: false,
            is_root: false,
            is_fallback: false,
            is_middleware: false,
        }
    }

    pub fn display(&self, f: &mut fmt::Formatter<'_>, container: &WalkerContainer) -> fmt::Result {
        let has_leaf = self.leaf.is_some();
        let name = format!(
            "{} {}",
            match (self.is_root, has_leaf) {
                (true, true) => "★".yellow(),
                (true, false) => "☆".bright_yellow(),
                (false, true) => "▲".yellow(),
                (false, false) => "△".bright_yellow(),
            },
            &self.rel_path.bold()
        );
        f.write_str(&name)?;

        if self.middleware.is_some() {
            f.write_str(&format!(
                "\n{} ■ {}",
                "|".dimmed().bright_black(),
                "middleware".bright_black()
            ))?;
        }

        for child in &self.children {
            let child = container.get_tree(*child).unwrap();
            let child = child.lock().unwrap();
            let fmtd = format!("{}", Fmt(move |f| child.display(f, &container)));

            for line in fmtd.split("\n") {
                write!(f, "\n{} {}", "|".dimmed().bright_black(), line)?;
            }
        }
        if self.fallback.is_some() {
            f.write_str(&format!(
                "\n{} {}",
                "|".dimmed().bright_black(),
                "...fallback".bright_black()
            ))?;
        }

        Ok(())
    }

    #[cfg(not(debug_assertions))]
    pub fn debug(&self, _f: &mut fmt::Formatter<'_>, _container: &WalkerContainer) -> fmt::Result {
        Ok(())
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self, f: &mut fmt::Formatter<'_>, container: &WalkerContainer) -> fmt::Result {
        let name = format!(
            "{}WalkerTree ({}) {}",
            if self.is_root { "ROOT - " } else { "" },
            self.get_id(),
            if self.is_container {
                " - CONTAINER"
            } else {
                ""
            }
        );
        let leaf = if let Some(leaf) = self.leaf {
            let leaf = container.get_leaf(leaf).unwrap();
            let leaf = leaf.lock().unwrap();

            format!("Some(<Leaf ({}|{})>)", leaf.path, leaf.rel_path,)
        } else {
            "None".to_string()
        };
        let middleware = if let Some(middleware) = self.middleware {
            let middleware = container.get_leaf(middleware).unwrap();
            let middleware = middleware.lock().unwrap();
            let middleware = middleware.clone();

            Some(middleware)
        } else {
            None
        };
        let fallback = if let Some(fallback) = self.fallback {
            let fallback = container.get_leaf(fallback).unwrap();
            let fallback = fallback.lock().unwrap();
            let fallback = fallback.clone();

            Some(fallback)
        } else {
            None
        };
        let mut children = "[\n".to_owned();
        for child in &self.children {
            let child = container.get_tree(*child).unwrap();
            let child = child.lock().unwrap();
            let child = format!("{:#?}", Fmt(|f| child.debug(f, &container)));
            let child = child
                .split("\n")
                .map(|s| format!("    {}", s))
                .collect::<Vec<String>>()
                .join("\n");
            children += &child;
            children += ",\n";
        }
        children += "]";
        f.debug_struct(name.as_str())
            .field("path", &self.path)
            .field("rel_path", &self.rel_path)
            .field("output_path", &self.output_path)
            .field("children", &format_args!("{}", children))
            .field("leaf", &format_args!("{}", leaf))
            .field("middleware", &middleware)
            .field("fallback", &fallback)
            .finish()
    }

    pub fn is_convention(&self) -> bool {
        let last_part: PathBuf = (*self.path).into();
        let last_part = last_part.iter().nth_back(0).unwrap();
        let last_part = last_part.to_str().unwrap();

        // Ignore all routes that starts with '_'
        last_part == "_fallback" || last_part == "_middleware"
    }

    pub fn get_middlewares(&mut self, container: &mut WalkerContainer) -> Vec<usize> {
        if self.middlewares.len() >= 1 {
            return self.middlewares.clone();
        }

        if let Some(parent) = self.parent {
            if parent != 0 {
                let parent = container.get_tree(parent).unwrap();
                let mut parent = parent.lock().unwrap();

                self.middlewares = parent.get_middlewares(container);
            }
        }

        if let Some(my_mid) = self.middleware {
            self.middlewares.push(my_mid);
        }

        return self.middlewares.clone();
    }

    /// Verify if the path is direct child of `self` and
    /// also if ends with the provided pattern
    fn ends_with<P>(&self, path: P, pattern: &str) -> bool
    where
        P: AsRef<str>,
    {
        let path = path.as_ref();
        // ROOT    : /PATTERN
        // NO-ROOT : /PATH/PATTERN
        let slash_len = if self.is_root { 0 } else { 1 };
        let is_child = path.len() == self.path.len() + slash_len + pattern.len();

        is_child && path.ends_with(pattern)
    }

    fn add_tree_child(&mut self, child_id: usize, container: &mut WalkerContainer) -> Option<()> {
        let child = container.get_tree(child_id)?;
        let mut child = child.lock().unwrap();
        child.parent = Some(self.get_id());

        let path = &child.path;
        let leaf_id = if self.ends_with(&path, "_fallback") {
            self.fallback = child.leaf;
            child.leaf.clone()
        } else if self.ends_with(&path, "_middleware") {
            self.middleware = child.leaf;
            child.leaf.clone()
        } else if self.ends_with(&path, "_index") {
            self.leaf = child.leaf;
            self.output_path = child.output_path.clone();
            child.leaf.clone()
        } else {
            // Update relative path and fix any '/' at start
            let rel_path = &child.path[self.path.len()..];
            let rel_path = if rel_path.starts_with('/') {
                &rel_path[1..]
            } else {
                rel_path
            };
            child.rel_path = rel_path.to_string();

            self.children.push(child_id);
            None
        };

        if let Some(leaf_id) = leaf_id {
            let leaf = container.get_leaf(leaf_id).unwrap();
            let mut leaf = leaf.lock().unwrap();

            // Update relative path and fix any '/' at start
            let rel_path = &leaf.path[self.path.len()..];
            let rel_path = if rel_path.starts_with('/') {
                &rel_path[1..]
            } else {
                rel_path
            };
            leaf.rel_path = rel_path.to_string();
            leaf.owner = self.get_id();
        }

        Some(())
    }

    /// Add the child to the tree. For that exists many ways:
    /// - *fallback* (`*/_fallback`): The file is used as fallback of this route and children.
    /// - *middleware* (`*/_middleware`): Same as fallback but with the middleware
    /// - *index* (`*/_index`): Move route to be the leaf of its parent.
    /// - *Any other*: Pass through an algoritnm to decide other many ways:
    ///   + *Merge*: If two routes share some segment on the `rel_path` then make
    ///            a new tree with that segment as `rel_path` and make it as container.
    ///            Both routes are moved in to that container.
    ///   + *Pull*: If two route share some segment on the `rel_path` and the route that already
    ///           exists is a container then move the child to that container.
    ///   + *Index*: This is just for any `_index` that doesn't have a container slibing, create
    ///            a tree as container and `rel_path` equal to child owner (`rel_path` - `_index`).
    ///            Move the child the created container.
    ///   + *Any other*: Just add it as child.
    pub fn add_child(&mut self, child: usize, container: &mut WalkerContainer) {
        let child_p = container.get_leaf(child).unwrap();
        let mut child_p = child_p.lock().unwrap();
        child_p.owner = self.id;

        let path = child_p.path.clone();
        if self.ends_with(&path, "_fallback") {
            self.fallback = Some(child);
        } else if self.ends_with(&path, "_middleware") {
            self.middleware = Some(child);
        } else if self.ends_with(&path, "_index") {
            self.leaf = Some(child);
            self.output_path = child_p.output_path.clone();
        } else {
            let last_part: PathBuf = (*path).into();
            let prefix_part = match last_part.parent() {
                Some(expr) => expr,
                None => return,
            };
            let prefix_part = prefix_part.display().to_string();
            let last_part = last_part.iter().nth_back(0).unwrap();
            let last_part = last_part.to_str().unwrap();
            let is_index = last_part == "_index";

            // Ignore all routes that starts with '_'
            if last_part.starts_with('_')
                && !is_index
                && last_part != "_fallback"
                && last_part != "_middleware"
            {
                return;
            }

            // Update relative path and fix any '/' at start
            let rel_path = &path[self.path.len()..];
            let rel_path = if rel_path.starts_with('/') {
                &rel_path[1..]
            } else {
                rel_path
            };
            child_p.rel_path = rel_path.to_string();

            // When the leaf has a common path with other leaf
            // then make a common branch for both or merge on
            // the index.
            // From:
            // /convention/some-route
            // /convention/with-index
            // /convention/with-index/index-child
            //
            // To:
            // /
            // | /convention
            // | | /some-route
            // | | /with-index *
            // | | | /index-child
            //
            // Steps:
            // - Make for both:
            //   /convention/some-route
            //   /convention/with-index
            //   To:
            //   /convention
            //   | /some-route
            //   | /with-index *
            //
            // - Merge:
            //   /convention
            //   /convention/with-index/index-child
            //   To:
            //   /convention
            //   | /some-route
            //   | /with-index *
            //   | /with-index/index-child
            //
            // - Repeat
            //

            let common_path = self.children.iter().find_map(|child| {
                // println!("COMMON_PATH: {}", &child);
                container
                    .get_tree(*child)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .get_common_path(rel_path.to_owned())
                    .map(|common_path| (child, common_path))
            });

            let leaf = if let Some((common_child_id, common_path)) = common_path {
                let common_child = container.get_tree(*common_child_id).unwrap();
                let mut common_child = common_child.lock().unwrap();
                let common_child_path = common_child.path.clone();

                let is_container = common_child.is_container;
                let is_container = is_container && path.starts_with(&common_child_path);

                // If is container, then insert the new child to it
                if is_container {
                    child_p.rel_path = (&path[common_child.path.len()..]).to_owned();
                    drop(child_p);
                    common_child.add_child(child, container);
                    None
                } else {
                    drop(child_p);
                    drop(common_child);
                    // else, then merge into one common container
                    let path = if self.path.as_str() == "/" {
                        format!("/{}", common_path)
                    } else {
                        format!("{}/{}", &self.path, common_path)
                    };
                    let output = join_paths(
                        "_index.ts",
                        join_paths(&path[1..], container.get_output_dir()),
                    );
                    let mut parent = WalkerTree::new_detailed(path, common_path, output);
                    parent.is_container = true;
                    parent.parent = Some(self.get_id());

                    parent.add_tree_child(*common_child_id, container);
                    parent.add_child(child, container);

                    let parent = container.add_tree(parent);
                    Some((parent, Some(*common_child_id)))
                }
            } else if is_index {
                // If try to put an _index without sliblings, then create a
                // container for it and use the child as leaf
                let rel_path: PathBuf = (*child_p.rel_path).into();
                let rel_path = rel_path.parent().unwrap().display().to_string();

                // Update the rel_path for leaf
                child_p.rel_path = rel_path.clone();

                let mut parent =
                    WalkerTree::new_detailed(prefix_part, rel_path, child_p.output_path.clone());
                parent.leaf = Some(child);
                parent.parent = Some(self.get_id());
                parent.is_container = true;
                let parent = container.add_tree(parent);
                drop(child_p);
                Some((parent, None))
            } else {
                // If there's no common slibling, then put it inside
                let mut parent = WalkerTree::new_detailed(
                    child_p.path.clone(),
                    child_p.rel_path.clone(),
                    child_p.output_path.clone(),
                );
                parent.leaf = Some(child);
                parent.parent = Some(self.get_id());
                let parent = container.add_tree(parent);
                drop(child_p);
                Some((parent, None))
            };

            // This is for borrowing errors, all are computed on the above
            // block and the actions are executed here.
            if let Some((leaf, remove_id)) = leaf {
                if let Some(remove_id) = remove_id {
                    self.children.retain(|child| child != &remove_id);
                }
                self.children.push(leaf)
            }
        }
    }

    /// Get the shared path between two branchs.
    /// Eg.
    /// ```
    /// use densky_core::walker::WalkerTree;
    ///
    /// let branch_1 = WalkerTree::new_detailed("path", "a/b/c/and/more".to_owned(), "output_path");
    ///
    /// // Just need the relative path
    /// let branch_2 = "a/b/some/other".to_owned();
    ///
    /// let common_path = branch_1.get_common_path(branch_2).unwrap();
    ///
    /// assert_eq!(common_path, "a/b".to_string());
    /// ```
    pub fn get_common_path(&self, other_path: String) -> Option<String> {
        // All segments of the path: a/b/c -> vec!["a", "b", "c"]
        let by_segments: Vec<_> = other_path.split('/').collect();
        // The accumulator of common path
        let mut carrier = String::new();

        for segment in by_segments {
            // Prevent wrong paths like "a//b/c", "/a/b/c" or "a/b/c/"
            if segment.len() == 0 {
                return None;
            }

            let is_first = carrier.as_str() == "";
            // The expected path
            let next = if is_first {
                segment.to_owned()
            } else {
                format!("{}/{}", &carrier, &segment)
            };

            if !self.rel_path.starts_with(&next) {
                if is_first {
                    return None;
                } else {
                    return Some(carrier);
                }
            }

            if !is_first {
                carrier += "/";
            }
            carrier += segment;
        }

        return Some(other_path);
    }
}
