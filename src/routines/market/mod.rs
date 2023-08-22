use chrono::{Duration, NaiveDateTime, Utc};

use crate::db::{get_pg_connection, Ppc};

use crate::crypto::hash_short_identity_string;
use crate::db::models::api_config::ApiConfig;
use crate::db::models::inventory::{Inventory, InventoryNoQuantity, TempInventoryVote};
use crate::db::models::maintenance::Maintenance;
use crate::db::models::stock_config::StockConfigRow;
use crate::db::models::trade_stats::TradeStatistic;

use crate::jtd::stock_config::{
    PriceThreshold, StockConfig, StockConfigPricing, StockConfigRestock, StockThreshold,
};
use crate::routines::system::utc_midnight;
use crate::structs::inventory::SILVER_ITEM;

use crate::traits::numerical::{CanRound, Percentage};
use bigdecimal::BigDecimal;
use diesel::connection::SimpleConnection;
use diesel::expression::dsl::any;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use fastrand::Rng;
use itertools::Itertools;
use rand::{thread_rng, Rng as randRng};
use std::collections::{HashMap, HashSet};

pub async fn perform_maintenance(api_config: &ApiConfig) {
    let start_time = Utc::now().naive_utc();
    info!("Starting Database Maintenance");
    let scheduled_time =
        utc_midnight() + Duration::seconds(api_config.config_data.maintenance.start_time as i64);

    let conn = &mut get_pg_connection();
    if update_maintenance_table(true, Some(scheduled_time), Some(start_time), conn).is_err() {
        warn!("Unable to update maintenance table! Another node might already be running maintenance.");
        return;
    }
    process_votes(conn, api_config);
    // Updates stock and pricing
    process_market_data(conn, api_config);
    update_maintenance_table(false, Some(scheduled_time), Some(start_time), conn)
        .expect("Unable to update maintenance table!");

    info!("Completed database maintenance")
}

fn update_maintenance_table(
    in_progress: bool,
    start_time: Option<NaiveDateTime>,
    execution_time: Option<NaiveDateTime>,
    conn: &Ppc,
) -> Result<(), ()> {
    use crate::db::schema::maintenance as m_schema;

    // Set the maintenance flag
    match conn
        .build_transaction()
        .serializable()
        .run::<_, diesel::result::Error, _>(|| {
            let node_name = hostname::get().unwrap().into_string().unwrap();
            let checksum = hash_short_identity_string(format!(
                "{}{}",
                start_time.unwrap_or(NaiveDateTime::from_timestamp(0, 0)),
                node_name
            ));
            if diesel::delete(m_schema::table).execute(conn).is_err() {
                return Err(diesel::result::Error::RollbackTransaction);
            }
            if diesel::insert_into(m_schema::table)
                .values(Maintenance {
                    checksum,
                    in_progress,
                    start_time,
                    execution_time,
                    node_name: Some(node_name),
                })
                .execute(conn)
                .is_err()
            {
                return Err(diesel::result::Error::RollbackTransaction);
            }
            Ok(())
        }) {
        Err(_) => Err(()),
        Ok(_) => Ok(()),
    }
}

fn get_stock_configuration(conn: &Ppc) -> StockConfig {
    use crate::db::schema::stock_config as sc;
    sc::table
        .order_by(sc::version.desc())
        .first::<StockConfigRow>(conn)
        .expect("Stock configuration is missing!")
        .config_data
}

#[derive(Default)]
pub struct TradeStatsPair {
    pub bought: i64,
    pub sold: i64,
}

fn get_stock_movements(conn: &Ppc, items: &Vec<Inventory>) -> HashMap<String, TradeStatsPair> {
    use crate::db::schema::trade_statistics as schema;
    let today = Utc::today().naive_utc();
    let item_codes: Vec<&String> = items.iter().map(|i| &i.item_code).collect();
    let mut stats = HashMap::<String, TradeStatsPair>::with_capacity(items.len());

    let results: Vec<TradeStatistic> = schema::table
        .filter(schema::item_code.eq(any(item_codes)))
        .filter(schema::date.eq(today))
        .load(conn)
        .expect("Unable to load stock stats batch");

    for stat in results {
        if stat.buy {
            stats
                .entry(stat.item_code.clone())
                .and_modify(|e| e.bought = stat.quantity)
                .or_insert(TradeStatsPair {
                    bought: stat.quantity,
                    sold: 0,
                });
        } else {
            stats
                .entry(stat.item_code.clone())
                .and_modify(|e| e.sold = stat.quantity)
                .or_insert(TradeStatsPair {
                    bought: 0,
                    sold: stat.quantity,
                });
        }
    }
    stats
}

