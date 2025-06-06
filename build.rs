fn main() {
    embed_resource::compile::<_, &str, Vec<&str>>("assets/app_icon.rc", Vec::new());
}
