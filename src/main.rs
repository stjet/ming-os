fn main() {
  // read env variables that were set in build script
  let uefi_path = env!("UEFI_PATH");
  println!("{}", uefi_path);

  let mut cmd = std::process::Command::new("qemu-system-x86_64");
  cmd.arg("-bios").arg("OVMF.4m.fd");
  cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));
  //cmd.arg("-m").arg("1G");
  cmd.arg("-serial").arg("stdio");
  println!("{:?}", cmd);
  let mut child = cmd.spawn().unwrap();
  child.wait().unwrap();
}