pub fn update_item_price(
    config: PriceThreshold,
    mut item: Inventory,
    stock_stats: Option<TradeStatsPair>,
    _rng: &Rng,
) -> Inventory {
    let stock_stats = stock_stats.unwrap_or_default();

    // Sale price adjust
    if stock_stats.bought > 0 {
        // There was trade, we increase the price we sell at by a number of steps up to max
        let step = item.base_value.percent_fraction(config.selling.step_size);
        let units = stock_stats.bought / (config.selling.unit_to_stock_ratio as i64);
        item.sell_at = &item.base_value
            + (step * BigDecimal::from(units.clamp(0, config.selling.max_steps as i64)));
    } else {
        // We didn't sell any today, decrease the price a step down to min.
        let step = item.base_value.percent_fraction(config.selling.step_size);
        item.sell_at = &item.base_value - step;
    }
    // Buy price adjust
    if stock_stats.sold > 0 {
        // There was trade, we decrease the price we buy at
        let step = item.base_value.percent_fraction(config.buying.step_size);
        let units = stock_stats.sold / (config.buying.unit_to_stock_ratio as i64);
        item.buy_at = &item.base_value
            - (step * BigDecimal::from(units.clamp(0, config.buying.max_steps as i64)));
    } else {
        // We didn't buy any today, increase the price a step.
        let step = item.base_value.percent_fraction(config.selling.step_size);
        item.buy_at = &item.base_value + step;
    }

    // Clamp the values to make sure they don't go out of range,
    // It also fixes the values if the base value changes due to a new version.
    item.sell_at = item
        .sell_at
        .clamp(
            &item.base_value
                - item
                    .base_value
                    .percent_fraction(config.selling.max_price_decrease_pct),
            &item.base_value
                + item
                    .base_value
                    .percent_fraction(config.selling.max_price_increase_pct),
        )
        .round_2dp();

    item.buy_at = item
        .buy_at
        .clamp(
            &item.base_value
                - item
                    .base_value
                    .percent_fraction(config.buying.max_price_decrease_pct),
            &item.base_value
                + item
                    .base_value
                    .percent_fraction(config.buying.max_price_increase_pct),
        )
        .round_2dp();
    item
}

pub fn update_item_stock(config: StockThreshold, mut item: Inventory, rng: &Rng) -> Inventory {
    // If we have too many, remove a random percentage of them
    if item.quantity > config.max_quantity as i32 {
        let rand_size = rng.f32().clamp(0f32, config.randomness);
        let amount = item.quantity.percent_fraction(rand_size) as i32;
        item.quantity -= amount;
    } else if item.quantity == 0 {
        // Only restock if RNG says we can
        let value = rng.f32();
        if value < config.chance_to_restock {
            // Pick a random amount between min_quantity and max_restock
            let rand_size = rng.u32(config.min_quantity..config.max_restock) as i32;
            item.quantity = rand_size;
        }
    } else {
        // The item is within bounds, 'Jiggle' the quantity a bit
        let rand_size = rng.f32().clamp(0.01, config.randomness);
        item.quantity +=
            match item.quantity.percent_fraction(rand_size) * (if rng.bool() { -1 } else { 1 }) {
                0 => -1, // make sure that we change it by at least something
                v => v,
            };
    }

    // Make sure it never goes negative or over the max quantity
    item.quantity = item.quantity.clamp(0, config.max_quantity as i32);
    item
}

fn get_price_threshold(config: &StockConfigPricing, item: &Inventory) -> Option<PriceThreshold> {
    config
        .thresholds
        .iter()
        .find(|threshold| {
            (BigDecimal::from(threshold.price_start) <= item.base_value)
                && (item.base_value <= BigDecimal::from(threshold.price_end))
        })
        .cloned()
}

fn get_stock_threshold(config: &StockConfigRestock, item: &Inventory) -> Option<StockThreshold> {
    config
        .thresholds
        .iter()
        .find(|threshold| {
            (BigDecimal::from(threshold.price_start) <= item.base_value)
                && (item.base_value <= BigDecimal::from(threshold.price_end))
        })
        .cloned()
}

