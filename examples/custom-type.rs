use btf::{AddToBtf, BtfTypes};
use btf_derive::AddToBtf;

#[repr(C, align(1))]
#[derive(AddToBtf)]
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
    pub _array3d: [[[u32; 100]; 32]; 10],
}

fn main() {
    let mut btf = BtfTypes::default();
    CustomStructure::add_to_btf(&mut btf).expect("Failed to add CustomStructure to btf.");
    let custom_type = btf
        .resolve_type_by_name("CustomStructure")
        .expect("New type wasn't added.");

    btf.get_type_by_name("[u8; 10]")
        .expect("inner type wasn't added");
    btf.get_type_by_name("[[u8; 32]; 10]")
        .expect("inner type wasn't added");
    btf.get_type_by_name("[[[u32; 100]; 32]; 10]")
        .expect("inner type wasn't added");
    println!("{:#?}", custom_type);
}
