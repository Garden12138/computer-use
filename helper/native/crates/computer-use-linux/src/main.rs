fn main() {
    #[cfg(target_os = "linux")]
    linux_impl::main();
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("computer-use-linux builds the native helper only on Linux.");
        std::process::exit(2);
    }
}

#[cfg(target_os = "linux")]
mod linux_impl;
