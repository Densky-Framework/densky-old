extern crate densky_adapter;

use densky_adapter::macros::cloud_setup;

cloud_setup!(views::html {
    source_folder: "views",
    file_ends: ".html",
    file_strategy: SimpleTree,
    dependencies: []
});
