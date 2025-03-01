fn main() {
    // Set macOS deployment target to 10.15 (Catalina) or higher
    std::env::set_var("MACOSX_DEPLOYMENT_TARGET", "10.15");
    
    // Skip generating bindings for whisper-rs-sys if there are issues
    std::env::set_var("WHISPER_DONT_GENERATE_BINDINGS", "1");
    
    // Add CMake to the PATH
    if let Some(path) = std::env::var_os("PATH") {
        let mut paths = std::env::split_paths(&path).collect::<Vec<_>>();
        // Add the path to CMake
        paths.push("/opt/homebrew/Cellar/cmake/3.31.6/bin".into());
        let new_path = std::env::join_paths(paths).unwrap();
        std::env::set_var("PATH", new_path);
    }
    
    // Run tauri-build
    tauri_build::build()
}
