// use super::{HttpLeaf, WalkerTree};
//
// #[test]
// fn separing_by_parts() {
//     let path = "a/b/c/d".to_string();
//     let by_parts: Vec<_> = path.split('/').collect();
//
//     assert_eq!(by_parts.as_slice(), &["a", "b", "c", "d"]);
//
//     let path = "/a/b/c/d/".to_string();
//     let by_parts: Vec<_> = path.split('/').collect();
//
//     assert_eq!(by_parts.as_slice(), &["", "a", "b", "c", "d", ""]);
// }
//
// #[test]
// fn get_common_path() {
//     let tree_1 = WalkerTree::new_leaf(HttpLeaf {
//         path: "".to_string(),
//         rel_path: "a/b/c".to_string(),
//         file_path: "".into(),
//         output_path: "".into(),
//         content: None,
//     });
//     let tree_2 = WalkerTree::new_leaf(HttpLeaf {
//         path: "".to_string(),
//         rel_path: "a/b/d".to_string(),
//         file_path: "".into(),
//         output_path: "".into(),
//         content: None,
//     });
//
//     assert_eq!(
//         tree_1.get_common_path(tree_2.rel_path),
//         Some("a/b".to_string())
//     );
// }
//
// #[test]
// fn resolve_import() {
//     let leaf = HttpLeaf {
//         path: "".to_string(),
//         rel_path: "".to_string(),
//         file_path: "/project/path/routes/file1.ts".into(),
//         output_path: "/project/path/.densky/http/file1.ts".into(),
//         content: None,
//     };
//
//     assert_eq!(
//         leaf.resolve_import("../utils/foo.ts"),
//         Some("../../utils/foo.ts".to_string())
//     );
//     assert_eq!(leaf.resolve_import("module"), Some("module".to_string()));
// }
//
// #[test]
// fn get_import() {
//     let leaf = HttpLeaf {
//         path: "".to_string(),
//         rel_path: "".to_string(),
//         file_path: "/project/path/routes/file1.ts".into(),
//         output_path: "/project/path/.densky/http/file1.ts".into(),
//         content: None,
//     };
//
//     let content = "
// import toString from \"module-a\";
// import { a, b } from \"../foo.ts\";
// import \"./side.ts\"
//
// function get_add() {
//     return toString(a + b);
// }
// ";
//     let result = leaf.get_imports(content.to_string()).unwrap();
//     let exp_1 = "import toString from \"module-a\";
// import { a, b } from \"../../foo.ts\";
// import \"../../routes/side.ts\"";
//     let exp_2 = "
//
// function get_add() {
//     return toString(a + b);
// }
// ";
//     let expected = (exp_1.to_string(), exp_2.to_string());
//     assert_eq!(result, expected);
// }
