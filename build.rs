use copy_to_output::copy_to_output;
use std::env;
use std::io;
#[cfg(windows)]
use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        WindowsResource::new()
            .set_icon("resources/icon.ico")
            .compile()?;
    }

    // copy files to output folder to support launching program without running cargo
    println!("cargo:rerun-if-changed=resources/*");
    copy_to_output("resources", &env::var("PROFILE").unwrap()).expect("Could not copy");

    println!("cargo:rerun-if-changed=lib/SDL2.dll");
    copy_to_output("lib/SDL2.dll", &env::var("PROFILE").unwrap()).expect("Could not copy");

    println!("cargo:rerun-if-changed=lib/SDL2_mixer.dll");
    copy_to_output("lib/SDL2_mixer.dll", &env::var("PROFILE").unwrap()).expect("Could not copy");

    Ok(())
}
