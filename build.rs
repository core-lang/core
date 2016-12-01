extern crate gcc;

fn main() {
    gcc::Config::new()
        .file("src/dwarfinfo.c")
        .include("/usr/include/libdwarf")
        .compile("libdwarfinfo.a");

    println!("cargo:rustc-link-lib=dwarf");
    println!("cargo:rustc-link-lib=elf");
}