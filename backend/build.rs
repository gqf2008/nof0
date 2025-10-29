use std::fs;
use std::path::Path;

fn emit_rerun_if_changed(path: &Path) {
    if path.is_file() {
        println!("cargo:rerun-if-changed={}", path.display());
    } else if path.is_dir() {
        println!("cargo:rerun-if-changed={}", path.display());
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                emit_rerun_if_changed(&entry.path());
            }
        }
    }
}

fn main() {
    let dist = Path::new("../web/dist");
    if !dist.exists() {
        panic!("Missing frontend build artifacts. Run `npm run build` inside the web directory before building the backend.");
    }

    emit_rerun_if_changed(dist);

    // Windows 下 Release 模式隐藏控制台
    #[cfg(target_os = "windows")]
    {
        if std::env::var("PROFILE").unwrap_or_default() == "release" {
            let mut res = winresource::WindowsResource::new();
            res.set_manifest(
                r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <assemblyIdentity version="1.0.0.0" name="nof0-backend.exe" type="win32"/>
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
</assembly>
"#,
            );
            let _ = res.compile();
        }
    }
}
