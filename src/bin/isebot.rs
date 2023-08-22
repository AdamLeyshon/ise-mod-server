use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use deepfreeze::config::{load_config, CONFIG};
use deepfreeze::db::get_pg_connection;
use deepfreeze::db::models::colony::Colony;
use diesel::dsl::{count, sum};
use diesel::prelude::*;
use diesel::QueryDsl;
use serenity::async_trait;
use serenity::client::{parse_token, Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use std::process::exit;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let settings = match load_config() {
        Ok(settings) => settings,
        Err(e) => {
            println!("Unable to parse settings: {}", e);
            exit(1);
        }
    };

    let settings = CONFIG.get_or_init(move || settings);

    simple_logger::SimpleLogger::new()
        .with_utc_timestamps()
        .with_level(
            log::Level::from_str(&*settings.logging.level)
                .unwrap()
                .to_level_filter(),
        )
        .init()
        .expect("Unable to start logging!");

    let token_data = parse_token(&settings.discord.auth_token).expect("Invalid auth token");

    let bot_user_id = token_data.bot_user_id;

    let framework = StandardFramework::new()
        .configure(|c| c.on_mention(Some(bot_user_id)).prefix(""))
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let mut client = Client::builder(&settings.discord.auth_token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[group]
#[commands(
    ping,
    get_colony_count,
    get_online_count,
    get_orders_today,
    get_orders_total,
    get_total_stock,
    get_online_colonies
)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command("colonycount")]
async fn get_colony_count(ctx: &Context, msg: &Message) -> CommandResult {
    let conn = &mut get_pg_connection();
    use deepfreeze::db::schema::colonies as schema;

    let c: i64 = schema::table
        .select(count(schema::colony_id))
        .first(conn)
        .expect("Failed to read colony table");

    msg.reply(ctx, format!("There are {} colonies in total", c))
        .await?;

    Ok(())
}

#[command("onlinecount")]
async fn get_online_count(ctx: &Context, msg: &Message) -> CommandResult {
    let conn = &mut get_pg_connection();
    use deepfreeze::db::schema::colonies as schema;

    let since = Utc::now().naive_utc() - Duration::minutes(30);

    let c: i64 = schema::table
        .select(count(schema::colony_id))
        .filter(schema::update_date.ge(since))
        .first(conn)
        .expect("Failed to read colony table");

    msg.reply(ctx, format!("There are {} colonies online now", c))
        .await?;

    Ok(())
}

#[command("whosonline")]
async fn get_online_colonies(ctx: &Context, msg: &Message) -> CommandResult {
    let conn = &mut get_pg_connection();
    use deepfreeze::db::schema::colonies as schema;

    let since = Utc::now().naive_utc() - Duration::minutes(30);

    let colonies: Vec<Colony> = schema::table
        .filter(schema::update_date.ge(since))
        .order(schema::update_date.desc())
        .limit(20)
        .get_results(conn)
        .expect("Failed to read colony table");

    let mut resp = String::new();
    resp.push_str("The top 20 most recently active colonies were:\n");
    for colony in colonies {
        resp.push_str(&*format!(
            "* {} from the {} faction\n",
            colony.name, colony.faction_name
        ))
    }

    msg.reply(ctx, resp).await?;

    Ok(())
}

#[command("orderstoday")]
async fn get_orders_today(ctx: &Context, msg: &Message) -> CommandResult {
    let conn = &mut get_pg_connection();
    use deepfreeze::db::schema::orders as schema;

    let now = NaiveDateTime::new(Utc::today().naive_utc(), NaiveTime::from_hms(0, 0, 0));

    let c: i64 = schema::table
        .select(count(schema::order_id))
        .filter(schema::create_date.ge(now))
        .first(conn)
        .expect("Failed to read order table");

    msg.reply(ctx, format!("There were {} orders since {}", c, now))
        .await?;

    Ok(())
}

#[command("totalorders")]
async fn get_orders_total(ctx: &Context, msg: &Message) -> CommandResult {
    let conn = &mut get_pg_connection();
    use deepfreeze::db::schema::orders as schema;

    let now = NaiveDateTime::new(Utc::today().naive_utc(), NaiveTime::from_hms(0, 0, 0));

    let c: i64 = schema::table
        .select(count(schema::order_id))
        .first(conn)
        .expect("Failed to read order table");

    msg.reply(ctx, format!("There were {} orders in total", c))
        .await?;

    Ok(())
}

#[command("totalstock")]
async fn get_total_stock(ctx: &Context, msg: &Message) -> CommandResult {
    let conn = &mut get_pg_connection();
    use deepfreeze::db::schema::inventory as schema;

    let types: i64 = schema::table
        .select(count(schema::item_code))
        .first(conn)
        .expect("Failed to read order table");

    let stock: i64 = schema::table
        .select(sum(schema::quantity))
        .first::<Option<i64>>(conn)
        .expect("Failed to read order table")
        .expect("No items in database?");

    msg.reply(
        ctx,
        format!(
            "There are {} types of items in the database and {} items in stock",
            types, stock
        ),
    )
    .await?;

    Ok(())
}
