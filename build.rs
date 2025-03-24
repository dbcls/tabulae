#[cfg(feature = "frontend")]
use std::process::Command;

fn main() {
    #[cfg(feature = "frontend")]
    {
        println!("cargo::rerun-if-changed=frontend");
        let mut install = Command::new("bun")
            .arg("install")
            .current_dir("./frontend")
            .spawn()
            .expect("failed to execute bun install");
        let ecode = install.wait().expect("failed to wait on bun install");
        assert!(ecode.success());

        let mut build = Command::new("bun")
            .args(["run", "build"])
            .current_dir("./frontend")
            .spawn()
            .expect("failed to execute bun run build");
        let ecode = build.wait().expect("failed to wait on bun run build");
        assert!(ecode.success());

        let mut tar = Command::new("tar")
            .args([
                "--create",
                "--owner=0",
                "--group=0",
                "--file=../../frontend.tar",
                ".",
            ])
            .current_dir("./frontend/dist")
            .spawn()
            .expect("failed to execute tar");
        let ecode = tar.wait().expect("failed to wait on tar");
        assert!(ecode.success());
    }
}
