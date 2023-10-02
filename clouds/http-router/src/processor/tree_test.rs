// use std::{cell::RefCell, rc::Rc};
//
// use super::WalkerTree;
//
// const OUTPUT_DIR: &'static str = "output";
//
// macro_rules! create_root {
//     () => {{
//         let root = HttpTree {
//             output_path: format!("/{}/_index.ts", OUTPUT_DIR).into(),
//             is_root: true,
//             ..Default::default()
//         };
//         Rc::new(RefCell::new(root))
//     }};
// }
//
// macro_rules! add_child {
//     ($root:ident, $path:expr) => {{
//         HttpTree::add_child(
//             $root.clone(),
//             &mut HttpTree {
//                 rel_path: format!("{}", $path),
//                 path: format!("/{}", $path),
//                 output_path: format!("/{}/{}.ts", OUTPUT_DIR, $path).into(),
//                 ..Default::default()
//             },
//             &OUTPUT_DIR.to_string(),
//         );
//         format!("/{}/{}.ts", OUTPUT_DIR, $path)
//     }};
// }
//
// #[test]
// pub fn get_middlewares() {
//     let root = create_root!();
//
//     let mid_root = add_child!(root, "_middleware");
//     add_child!(root, "foo/bar");
//     let mid_foo = add_child!(root, "foo/_middleware");
//     add_child!(root, "faz/bar");
//     add_child!(root, "faz/_middleware");
//
//     let root = root.borrow();
//     let foo = root.children.get(0).unwrap().borrow();
//     let bar = foo.children.get(0).unwrap().borrow();
//
//     assert_eq!(bar.get_middlewares(), [(0, mid_root), (1, mid_foo)]);
// }
