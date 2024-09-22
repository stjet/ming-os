fn main() {
  // read env variables that were set in build script
  let bios_path = env!("BIOS_PATH");
  println!("{}", bios_path);
  let uefi_path = env!("UEFI_PATH");
  println!("{}", uefi_path);

  let bios = true;

  let mut cmd = std::process::Command::new("qemu-system-x86_64");
  if bios {
    cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));
    cmd.arg("-m").arg("32M");
  } else {
    cmd.arg("-bios").arg("OVMF.4m.fd");
    cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));
    cmd.arg("-m").arg("64M");
  }
  cmd.arg("-serial").arg("stdio");
  println!("{:?}", cmd);
  let mut child = cmd.spawn().unwrap();
  child.wait().unwrap();
}
