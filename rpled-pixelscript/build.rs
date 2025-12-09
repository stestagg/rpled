pub fn main() {
    println!("cargo::rerun-if-changed=testprogs");
    println!("cargo::rerun-if-env-changed=BASE_TEST_DIR");
}
