extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate log;

use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;

use actix_web::{middleware, web, App, HttpRequest, HttpServer, Responder};
use log::Level;
use tokio::spawn;

use deepfreeze::api;
use deepfreeze::config::load_config;
use deepfreeze::db::create_redis_connection_pool;
use deepfreeze::db::get_pg_connection;
use deepfreeze::db::models::api_config::ApiConfig;
use deepfreeze::db::RS_POOL;
use deepfreeze::decompress_payload::DecompressPayload;
use deepfreeze::jtd::api_config::structure::{ApiConfigData, ApiConfigDataApi};
use deepfreeze::request_helpers::ProtoBufConfig;
use deepfreeze::routines::system::poll_api_online_status;
use deepfreeze::structs::api_config::{ApiConfigStatus, API_CONFIG_ARC};
use deepfreeze::structs::general::SERVER_VERSION;

embed_migrations!("./migrations");

async fn root(_req: HttpRequest) -> impl Responder {
    format!("ISE API Server {}", SERVER_VERSION)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let settings = match load_config() {
        Ok(settings) => settings,
        Err(e) => {
            println!("Unable to parse settings: {}", e);
            exit(1);
        }
    };

    println!("Connecting to Redis...");
    RS_POOL
        .set(create_redis_connection_pool(&settings.redis.connection_string).await)
        .expect("Error creating Redis Pool");

    println!(
        "    _________ ______
   /  _/ ___// ____/
   / / \\__ \\/ __/
 _/ / ___/ / /___
/___//____/_____/ Version: {}",
        SERVER_VERSION
    );

    simple_logger::SimpleLogger::new()
        .with_utc_timestamps()
        .with_level(
            Level::from_str(&*settings.logging.level)
                .unwrap()
                .to_level_filter(),
        )
        .init()
        .expect("Unable to start logging!");

    info!("Applying pending migrations");
    if let Err(e) =
        embedded_migrations::run_with_output(&get_pg_connection(), &mut std::io::stdout())
    {
        error!("Failed to apply migration: {:?}", e);
        exit(254);
    }
    info!("Migration complete, starting...");

    let api_config = Some(ApiConfig {
        version: 0,
        config_data: ApiConfigData {
            api: ApiConfigDataApi {
                // Always start offline until we pickup config from server.
                force_offline: true,
            },
            delivery: Default::default(),
            inventory: Default::default(),
            maintenance: Default::default(),
        },
    });

    let mut lock = API_CONFIG_ARC.write();
    *lock = api_config;
    drop(lock);

    info!("Start monitoring API Status");
    let config_lock = Arc::clone(&API_CONFIG_ARC);
    spawn(poll_api_online_status(config_lock));

    let app_data = web::Data::new(settings);
    let app_state = web::Data::new(Arc::clone(&API_CONFIG_ARC));

    HttpServer::new(move || {
        let header_middleware = middleware::DefaultHeaders::new()
            .add(("x-api-server", "deep-freeze"))
            .add(("x-api-server-version", SERVER_VERSION.to_string()));

        App::new()
            // .data_factory(|| async {
            //     Ok::<LockedApiConfigWrapper, ()>(LockedApiConfigWrapper {
            //         data: Arc::clone(&API_CONFIG),
            //     })
            // })
            .app_data(app_data.clone())
            .app_data(app_state.clone())
            .app_data({
                // Default to 1MB payload size
                let mut pbc = ProtoBufConfig::default();
                pbc.limit(1_000_000);
                pbc
            })
            .wrap(middleware::Logger::new(
                "%a \"%r\" %s %b \"%{User-Agent}i\" \"%{x-ise-client-id}i\" %T",
            ))
            .wrap(DecompressPayload)
            .wrap(middleware::Compress::default())
            .wrap(ApiConfigStatus)
            .wrap(header_middleware)
            .configure(api::config)
            .route("/", web::get().to(root))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
