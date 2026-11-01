fn main() {
    println!("cargo:rerun-if-changed=protocols/infinity-window-manager-v1.xml");
    println!("cargo:rerun-if-changed=protocols/infinity-focus-v1.xml");
}
