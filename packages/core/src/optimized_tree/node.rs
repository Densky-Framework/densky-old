use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

use densky_adapter::{
    log::PathDebugDisplay,
    log_debug, log_trace,
    utils::{Color, Fmt, StringStripExtend},
    AHashMap, CloudFileResolve, OptimizedTreeLeaf,
};

use crate::utils::next_node_id;

use super::OptimizedTreeContainer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptimizedTreeNodeInsertResult {
    None,
    /// `(new_parent, new_suffix)`
    /// Move current to `new_parent` with `new_suffix`
    Resolve(u64, String),
    RemoveNode,

    /// `(new_node, new_suffix)`
    MergeNodes(u64, String), // ResolveAndDeleteNode()
}

#[derive(Clone, Debug, Default)]
pub struct OptimizedTreeNode {
    pub id: u64,
    /// Absolute path
    pub pathname: String,

    /// Relative path to parent
    pub relative_pathname: String,

    pub input_path: Option<PathBuf>,
    /// Output file path
    pub output_path: Option<PathBuf>,

    pub static_children: AHashMap<String, u64>,
    pub dynamic_children: AHashMap<String, u64>,
    pub index: Option<u64>,
    pub dynamic: Option<(u64, String)>,

    pub is_root: bool,
    pub is_static: bool,
    pub varname: Option<String>,
}

impl Hash for OptimizedTreeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.id);
    }
}

impl OptimizedTreeNode {
    pub fn new(
        relative_pathname: String,
        input_path: Option<PathBuf>,
        output_path: PathBuf,
    ) -> Self {
        Self {
            id: next_node_id(),
            pathname: relative_pathname.clone(),
            relative_pathname,
            input_path,
            output_path: Some(output_path),
            static_children: AHashMap::new(),
            dynamic_children: AHashMap::new(),
            index: None,
            dynamic: None,
            is_root: false,
            is_static: false,
            varname: None,
        }
    }

    pub fn new_leaf(
        relative_pathname: String,
        input_path: Option<PathBuf>,
        output_path: PathBuf,
    ) -> Self {
        Self {
            id: next_node_id(),
            pathname: relative_pathname.clone(),
            relative_pathname,
            input_path,
            output_path: Some(output_path),
            static_children: AHashMap::new(),
            dynamic_children: AHashMap::new(),
            index: None,
            dynamic: None,
            is_root: false,
            is_static: false,
            varname: None,
        }
    }

    pub fn new_child(
        parent_pathname: &String,
        relative_pathname: String,
        output_path: Option<PathBuf>,
    ) -> Self {
        let pathname = parent_pathname.to_owned()
            + "/"
            + relative_pathname.strip_prefix_if_can(&parent_pathname);
        Self {
            id: next_node_id(),
            pathname: pathname.strip_prefix_if_can(&"/").to_string(),
            relative_pathname,
            input_path: None,
            output_path,
            static_children: AHashMap::new(),
            dynamic_children: AHashMap::new(),
            index: None,
            dynamic: None,
            is_root: false,
            is_static: false,
            varname: None,
        }
    }

