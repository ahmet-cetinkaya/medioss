use web_api::{
    bootstrap::{
        app_state::build_state,
        builtin_sounds::seed_builtin_soundscapes,
        ensure_data_dir,
        listener::bind_listener,
        logging::init_tracing,
        media_layout::migrate_media_layout,
        migrations::migrate_all,
        report::{fail, with_causes},
    },
    config::{self, Config},
    router::router,
};

mod operator;

#[derive(clap::Parser)]
#[command(name = "medioss", about = "MediOSS server and operator CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Reset an account password from the host, without a running server.
    #[command(name = "reset-password")]
    ResetPassword { email: String },
    /// Clear the login throttle for an identifier from the host.
    #[command(name = "clear-throttle")]
    ClearThrottle { identifier: String },
}

fn main() {
    // Local convenience: load ./.env if present. Real environment variables
    // always win (dotenv_override is off), so production env files cannot
    // shadow an operator's explicit settings. A missing file is fine.
    let _ = dotenvy::dotenv();
    let cli = <Cli as clap::Parser>::parse();
    // Environment mutation is only sound while the process is single-threaded,
    // so the server config is read and published before the runtime exists.
    let server_config = cli.command.is_none().then(|| {
        let config = config::from_env().unwrap_or_else(|err| {
            eprintln!("{}", with_causes(&err));
            std::process::exit(1)
        });
        config::publish_dev_origin(&config);
        config
    });
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|err| {
            eprintln!("could not start the async runtime: {err}");
            std::process::exit(1)
        })
        .block_on(run(cli, server_config));
}

async fn run(cli: Cli, server_config: Option<Config>) {
    match cli.command {
        Some(Commands::ResetPassword { email }) => {
            return operator::run_reset_password(&email).await;
        }
        Some(Commands::ClearThrottle { identifier }) => {
            return operator::run_clear_throttle(&identifier).await;
        }
        None => {}
    }
    let Some(config) = server_config else { return };

    init_tracing();
    config::warn_about_weakened_defaults(&config);
    config::log_upload_limits(&config);
    if let Err(err) = ensure_data_dir(&config.data_dir) {
        fail(with_causes(&err));
    }
    migrate_media_layout(&config.data_dir);

    let listener = bind_listener(config.bind_host, config.port).await;
    let database = web_api::bootstrap::database::open(&config.data_dir.join("medioss.db"))
        .await
        .unwrap_or_else(|err| fail(with_causes(&err)));
    migrate_all(&database).await;
    seed_builtin_soundscapes(&database, &config).await;

    let state = build_state(config, database).await;
    if let Err(err) = axum::serve(listener, router(state)).await {
        fail(format!("server error: {err}"));
    }
}