fn update_prices(
    config: &StockConfig,
    conn: &Ppc,
    mut items: Vec<Inventory>,
    rng: &Rng,
) -> Vec<Inventory> {
    let item_count = items.len();

    let mut stock_stats = get_stock_movements(conn, &items);
    let mut processed_items = Vec::<Inventory>::with_capacity(items.len());
    for item in items.drain(..) {
        if let Some(threshold_config) = get_price_threshold(&config.pricing, &item) {
            let stock_stat = stock_stats.remove(&item.item_code);
            processed_items.push(update_item_price(threshold_config, item, stock_stat, rng));
        } else {
            // We can't update the price since we don't know what to do with it
            // Just push it onto the processed stack
            // warn!(
            //     "No pricing threshold found for item {} @ version {}",
            //     item.item_code, item.version
            // );
            processed_items.push(item);
        }
    }

    assert_eq!(processed_items.len(), item_count);

    processed_items
}

fn update_stock(
    config: &StockConfig,
    _conn: &Ppc,
    mut items: Vec<Inventory>,
    rng: &Rng,
) -> Vec<Inventory> {
    let item_count = items.len();

    let mut processed_items = Vec::<Inventory>::with_capacity(items.len());
    for item in items.drain(..) {
        if let Some(threshold_config) = get_stock_threshold(&config.restock, &item) {
            processed_items.push(update_item_stock(threshold_config, item, rng));
        } else {
            // We can't update the stock since we don't know what to do with it
            // Just push it onto the processed stack
            // warn!(
            //     "No stock threshold found for item {} @ version {}",
            //     item.item_code, item.version
            // );
            processed_items.push(item);
        }
    }

    assert_eq!(processed_items.len(), item_count);

    processed_items
}

fn process_market_batch(config: StockConfig, start_offset: i64, page_size: i64) {
    // This is run in a thread, we need a new connection
    use crate::db::schema::inventory as schema;
    let conn = &get_pg_connection();

    // Get the inventory
    let mut rows: Vec<Inventory> = schema::table
        .order(schema::item_code)
        .offset(start_offset)
        .limit(page_size)
        .get_results(conn)
        .expect("Failed to load inventory");

    if rows.len() == 0 {
        warn!("Nothing to process at offset {}", start_offset);
        return;
    }

    // Strip blacklisted items from the list of item codes
    rows = rows
        .drain(..)
        .filter(|item| item.item_code != &*SILVER_ITEM.item_code)
        .collect();

    let count = rows.len();
    let seed = thread_rng().gen::<u64>();
    debug!("Batch RNG seed is {}", seed);
    let rng = Rng::with_seed(seed);

    debug!("Updating prices");
    rows = update_prices(&config, conn, rows, &rng);
    debug!("Updating stock");
    rows = update_stock(&config, conn, rows, &rng);

    // Check we didn't lose any items during processing
    assert_eq!(rows.len(), count);

    diesel::insert_into(schema::table)
        .values(&rows)
        .on_conflict(schema::item_code)
        .do_update()
        .set((
            schema::buy_at.eq(excluded(schema::buy_at)),
            schema::sell_at.eq(excluded(schema::sell_at)),
            schema::quantity.eq(excluded(schema::quantity)),
        ))
        .execute(conn)
        .expect("Failed to update inventory table!");
}