    /// # Returns
    /// + Some(Middle node) -> Leaf requires a re-resolve for a new parent and new relative paths.
    /// + None -> Leaf was inserted on self
    pub fn insert(
        &mut self,
        leaf_id: u64,
        file_resolved: CloudFileResolve,
        container: &mut OptimizedTreeContainer,
    ) -> OptimizedTreeNodeInsertResult {
        if file_resolved == CloudFileResolve::Ignore {
            return OptimizedTreeNodeInsertResult::None;
        }

        let leaf_rw = container
            .nodes
            .get(leaf_id)
            .expect("New leaf should be inserted on container before use it");

        let leaf = leaf_rw.read().unwrap();
        log_trace!(["OTreeNode"] "Inserting <{1:?}> {0:#?}", leaf.pathname, file_resolved);

        match file_resolved {
            CloudFileResolve::Pass => {
                let static_pathname = leaf.relative_pathname.strip_prefix_if_can("/").to_string();

                drop(leaf);
                let mut leaf = leaf_rw.write().unwrap();
                leaf.relative_pathname = static_pathname.clone();
                leaf.is_static = true;
                leaf.varname = None;

                self.static_children.insert(static_pathname, leaf_id);

                OptimizedTreeNodeInsertResult::None
            }
            CloudFileResolve::Dynamic(prefix, varname, suffix) => {
                macro_rules! return_fix_suffix {
                    ($id:expr) => {
                        if suffix.len() == 0 {
                            OptimizedTreeNodeInsertResult::None
                        } else {
                            OptimizedTreeNodeInsertResult::Resolve($id, suffix)
                        }
                    };
                }

                if prefix.len() == 0 {
                    return if suffix.len() == 0 {
                        self.dynamic = Some((leaf_id, varname));
                        self.index = leaf.index.clone();
                        OptimizedTreeNodeInsertResult::None
                    } else {
                        let mut cloned_leaf = OptimizedTreeNode::new_child(
                            &self.pathname,
                            format!("{varname}"),
                            None,
                        );
                        cloned_leaf.is_static = false;
                        cloned_leaf.varname = Some(varname.clone());

                        let cloned_leaf = container.nodes.add(cloned_leaf);
                        self.dynamic = Some((cloned_leaf, varname));
                        OptimizedTreeNodeInsertResult::Resolve(cloned_leaf, suffix)
                    };
                }

                // Search for a common child in the children
                let common_child: Option<(&u64, Option<String>)> = 'common_child: {
                    if let Some(new_parent) = self.dynamic_children.get(&prefix) {
                        Some((new_parent, None))
                    } else {
                        for child in self.dynamic_children.iter() {
                            break 'common_child OptimizedTreeNode::get_common_path(
                                &prefix, &child.0,
                            )
                            .and_then(|common_path| {
                                Some((
                                    child.1,
                                    // Some(_) when common_path is not equal
                                    // to complete prefix or child relative path
                                    if &common_path == child.0 {
                                        None
                                    } else {
                                        (common_path != prefix).then_some(common_path)
                                    },
                                ))
                            });
                        }
                        None
                    }
                };

                match common_child {
                    // Has a common child and needs a merge
                    Some((common_id, Some(common_path))) => {
                        let common_child = container.nodes.get_reader(*common_id).unwrap();
                        log_debug!(["OTreeNode"] "Merging children ({}, {}) from {common_path}", common_child.relative_pathname, leaf.relative_pathname);

                        let old_common_child_pathname = common_child.relative_pathname.clone();
                        let new_common_child_pathname = common_child
                            .relative_pathname
                            .strip_prefix(&common_path)
                            .unwrap()
                            .strip_prefix_if_can("/")
                            .to_string();

                        // Unlock container.nodes from inmutable reference
                        drop(common_child);

                        container
                            .nodes
                            .get_writer(*common_id)
                            .unwrap()
                            .relative_pathname = new_common_child_pathname.to_owned();

                        let new_leaf_pathname = leaf
                            .relative_pathname
                            .strip_prefix(&common_path)
                            .unwrap()
                            .strip_prefix_if_can("/")
                            .to_string();

                        let mut new_parent =
                            OptimizedTreeNode::new_child(&self.pathname, common_path.clone(), None);
                        new_parent.is_static = false;

                        new_parent
                            .dynamic_children
                            .insert(new_common_child_pathname, *common_id);

                        let new_parent = container.nodes.add(new_parent);

                        // Borrow self as mutable makes this imposible, so we do
                        // it before the borrow and just see the magik
                        self.dynamic_children.insert(common_path, new_parent);
                        self.dynamic_children
                            .remove(&old_common_child_pathname)
                            .expect("Child doesn't exist?");

                        OptimizedTreeNodeInsertResult::MergeNodes(new_parent, new_leaf_pathname)
                    }
                    // Has a common child, resolve to that child as the new parent
                    Some((common_id, None)) => {
                        log_trace!(["OTreeNode"] "Common child encountered {common_id}");
                        let new_parent = container.nodes.get_writer(*common_id).unwrap();

                        if let Some(old_leaf_id) = &new_parent.dynamic {
                            return_fix_suffix!(old_leaf_id.0)
                        } else {
                            let prefix = leaf
                                .relative_pathname
                                .strip_prefix_if_can(&new_parent.relative_pathname)
                                .strip_prefix_if_can("/")
                                .to_string();
                            // println!("{prefix}");

                            OptimizedTreeNodeInsertResult::Resolve(*common_id, prefix)
                        }
                    }
                    // Insert it as normal node
                    None => {
                        let mut new_parent =
                            OptimizedTreeNode::new_child(&self.pathname, prefix.clone(), None);
                        new_parent.is_static = false;

                        let cloned_leaf = if suffix.len() == 0 {
                            new_parent.dynamic = Some((leaf_id, varname));
                            None
                        } else {
                            let mut cloned_leaf = OptimizedTreeNode::new_child(
                                &new_parent.pathname,
                                format!("{varname}"),
                                None,
                            );
                            cloned_leaf.is_static = false;
                            cloned_leaf.varname = Some(varname.clone());
                            let cloned_leaf = container.nodes.add(cloned_leaf);
                            new_parent.dynamic = Some((cloned_leaf, varname));

                            Some(cloned_leaf)
                        };

                        drop(leaf);
                        let mut leaf = container.nodes.get_writer(leaf_id).unwrap();
                        leaf.relative_pathname = leaf
                            .relative_pathname
                            .strip_prefix_if_can(&prefix)
                            .strip_prefix_if_can("/")
                            .to_string();
                        drop(leaf);

                        let new_parent = container.nodes.add(new_parent);
                        self.dynamic_children.insert(prefix, new_parent);

                        if let Some(cloned_leaf) = cloned_leaf {
                            OptimizedTreeNodeInsertResult::Resolve(cloned_leaf, suffix)
                        } else {
                            OptimizedTreeNodeInsertResult::None
                        }
                    }
                }
            }
            CloudFileResolve::Index => {
                let last_slash = leaf
                    .pathname
                    .chars()
                    .rev()
                    .position(|x| x == '/')
                    .map(|x| leaf.relative_pathname.len() - x);

                let last_slash = match last_slash {
                    Some(expr) if expr > 2 => expr - 1,
                    _ => {
                        self.index =
                            Some(leaf.index.expect("Inserted leaf should have 'index' field"));
                        return OptimizedTreeNodeInsertResult::RemoveNode;
                    }
                };

                let parent = leaf.pathname[0..last_slash].to_string();
                if parent.len() == 0 {
                    self.index = Some(leaf.index.expect("Inserted leaf should have 'index' field"));
                    return OptimizedTreeNodeInsertResult::RemoveNode;
                }

                drop(leaf);
                container.nodes.get_writer(leaf_id).unwrap().pathname = parent.clone();

                OptimizedTreeNodeInsertResult::Resolve(container.nodes.hash(self), parent)
            }
            CloudFileResolve::SingleThorn(name) => {
                let path = &leaf.pathname;
                let last_slash = path
                    .chars()
                    .rev()
                    .position(|f| f == '/')
                    .map(|f| path.len() - f - 1)
                    .filter(|f| f > &0);

                let path = if let Some(last_slash) = last_slash {
                    &path[0..last_slash]
                } else {
                    &path
                };

                container
                    .single_thorn
                    .insert(name.into(), path.to_string(), leaf_id);
                OptimizedTreeNodeInsertResult::None
            }
            _ => OptimizedTreeNodeInsertResult::None,
        }
    }

    /// Get the shared path between two branchs.
    /// Eg.
    /// ```
    /// # use densky_core::optimized_tree::OptimizedTreeNode;
    /// #
    /// // Just need the relative path
    /// let branch_1 = "a/b/c/and/more".to_owned();
    /// let branch_2 = "a/b/some/other".to_owned();
    ///
    /// let common_path = OptimizedTreeNode::get_common_path(branch_1, branch_2);
    ///
    /// assert_eq!(common_path, Some("a/b".to_string()));
    /// ```
    pub fn get_common_path(path_a: &String, path_b: &String) -> Option<String> {
        // The accumulator of common path
        let mut carrier = String::new();

        // All segments of the path: a/b/c -> vec!["a", "b", "c"]
        let by_segments = path_b.split('/');
        for segment in by_segments {
            // Prevent wrong paths like "a//b/c", "/a/b/c" or "a/b/c/"
            if segment.len() == 0 {
                continue;
            }

            let is_first = carrier.len() == 0;
            // The expected path
            let next = if is_first {
                segment.to_owned()
            } else {
                format!("{}/{}", &carrier, &segment)
            };

            if !path_a.starts_with(&next) {
                if is_first {
                    return None;
                } else {
                    return Some(carrier);
                }
            }

            carrier.reserve_exact(segment.len() + 1);
            if !is_first {
                carrier += "/";
            }
            carrier += segment;
        }

        return Some(path_b.clone());
    }

    pub fn display(
        &self,
        f: &mut std::fmt::Formatter,
        container: &OptimizedTreeContainer,
    ) -> std::fmt::Result {
        let has_leaf = self.index.is_some();
        let name = format!(
            "{} {}",
            match (self.is_root, has_leaf) {
                (true, true) => Color::FgYellow.color("★"),
                (true, false) => Color::FgYellow.color("☆"),
                (false, true) => Color::FgYellow.color("▲"),
                (false, false) => Color::FgYellow.color("△"),
            },
            Color::Bold.color(&self.relative_pathname)
        );
        f.write_str(&name)?;

        // if self.middleware.is_some() {
        //     f.write_str(&format!(
        //         "\n{} ■ {}",
        //         "|".dimmed().bright_black(),
        //         "middleware".bright_black()
        //     ))?;
        // }

        for (name, thorns) in container
            .single_thorn
            .get_all_of(&self.pathname)
            .into_iter()
        {
            write!(
                f,
                "\n{0}",
                Color::custom([Color::Dim, Color::Bold])
                    .color(format!("|  _ {name}({})", thorns.len())),
            )?;
        }

        if let Some((dynamic_id, _)) = &self.dynamic {
            let dynamic = container.nodes.get_reader(*dynamic_id).unwrap();
            let dynamic = format!("{}", Fmt(move |f| dynamic.display(f, &container)));

            // write!(
            //     f,
            //     "\n{} {}",
            //     Color::custom([Color::Dim, Color::Bold]).color("|"),
            //     Color::custom([Color::Dim, Color::Bold])
            //         .color("--- ".to_string() + varname + " ---")
            // )?;
            for line in dynamic.split("\n") {
                write!(
                    f,
                    "\n{0} {1}",
                    Color::custom([Color::Dim, Color::Bold]).color("|+"),
                    line
                )?;
            }
        }

        let mut static_children = self.static_children.keys().collect::<Vec<_>>();
        static_children.sort();
        for child in static_children {
            let child = self.static_children.get(child).unwrap();
            let child = container.nodes.get_reader(*child).unwrap();
            let fmtd = format!("{}", Fmt(move |f| child.display(f, &container)));

            for line in fmtd.split("\n") {
                write!(
                    f,
                    "\n{} {}",
                    Color::custom([Color::Dim, Color::Bold]).color("|-"),
                    line
                )?;
            }
        }

        // if !self.dynamic_children.is_empty() {
        //     write!(
        //         f,
        //         "\n{}",
        //         Color::custom([Color::Dim, Color::Bold]).color("| --- Dynamic Children ---"),
        //     )?;
        // }

        let mut dynamic_children = self.dynamic_children.keys().collect::<Vec<_>>();
        dynamic_children.sort();
        for child in dynamic_children {
            let child = self.dynamic_children.get(child).unwrap();
            let child = container.nodes.get_reader(*child).unwrap();
            let fmtd = format!("{}", Fmt(move |f| child.display(f, &container)));

            for line in fmtd.split("\n") {
                write!(
                    f,
                    "\n{} {}",
                    Color::custom([Color::Dim, Color::Bold]).color("| "),
                    line
                )?;
            }
        }
        // if self.fallback.is_some() {
        //     f.write_str(&format!(
        //         "\n{} {}",
        //         "|".dimmed().bright_black(),
        //         "...fallback".bright_black()
        //     ))?;
        // }

        Ok(())
    }

    pub fn debug(
        &self,
        f: &mut std::fmt::Formatter,
        container: &OptimizedTreeContainer,
    ) -> std::fmt::Result {
        let name = format!(
            "{}OptimizedTreeNode ({})",
            if self.is_root { "ROOT - " } else { "" },
            self.relative_pathname
        );
        let index = 'index: {
            if let Some(index) = &self.index {
                let self_hash = container.nodes.hash(self);
                if index == &self_hash {
                    break 'index "<RECURSIVE NODE>".to_owned();
                }

                let index = container.nodes.get_reader(*index);
                if let Some(index) = index {
                    let index = format!("{:#?}", Fmt(|f| index.debug(f, &container)));
                    index
                } else {
                    "<DELETED NODE>".to_owned()
                }
            } else {
                "None".into()
            }
        };

        let mut static_children: String = "{\n".into();
        for (child_pathname, child_id) in &self.static_children {
            let child = container.nodes.get_reader(*child_id).unwrap();
            let child = format!("{:#?}", Fmt(|f| child.debug(f, &container)));
            let mut child = child
                .split('\n')
                .map(|line| format!("  {line}\n"))
                .collect::<String>();
            child.pop();

            static_children.reserve_exact(2 + child_pathname.len() + 1 + child.len() + 2);
            static_children.push_str("  ");
            static_children.push_str(&child_pathname);
            static_children.push_str(":");
            static_children.push_str(&child);
            static_children.push_str(",\n");
        }
        static_children.push('}');

        let mut dynamic_children: String = "{\n".into();
        for (child_pathname, child_id) in &self.dynamic_children {
            let child = container.nodes.get_reader(*child_id).unwrap();
            let child = format!("{:#?}", Fmt(|f| child.debug(f, &container)));
            let mut child = child
                .split('\n')
                .map(|line| format!("  {line}\n"))
                .collect::<String>();
            child.pop();

            dynamic_children.reserve_exact(2 + child_pathname.len() + 1 + child.len() + 2);
            dynamic_children.push_str("  ");
            dynamic_children.push_str(&child_pathname);
            dynamic_children.push_str(":");
            dynamic_children.push_str(&child);
            dynamic_children.push_str(",\n");
        }
        dynamic_children.push('}');

        let dynamic_child = if let Some((dynamic, _)) = self.dynamic {
            let dynamic = container.nodes.get_reader(dynamic).unwrap();
            let dynamic = format!("{:#?}", Fmt(|f| dynamic.debug(f, &container)));
            let mut dynamic = dynamic
                .split('\n')
                .map(|line| line.to_owned() + "\n")
                .collect::<String>();
            dynamic.pop();

            dynamic
        } else {
            "None".to_owned()
        };

        f.debug_struct(name.as_str())
            .field("path", &format_args!("{}", self.input_path.display_debug()))
            .field("output_path", &self.output_path.display_debug())
            .field("static_children", &format_args!("{static_children}"))
            .field("dynamic_children", &format_args!("{dynamic_children}"))
            .field("index", &format_args!("{index}"))
            .field("dynamic", &format_args!("{dynamic_child}"))
            .finish()
    }
}

