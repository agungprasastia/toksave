mod common;

use common::setup;
use toksave::registry::RunOpts;
use toksave::tools::rtk::{asset_name, is_installed_but_unreachable, local_rtk_path, RtkTool};
use toksave::tools::Tool;
use toksave::util::paths::{ensure_dir, local_bin};

#[test]
fn asset_name_matches_platform() {
    let asset = asset_name();
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    assert_eq!(asset, Some("rtk-x86_64-pc-windows-msvc.zip"));
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    assert_eq!(asset, Some("rtk-x86_64-unknown-linux-musl.tar.gz"));
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64")
    )))]
    assert!(
        asset.is_none() || asset.unwrap().ends_with(".tar.gz") || asset.unwrap().ends_with(".zip")
    );
}

#[test]
fn installed_but_unreachable_detects_local_bin_only() {
    let _env = setup();
    assert!(!is_installed_but_unreachable()); // nothing installed
    let bin = local_bin();
    ensure_dir(&bin).unwrap();
    std::fs::write(local_rtk_path(), "fake").unwrap();
    assert!(is_installed_but_unreachable());
    std::fs::remove_file(local_rtk_path()).ok();
}

#[tokio::test]
async fn dry_run_install_returns_true_without_writing() {
    let _env = setup();
    let opts = RunOpts {
        dry_run: true,
        upgrade: false,
        verbose: false,
        yes: true,
    };
    assert!(RtkTool.install(&opts).await.unwrap());
    assert!(!local_rtk_path().exists());
}
