fn main() {
    cynic_codegen::register_schema("saleor")
        .from_sdl_file("../schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