impl OptimizedTreeNode {
    pub fn into_leaf(&self, container: &OptimizedTreeContainer) -> OptimizedTreeLeaf {
        let mut single_thorns: AHashMap<String, Vec<String>> = AHashMap::new();

        for (name, thorns) in container
            .single_thorn
            .get_all_of(&self.pathname)
            .into_iter()
        {
            single_thorns.insert(
                name,
                thorns
                    .into_iter()
                    .map(|f| {
                        container
                            .nodes
                            .get_reader(f)
                            .unwrap()
                            .input_path
                            .clone()
                            .unwrap()
                            .display()
                            .to_string()
                    })
                    .collect(),
            );
        }

        OptimizedTreeLeaf {
            pathname: "/".to_string() + self.pathname.as_str(),
            relative_pathname: self.relative_pathname.clone(),

            // Try with it own input_path, but probably it doesn't have it,
            // so then try with its index if it has.
            index: self
                .input_path
                .as_ref()
                .map(|f| f.display().to_string())
                .or_else(|| {
                    self.index.map(|f| {
                        container
                            .nodes
                            .get_reader(f)
                            .unwrap()
                            .input_path
                            .as_ref()
                            .expect("Index node should have an input_path")
                            .display()
                            .to_string()
                    })
                }),

            single_thorns,
            is_root: self.is_root,
            is_static: self.is_static,
            varname: self.varname.clone(),
        }
    }
}
