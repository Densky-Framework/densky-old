use std::sync::{Arc, RwLock};

use crate::{utils::Fmt, CloudFileResolve};

use super::{node::OptimizedTreeNodeInsertResult, OptimizedTreeContainer, OptimizedTreeNode};

fn create_dummy_leaf(
    container: &mut OptimizedTreeContainer,
    path: impl Into<String>,
    file_name: impl Into<String>,
) -> u64 {
    let path = path.into();
    let file_name = file_name.into();
    let file_path = format!("FILE/{file_name}");
    let output_path = format!("OUTPUT/{file_name}");

    let dummy_leaf = OptimizedTreeNode::new_leaf(
        path.clone() + ".dummy",
        Some(file_path.clone().into()),
        output_path.clone().into(),
    );
    let mut leaf = OptimizedTreeNode::new_leaf(path, Some(file_path.into()), output_path.into());
    let dummy_leaf = container.nodes.add(dummy_leaf);
    leaf.index = Some(dummy_leaf);

    container.nodes.add(leaf)
}

fn insert_dummy_leaf(
    container: &mut OptimizedTreeContainer,
    root: Arc<RwLock<OptimizedTreeNode>>,
    path: impl Into<String>,
    file_name: impl Into<String>,
    resolved: CloudFileResolve,
) -> (u64, OptimizedTreeNodeInsertResult) {
    let leaf = create_dummy_leaf(container, path, file_name);
    let insert_result = root.write().unwrap().insert(leaf, resolved, container);

    (leaf, insert_result)
}

#[test]
fn simple_static_insert() {
    let mut container = OptimizedTreeContainer::new("OUTPUT_DIR");
    let root = container.create_root();

    let (static_leaf, insert_result) = insert_dummy_leaf(
        &mut container,
        root.clone(),
        "/a/b",
        "a/b.ts",
        CloudFileResolve::Pass,
    );

    assert_eq!(insert_result, OptimizedTreeNodeInsertResult::None);

    let root_static_children = &root.read().unwrap().static_children;
    assert!(
        root_static_children.len() == 1,
        "root should insert 1 node, but it inserts {} nodes",
        root_static_children.len()
    );

    assert_eq!(
        root_static_children.get("a/b"),
        Some(&static_leaf),
        "root should insert leaf as \"a/b\", but it inserts it as {:?}",
        root_static_children.keys().nth(0).unwrap()
    );
}

#[test]
fn multi_static_insert() {
    let mut container = OptimizedTreeContainer::new("OUTPUT_DIR");
    let root = container.create_root();

    let (leaf1, insert_result) = insert_dummy_leaf(
        &mut container,
        root.clone(),
        "/a/b",
        "a/b.ts",
        CloudFileResolve::Pass,
    );
    assert_eq!(insert_result, OptimizedTreeNodeInsertResult::None);

    let (leaf2, insert_result) = insert_dummy_leaf(
        &mut container,
        root.clone(),
        "/a/shared-prefix",
        "a/shared-prefix.ts",
        CloudFileResolve::Pass,
    );
    assert_eq!(insert_result, OptimizedTreeNodeInsertResult::None);

    let (leaf3, insert_result) = insert_dummy_leaf(
        &mut container,
        root.clone(),
        "/other/path",
        "other/path.ts",
        CloudFileResolve::Pass,
    );
    assert_eq!(insert_result, OptimizedTreeNodeInsertResult::None);

    let root_static_children = &root.read().unwrap().static_children;
    assert!(
        root_static_children.len() == 3,
        "root should insert 1 node, but it inserts {} nodes",
        root_static_children.len()
    );

    assert_eq!(
        root_static_children.get("a/b"),
        Some(&leaf1),
        "root should insert leaf 1 as \"a/b\", but it inserts {:?}",
        root_static_children
    );

    assert_eq!(
        root_static_children.get("a/shared-prefix"),
        Some(&leaf2),
        "root should insert leaf 2 as \"a/shared-prefix\", but it inserts {:?}",
        root_static_children
    );

    assert_eq!(
        root_static_children.get("other/path"),
        Some(&leaf3),
        "root should insert leaf 3 as \"other/path\", but it inserts {:?}",
        root_static_children
    );
}

#[test]
fn simple_dynamic_insert() {
    let mut container = OptimizedTreeContainer::new("OUTPUT_DIR");
    let root = container.create_root();

    let (_, insert_result) = insert_dummy_leaf(
        &mut container,
        root.clone(),
        "/a/$b",
        "a/b.ts",
        CloudFileResolve::Dynamic("a".into(), "b".into(), "".into()),
    );

    assert_eq!(insert_result, OptimizedTreeNodeInsertResult::None);

    let root_dynamic_children = &root.read().unwrap().dynamic_children;
    assert!(
        root_dynamic_children.len() == 1,
        "root should insert 1 node, but it inserts {} nodes",
        root_dynamic_children.len()
    );

    assert!(
        root_dynamic_children.get("a").is_some(),
        "root should insert leaf as \"a\", but it inserts it as {:?}",
        root_dynamic_children.keys().nth(0).unwrap()
    );
}

#[test]
fn multi_dynamic_insert() {
    let mut container = OptimizedTreeContainer::new("OUTPUT_DIR");
    let root = container.create_root();

    let (leaf, result) = insert_dummy_leaf(
        &mut container,
        root.clone(),
        "a/b/$c/d",
        "a/b/$c/d.ts",
        CloudFileResolve::Dynamic("a/b".into(), "$c".into(), "d".into()),
    );
    println!("{result:?}");

    let OptimizedTreeNodeInsertResult::Resolve(new_parent, new_relative) = result else {
        panic!("Insert result should be OptimizedTreeNodeInsertResult::Resolve");
    };

    container.nodes.get_writer(leaf).unwrap().relative_pathname = new_relative;

    let b = container.nodes.get(new_parent).unwrap();
    let a = b
        .write()
        .unwrap()
        .insert(leaf, CloudFileResolve::Pass, &mut container);

    println!("{a:?}");
    println!("{}", Fmt(|f| root.read().unwrap().display(f, &container)));

    let (leaf, result) = insert_dummy_leaf(
        &mut container,
        root.clone(),
        "a/$b/c",
        "a/$b/c.ts",
        CloudFileResolve::Dynamic("a".into(), "$b".into(), "c".into()),
    );
    println!("{result:?}");

    let OptimizedTreeNodeInsertResult::Resolve(new_parent, new_relative) = result else {
        panic!("Insert result should be OptimizedTreeNodeInsertResult::MergeNodes");
    };

    container.nodes.get_writer(leaf).unwrap().relative_pathname = new_relative;

    let b = container.nodes.get(new_parent).unwrap();
    let result = b.write().unwrap().insert(
        leaf,
        CloudFileResolve::Dynamic("".into(), "$b".into(), "c".into()),
        &mut container,
    );
    println!("{result:?}");

    let OptimizedTreeNodeInsertResult::Resolve(new_parent, new_relative) = result else {
        panic!("Insert result should be OptimizedTreeNodeInsertResult::Resolve");
    };

    container.nodes.get_writer(leaf).unwrap().relative_pathname = new_relative;

    let b = container.nodes.get(new_parent).unwrap();
    let a = b
        .write()
        .unwrap()
        .insert(leaf, CloudFileResolve::Pass, &mut container);

    println!("{a:?}");

    println!("{}", Fmt(|f| root.read().unwrap().display(f, &container)));
}
