fn main() {
    #[cfg(all(any(target_os = "macos", target_os = "ios"), feature = "moltenvk"))]
    rosemetal_build::moltenvk::setup();
}
