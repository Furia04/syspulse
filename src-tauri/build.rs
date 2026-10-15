fn main() {
    let llvm_bin = "C:\\Users\\furia\\AppData\\Local\\Microsoft\\WinGet\\Packages\\MartinStorsjo.LLVM-MinGW.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\\llvm-mingw-20260908-ucrt-x86_64\\bin";
    if let Ok(path) = std::env::var("PATH") {
        let clean_path: Vec<&str> = path
            .split(';')
            .filter(|p| !p.to_lowercase().contains("c:\\mingw"))
            .collect();
        let new_path = format!("{};{}", llvm_bin, clean_path.join(";"));
        unsafe {
            std::env::set_var("PATH", new_path);
            std::env::set_var("RC", "x86_64-w64-mingw32-windres");
            std::env::set_var("RC_x86_64_pc_windows_gnullvm", "x86_64-w64-mingw32-windres");
        }
    }
    tauri_build::build()
}
