//! `cargo run -p atabey-gen`: writes every schema into `schemas/`. Run through `pnpm gen`.

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let dir = atabey_gen::schemas_dir();
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    for (file, schema) in atabey_gen::schemas() {
        let path = dir.join(file);
        std::fs::write(&path, atabey_gen::render(&schema)?)
            .with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}
