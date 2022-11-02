use bpf_script::types::{AddToTypeDatabase, TypeDatabase};
use bpf_script_derive::AddToTypeDatabase;

#[repr(C, align(1))]
#[derive(AddToTypeDatabase)]
struct InnerType {
    pub _inner_array: [u8; 10],
    pub _u64: u64,
}

#[repr(C, align(1))]
#[derive(AddToTypeDatabase)]
struct CustomStructure {
    pub _u8: u8,
    pub _u16: u16,
    pub _u32: u32,
    pub _u64: u64,
    pub _i8: i8,
    pub _i16: i16,
    pub _i32: i32,
    pub _i64: i64,
    pub _array: [u8; 10],
    pub _array2d: [[u8; 32]; 10],
    pub _inner: InnerType,
    pub _array3d: [[[u32; 100]; 32]; 10],
}

fn main() {
    let mut database = TypeDatabase::default();
    CustomStructure::add_to_database(&mut database)
        .expect("Failed to add CustomStructure to type database.");

    database
        .get_type_by_name("CustomStructure")
        .expect("New type wasn't added.");
    database
        .get_type_by_name("[u8; 10]")
        .expect("inner type wasn't added");
    database
        .get_type_by_name("[[u8; 32]; 10]")
        .expect("inner type wasn't added");
    database
        .get_type_by_name("[[[u32; 100]; 32]; 10]")
        .expect("inner type wasn't added");

    println!("{:#?}", database);
}