fn process_votes(conn: &mut Ppc, config: &ApiConfig) {
    use crate::db::schema::inventory as inventory_schema;
    use crate::db::schema::new_inventory as new_inventory_schema;
    use crate::db::schema::new_inventory_vote_tracker as vote_tracker;

    use crate::db::views::temporary_vote_data as temp_votes;

    // Get the votes.
    conn.build_transaction()
        .repeatable_read()
        .run::<_, diesel::result::Error, _>(|| {
            // Delete any votes that relate to silver.
            // Stop abusers trying to overwrite the values.
            diesel::delete(vote_tracker::table)
                .filter(
                    vote_tracker::version.eq_any(
                        new_inventory_schema::table
                            .filter(new_inventory_schema::item_code.eq(&SILVER_ITEM.item_code))
                            .select(new_inventory_schema::version),
                    ),
                )
                .execute(conn)
                .expect("Failed to delete blacklisted votes");

            // Then delete any new inventory for silver
            diesel::delete(new_inventory_schema::table)
                .filter(new_inventory_schema::item_code.eq(&SILVER_ITEM.item_code))
                .execute(conn)
                .expect("Failed to delete blacklisted votes");

            // Refresh the existing silver item and ensure it exists.
            diesel::insert_into(inventory_schema::table)
                .values(&*SILVER_ITEM)
                .on_conflict(inventory_schema::item_code)
                .do_update()
                .set(&*SILVER_ITEM)
                .execute(conn)
                .expect("Failed to insert silver inventory item");

            // Create temp table to hold votes
            // We sort by votes ascending so that items with higher votes overwrite lower ones.
            let sql = format!(
                r#"DROP TABLE IF EXISTS "temporary_vote_data";
            DROP SEQUENCE IF EXISTS "temporary_vote_data_position_seq";
            CREATE TEMP TABLE "temporary_vote_data"( position bigserial primary key, item_code
            varchar(32) not null, thing_def varchar(200) not null, quality integer, minified boolean
            default false not null, base_value numeric(10, 2) default 0.0 not null,
            stuff varchar(200), weight numeric(10, 2) default 0.0 not null, version varchar(32) not
            null, votes integer) ON COMMIT DROP; INSERT INTO "temporary_vote_data"
            SELECT nextval('temporary_vote_data_position_seq'::regclass) as position, item_code,
            thing_def, quality, minified, base_value, stuff, weight, version, votes FROM
            summary_inventory_votes siv WHERE siv.votes > {}
            ORDER BY siv.votes ASC, siv.base_value ASC;"#,
                config.config_data.inventory.vote_promotion_threshold as i64
            );

            conn.batch_execute(&*sql)
                .expect("Unable to create vote temp data table!");

            let mut last_position: i64 = 0;
            loop {
                // We use the InventoryNoQuantity max batch size because
                // it's the larger of the two structs and we don't have to re-batch it again later.
                let mut voted_items: Vec<TempInventoryVote> = temp_votes::table
                    .filter(temp_votes::position.gt(last_position))
                    .order_by(temp_votes::position.asc())
                    .limit(InventoryNoQuantity::batch_size() as i64)
                    .load(conn)
                    .expect("Failed to read votes");

                // Insert items with vote > threshold to inventory and return ID's that were inserted.
                if voted_items.len() == 0 {
                    debug!("No more votes to process");
                    break;
                }

                // Get the last item and remember the position of it in the table.
                // Safe to unwrap because we checked the length of the vec above.
                last_position = voted_items.last().unwrap().position;

                debug!("Updated position to {}", last_position);
                debug!("Processing {} votes", voted_items.len());

                let mut voted_items: Vec<InventoryNoQuantity> = voted_items
                    .drain(..)
                    .map_into::<InventoryNoQuantity>()
                    .collect();

                let mut known_keys = HashSet::<String>::with_capacity(voted_items.len());
                let mut votes_to_delete = Vec::<String>::with_capacity(voted_items.len());

                // Since the table is sorted in ascending order by item code then by cost,
                // We iterate backwards to get the version of an item with the most votes,
                // if there are two versions with the same number of votes pick the one
                // that is the most expensive and discard all other versions of that item.
                let items_to_insert = voted_items
                    .drain(..)
                    .rev()
                    .map(|i| {
                        votes_to_delete.push(i.version.clone());
                        i
                    })
                    .filter(|item| known_keys.insert(item.item_code.clone()))
                    .collect::<Vec<InventoryNoQuantity>>();

                info!(
                    "Have {} votes to insert, {} filtered out",
                    items_to_insert.len(),
                    votes_to_delete.len() - items_to_insert.len()
                );

                drop(known_keys);

                // Upsert the new Inventory and update existing rows
                if let Err(e) = diesel::insert_into(inventory_schema::table)
                    .values(&items_to_insert)
                    .on_conflict(inventory_schema::item_code)
                    .do_update()
                    .set((
                        inventory_schema::version.eq(excluded(inventory_schema::version)),
                        inventory_schema::minified.eq(excluded(inventory_schema::minified)),
                        inventory_schema::base_value.eq(excluded(inventory_schema::base_value)),
                        inventory_schema::weight.eq(excluded(inventory_schema::weight)),
                    ))
                    .execute(conn)
                {
                    error!("Database error adding new inventory: {:?}", e);
                    return Err(diesel::result::Error::RollbackTransaction);
                };

                // Delete votes that were processed, we can't do this in the same block above
                // Because deleting results will affect our page position.
                // When the votes are deleted, delete the new inventory row too.
                debug!("{} items to clear", votes_to_delete.len());

                if let Err(e) = diesel::delete(vote_tracker::table)
                    .filter(vote_tracker::version.eq(any(&votes_to_delete)))
                    .execute(conn)
                {
                    error!("Database error deleting votes: {:?}", e);
                    return Err(diesel::result::Error::RollbackTransaction);
                };
                if let Err(e) = diesel::delete(new_inventory_schema::table)
                    .filter(new_inventory_schema::version.eq(any(votes_to_delete)))
                    .execute(conn)
                {
                    error!("Database error deleting new inventory: {:?}", e);
                    return Err(diesel::result::Error::RollbackTransaction);
                };
            }

            // Calculate the cut-off point for items that haven't been added within X number of days
            let vote_age_limit = utc_midnight()
                - Duration::days(config.config_data.inventory.vote_age_threshold as i64);

            loop {
                // Delete all items that haven't been voted in after a period.
                if let Ok(batch) = new_inventory_schema::table
                    .filter(new_inventory_schema::date_added.lt(vote_age_limit))
                    .select(new_inventory_schema::version)
                    .limit(65_000)
                    .load::<String>(conn)
                {
                    if batch.len() == 0 {
                        break;
                    }

                    // Delete the votes first
                    if let Err(e) = diesel::delete(vote_tracker::table)
                        .filter(vote_tracker::version.eq_any(&batch))
                        .execute(conn)
                    {
                        error!("Database error deleting old votes: {:?}", e);
                        return Err(diesel::result::Error::RollbackTransaction);
                    };

                    // Then the inventory items
                    if let Err(e) = diesel::delete(new_inventory_schema::table)
                        .filter(new_inventory_schema::version.eq_any(batch))
                        .execute(conn)
                    {
                        error!("Database error deleting old votes: {:?}", e);
                        return Err(diesel::result::Error::RollbackTransaction);
                    };
                }
            }
            info!("Committing changes");
            Ok(())
        });
}

