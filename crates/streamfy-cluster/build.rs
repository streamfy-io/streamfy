use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("../..");
    let version_path = workspace_root.join("VERSION");
    let sys_chart = workspace_root.join("k8-util/helm/pkg_sys/streamfy-chart-sys.tgz");
    let app_chart = workspace_root.join("k8-util/helm/pkg_app/streamfy-chart-app.tgz");

    if version_path.exists() {
        println!("cargo:rerun-if-changed={}", version_path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", sys_chart.display());
    println!("cargo:rerun-if-changed={}", app_chart.display());
    // Also rebuild when chart sources change
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("k8-util/helm/streamfy-sys").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("k8-util/helm/streamfy-app").display()
    );

    // Package helm charts into k8-util/helm/pkg_{sys,app} for include_dir!
    let output = Command::new("make")
        .arg("install")
        .current_dir(manifest_dir)
        .output()
        .expect("failed to spawn `make install` to package helm charts");
    if !output.status.success() {
        panic!(
            "failed to package helm charts via `make install` in {}:\nstdout:\n{}\nstderr:\n{}",
            manifest_dir.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    if !sys_chart.exists() || !app_chart.exists() {
        panic!(
            "helm chart packages missing after packaging:\n  {}\n  {}",
            sys_chart.display(),
            app_chart.display()
        );
    }

    // Fetch current git hash to print version output
    let git_version_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("should run 'git rev-parse HEAD' to get git hash");
    let git_hash = String::from_utf8(git_version_output.stdout)
        .expect("should read 'git' stdout to find hash");
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
