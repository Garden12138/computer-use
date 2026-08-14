fn main() {
    #[cfg(windows)]
    windows_impl::main();
    #[cfg(not(windows))]
    {
        eprintln!("computer-use-windows builds the native helper only on Windows.");
        std::process::exit(2);
    }
}

#[cfg(windows)]
mod windows_impl;