fn process_market_data(conn: &mut Ppc, _: &ApiConfig) {
    // Fetch inventory from DB, chunk it up and process it in parallel.
    use crate::db::schema::inventory as schema;

    let _page = 0;
    let stock_config = get_stock_configuration(conn);
    let thread_count = (stock_config.threading.parallelism as usize)
        .min(num_cpus::get()) // No more threads than logical cores
        .max(1); // Make sure there's at least 1 thread.
    let pool = threadpool::ThreadPool::new(thread_count);
    let page_size = (Inventory::batch_size() as i64).min(stock_config.threading.batch_size as i64);
    let mut num_batches = 0u32;
    let mut num_items = 0i64;
    info!(
        "Processing market data, running up to {} threads in parallel in pages of {}",
        thread_count, page_size
    );

    conn.build_transaction()
        .run::<_, diesel::result::Error, _>(|| {
            // Count how many items are in inventory table
            num_items = schema::table
                .count()
                .get_result(conn)
                .expect("Unable to count inventory");
            let mut current_offset = 0;
            while current_offset < num_items {
                // Take this offset of inventory, make a new copy of
                // the configuration and send it to a new thread
                // If the thread pool has no capacity, it will wait
                // here until it can queue it.
                let thread_config = stock_config.clone();
                pool.execute(move || {
                    // This closure is run on a separate thread
                    debug!(
                        "Processing market data batch starting at {}",
                        current_offset
                    );
                    process_market_batch(thread_config, current_offset, page_size);
                    debug!("Completed market data batch");
                });
                current_offset += page_size;
                num_batches += 1;
                debug!("Queued threads: {}", pool.queued_count());
                debug!("Active threads: {}", pool.active_count());
            }
            Ok(())
        });

    // Wait for all threads to complete.
    info!("Waiting on threads to finish");
    pool.join();
    info!(
        "All threads exited, {} batches processed, {} items total",
        num_batches, num_items
    );
    if pool.panic_count() > 0 {
        error!("{} of {} batches failed", pool.panic_count(), num_batches)
    }
}

fn process_statistics_rollup(conn: &Ppc) {
    let _date = Utc::today().naive_utc();
    conn.build_transaction()
        .run::<_, diesel::result::Error, _>(|| {
            // pass

            Ok(())
        });
}
