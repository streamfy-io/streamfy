use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.join("../.."));
    let version_path = workspace_root.join("VERSION");
    let helm_dir = workspace_root.join("k8-util/helm");
    let sys_chart = helm_dir.join("pkg_sys/streamfy-chart-sys.tgz");
    let app_chart = helm_dir.join("pkg_app/streamfy-chart-app.tgz");

    if version_path.exists() {
        println!("cargo:rerun-if-changed={}", version_path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", sys_chart.display());
    println!("cargo:rerun-if-changed={}", app_chart.display());
    // Also rebuild when chart sources change
    println!(
        "cargo:rerun-if-changed={}",
        helm_dir.join("streamfy-sys").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        helm_dir.join("streamfy-app").display()
    );

    // Package helm charts into k8-util/helm/pkg_{sys,app} for include_dir!
    // Invoke the helm makefile directly so cwd and relative paths are correct.
    // Cargo injects MAKEFLAGS=-jN into build scripts; that races directory-based
    // packaging (and concurrent build.rs can stomp pkg_*). Force a serial,
    // isolated make; helm/Makefile `package` also flock-serializes.
    let output = Command::new("make")
        .arg("package")
        .arg("-j1")
        .current_dir(&helm_dir)
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .env_remove("MAKELEVEL")
        .env("MAKEFLAGS", "-j1")
        .output()
        .expect("failed to spawn `make package` to package helm charts");
    if !output.status.success() {
        panic!(
            "failed to package helm charts via `make package` in {}:\nstdout:\n{}\nstderr:\n{}",
            helm_dir.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    if !sys_chart.is_file() || !app_chart.is_file() {
        panic!(
            "helm chart packages missing after packaging:\n  {}\n  {}\nmake stdout:\n{}\nmake stderr:\n{}",
            sys_chart.display(),
            app_chart.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
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
