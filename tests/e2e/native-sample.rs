//! A real owned native test executable, with a bounded Linux analysis fixture.
pub fn owned_native(directory: &std::path::Path) -> std::path::PathBuf {
    let original = std::env::current_exe().expect("Locate the owned test executable");
    #[cfg(target_os = "linux")]
    {
        // DWARF can make a debug test executable exceed the production 64 MiB
        // import limit. Strip debug sections from a private copy, never the
        // running executable; do not relax the parser's input limit for tests.
        let sample = directory.join("owned-native-elf");
        std::fs::copy(&original, &sample).expect("Copy the owned Linux fixture");
        let result = std::process::Command::new("strip")
            .arg("--strip-debug")
            .arg(&sample)
            .output()
            .expect("Linux native fixture tests require binutils strip");
        assert!(
            result.status.success(),
            "Cannot strip debug sections from the owned fixture copy"
        );
        assert!(
            original.is_file(),
            "The original test executable must remain present"
        );
        sample
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = directory;
        original
    }
}
