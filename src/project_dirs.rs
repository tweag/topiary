use directories::ProjectDirs;

// TODO: unwrap?

lazy_static! {
    pub static ref TOPIARY_DIRS: ProjectDirs =
        ProjectDirs::from("com", "Tweag", "Topiary").unwrap();
}
