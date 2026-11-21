use super::*;
use std::{fs, path::Path, time::{SystemTime, UNIX_EPOCH}};

struct TestTree(PathBuf);

impl TestTree {
    fn new() -> Self {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("aurea-gpu-discovery-{unique}"));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn write(&self, relative: &str, value: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, value).unwrap();
    }
}

impl Drop for TestTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn discovers_builtin_panel_and_boot_vga() {
    let tree = TestTree::new();
    tree.write("sys/class/drm/card0/dev", "226:0");
    tree.write("sys/class/drm/card0/device/boot_vga", "1");
    tree.write("sys/class/drm/card0-eDP-1/status", "connected");
    let metadata = discover_gpu_selection_metadata_from(
        Path::new("/dev/dri/card0"),
        &tree.0.join("sys"),
        &tree.0.join("udev"),
    ).unwrap();
    assert!(metadata.has_builtin_panel);
    assert!(metadata.is_boot_vga);
    assert!(!metadata.preferred_primary);
}

#[test]
fn discovers_udev_preferred_primary_tag() {
    let tree = TestTree::new();
    tree.write("sys/class/drm/card1/dev", "226:1");
    tree.write("udev/c226:1", "I:123\nG:mutter-device-preferred-primary\n");
    let metadata = discover_gpu_selection_metadata_from(
        Path::new("/dev/dri/card1"),
        &tree.0.join("sys"),
        &tree.0.join("udev"),
    ).unwrap();
    assert!(metadata.preferred_primary);
}

#[test]
fn missing_sysfs_hints_are_non_fatal() {
    let tree = TestTree::new();
    tree.write("sys/class/drm/card2/dev", "226:2");
    let metadata = discover_gpu_selection_metadata_from(
        Path::new("/dev/dri/card2"),
        &tree.0.join("sys"),
        &tree.0.join("udev"),
    ).unwrap();
    assert_eq!(metadata, GpuSelectionMetadata::default());
}

#[test]
fn rejects_non_card_device_paths() {
    let tree = TestTree::new();
    assert_eq!(
        discover_gpu_selection_metadata_from(
            Path::new("/dev/dri/renderD128"),
            &tree.0.join("sys"),
            &tree.0.join("udev"),
        ).unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
}
