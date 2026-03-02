#[cfg(feature = "binary")]
fn main() {
    use clap::Parser;
    use ouro::{config, Suite};
    use std::path::PathBuf;

    #[derive(Parser)]
    #[command(name = "ouro", about = "Golden test runner for language hackers")]
    struct Cli {
        /// Binary to test
        #[arg(long)]
        binary: Option<PathBuf>,

        /// Test file glob
        #[arg(long)]
        files: Option<String>,

        /// Comment prefix
        #[arg(long)]
        prefix: Option<String>,

        /// Overwrite expected output with actual
        #[arg(long, default_value_t = false)]
        update: bool,

        /// Parallel workers (default: num CPUs)
        #[arg(long)]
        jobs: Option<usize>,

        /// Path to ouro.toml (default: search upward from CWD)
        #[arg(long)]
        config: Option<PathBuf>,
    }

    let cli = Cli::parse();

    // Load base config from ouro.toml if present
    let base = {
        let config_path = cli.config.or_else(|| {
            let cwd = std::env::current_dir().unwrap_or_default();
            config::find_config_file(&cwd)
        });
        config_path.and_then(|p| config::Config::from_file(&p).ok())
    };

    // Build suite, overriding config with CLI flags
    let mut suite = Suite::new();

    if let Some(ref cfg) = base {
        suite = suite
            .binary(cfg.binary.clone())
            .files(cfg.files.clone())
            .prefix(cfg.prefix.clone());
        if let Some(j) = cfg.jobs {
            suite = suite.jobs(j);
        }
    }

    if let Some(b) = cli.binary {
        suite = suite.binary(b);
    }
    if let Some(f) = cli.files {
        suite = suite.files(f);
    }
    if let Some(p) = cli.prefix {
        suite = suite.prefix(p);
    }
    if let Some(j) = cli.jobs {
        suite = suite.jobs(j);
    }
    suite = suite.update(cli.update);

    match suite.run() {
        Ok(()) => std::process::exit(0),
        Err(()) => std::process::exit(1),
    }
}

#[cfg(not(feature = "binary"))]
fn main() {
    eprintln!("ouro: binary feature not enabled");
    std::process::exit(1);
}
