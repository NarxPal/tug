use crate::pull_and_extract_image;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tugfile::instruction::Instruction;

pub struct BuildContext {
    pub rootfs: PathBuf,
    pub workdir: PathBuf,
}

pub fn build_from_instructions(instructions: Vec<Instruction>) {
    let mut context: Option<BuildContext> = None;
    for inst in instructions {
        match inst {
            Instruction::From(image) => {
                println!("Pulling base image: {}", image);

                let ctx_path = PathBuf::from("/var/lib/tug/");

                if ctx_path.exists() {
                    fs::remove_dir_all(&ctx_path).unwrap();
                }

                let build_id = uuid::Uuid::new_v4().to_string();
                let ctx_path = PathBuf::from(format!("/var/lib/tug/{}", build_id));

                fs::create_dir_all(&ctx_path).unwrap();

                pull_and_extract_image(&image, &ctx_path);

                context = Some(BuildContext {
                    rootfs: ctx_path.clone(),
                    workdir: ctx_path.clone(),
                });
                if let Some(ctx) = &context {
                    println!("Final context path {}", ctx.rootfs.display());
                };
            }
            Instruction::Run(cmd) => {
                println!("run {}", cmd);
                if let Some(ctx) = &context {
                    Command::new("sh")
                        .arg("-c")
                        .arg(&cmd)
                        .current_dir(&ctx.workdir)
                        .output()
                        .expect("failed to execute command");
                }
            }
            Instruction::Copy { src, dest } => {
                println!("Copying from {} to {}", src, dest);

                if let Some(ctx) = &context {
                    let src_path = PathBuf::from(&src);
                    let dest_path = ctx.workdir.join(&dest);

                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent).unwrap();
                    }
                    fs::copy(&src_path, &dest_path).unwrap();
                    println!("Copied {} → {}", src_path.display(), dest_path.display());
                }
            }
            Instruction::Cmd(cmd) => {
                println!("cmd {}", cmd);

                if let Some(ctx) = &context {
                    let cmd_path = ctx.rootfs.join("cmd.txt");
                    fs::write(&cmd_path, format!("CMD: {}", cmd)).unwrap();
                    println!("Saved CMD to: {}", cmd_path.display());
                } else {
                    println!("CMD used before FROM — no context initialized");
                }
            }
            Instruction::Workdir(dir) => {
                println!("Setting workdir: {}", dir);
                // TODO: set context path

                if let Some(ctx) = &mut context {
                    let new_path = ctx.workdir.join(&dir);
                    fs::create_dir_all(&new_path).unwrap();
                    ctx.workdir = new_path;
                    println!("Set WORKDIR to: {}", ctx.workdir.display());
                } else {
                    println!("WORKDIR used before FROM — no context initialized");
                }
            }
            Instruction::Expose(port) => {
                print!("expost {}", port)
            }
            Instruction::Env { key, value } => {
                println!("env key {} value {}", key, value)
            }
            Instruction::EntryPoint(entrypoint) => {
                println!("entry point, {}", entrypoint)
            }
            Instruction::Add { src, dest } => {
                println!("src  {}, des , {} ", src, dest)
            }
        }
    }
}
