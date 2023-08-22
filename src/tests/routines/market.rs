use crate::db::models::inventory::Inventory;
use crate::jtd::stock_config::{PriceThreshold, StockThreshold};
use crate::routines::market::{update_item_price, update_item_stock, TradeStatsPair};
use crate::traits::numerical::Percentage;
use bigdecimal::BigDecimal;

use fastrand::Rng as fastRng;
use rand::{thread_rng, Rng};

fn get_rng() -> fastRng {
    let seed = thread_rng().gen::<u64>();
    fastRng::with_seed(seed)
}

fn create_inventory_item(quantity: i32, base_value: f32) -> Inventory {
    let base_value = BigDecimal::from(base_value);
    Inventory {
        item_code: "ABCD1234".to_string(),
        thing_def: "steel".to_string(),
        quality: None,
        quantity,
        minified: false,
        buy_at: base_value.clone(),
        sell_at: base_value.clone(),
        base_value,
        stuff: None,
        weight: BigDecimal::from(1.0),
        version: "EFGH5678".to_string(),
    }
}

fn create_config_10_pct_step_per_unit() -> PriceThreshold {
    let mut config = PriceThreshold::default();
    config.buying.step_size = 0.1f32;
    config.buying.max_steps = 2;
    config.buying.unit_to_stock_ratio = 1;
    config.buying.max_price_decrease_pct = 1f32;
    config.buying.max_price_increase_pct = 1f32;
    config.selling = config.buying.clone();
    config
}

#[test]
fn test_cheap_item_high_sales_update_item_price() {
    let config: PriceThreshold = create_config_10_pct_step_per_unit();

    let original_item: Inventory = create_inventory_item(100, 2.0f32);
    let item = original_item.clone();
    let stock_stats: Option<TradeStatsPair> = Some(TradeStatsPair {
        bought: 0,
        sold: 1000,
    });

    let item = update_item_price(config, item, stock_stats, &get_rng());
    assert!(item.buy_at < original_item.buy_at);
    assert_eq!(item.buy_at, BigDecimal::from(1.6f32));
    assert_eq!(item.sell_at, BigDecimal::from(1.80f32));
}

#[test]
fn test_cheap_item_high_sales_clamps_update_item_price() {
    let mut config: PriceThreshold = create_config_10_pct_step_per_unit();
    config.selling.max_steps = 1000;

    let original_item: Inventory = create_inventory_item(100, 2.0f32);
    let item = original_item.clone();
    let stock_stats: Option<TradeStatsPair> = Some(TradeStatsPair {
        bought: 0,
        sold: 1000,
    });

    let item = update_item_price(config, item, stock_stats, &get_rng());
    let config: PriceThreshold = create_config_10_pct_step_per_unit();
    assert_eq!(original_item.item_code, item.item_code);
    assert!(
        item.sell_at
            <= &item.base_value
                + &item
                    .base_value
                    .percent_fraction(config.selling.max_price_increase_pct)
    );
    assert!(item.buy_at < original_item.buy_at);
    assert_eq!(item.buy_at, BigDecimal::from(1.60f32));
}

#[test]
fn test_expensive_item_high_sales_update_item_price() {
    let config: PriceThreshold = create_config_10_pct_step_per_unit();

    let original_item: Inventory = create_inventory_item(100, 200.0f32);
    let item = original_item.clone();
    let stock_stats: Option<TradeStatsPair> = Some(TradeStatsPair {
        bought: 0,
        sold: 1000,
    });

    let item = update_item_price(config, item, stock_stats, &get_rng());
    assert_eq!(original_item.item_code, item.item_code);
    assert!(item.buy_at < original_item.buy_at);
    assert_eq!(item.buy_at, BigDecimal::from(160f32));
    assert_eq!(item.sell_at, BigDecimal::from(180f32));
}

#[test]
fn test_expensive_item_high_sales_clamps_update_item_price() {
    let mut config: PriceThreshold = create_config_10_pct_step_per_unit();
    config.selling.max_steps = 1000;

    let original_item: Inventory = create_inventory_item(100, 200.0f32);
    let item = original_item.clone();
    let stock_stats: Option<TradeStatsPair> = Some(TradeStatsPair {
        bought: 0,
        sold: 1000,
    });

    let item = update_item_price(config, item, stock_stats, &get_rng());
    let config: PriceThreshold = create_config_10_pct_step_per_unit();
    assert_eq!(original_item.item_code, item.item_code);
    assert!(
        item.sell_at
            <= &item.base_value
                + &item
                    .base_value
                    .percent_fraction(config.selling.max_price_increase_pct)
    );
    assert!(item.buy_at < original_item.buy_at);
    assert_eq!(item.buy_at, BigDecimal::from(160.0f32));
}

#[test]
fn test_cheap_item_high_buys_update_item_price() {
    let config: PriceThreshold = create_config_10_pct_step_per_unit();

    let original_item: Inventory = create_inventory_item(100, 2.0f32);
    let item = original_item.clone();
    let stock_stats: Option<TradeStatsPair> = Some(TradeStatsPair {
        bought: 1000,
        sold: 0,
    });

    let item = update_item_price(config, item, stock_stats, &get_rng());
    assert!(original_item.buy_at < item.buy_at);
    assert_eq!(item.sell_at, BigDecimal::from(2.40f32));
    assert_eq!(item.buy_at, BigDecimal::from(2.20f32));
}

#[test]
fn test_cheap_item_high_buys_clamps_update_item_price() {
    let mut config: PriceThreshold = create_config_10_pct_step_per_unit();
    config.buying.max_steps = 1000;
    config.selling.max_steps = 1000;

    let original_item: Inventory = create_inventory_item(100, 2.0f32);
    let item = original_item.clone();
    let stock_stats: Option<TradeStatsPair> = Some(TradeStatsPair {
        bought: 1000,
        sold: 0,
    });

    let item = update_item_price(config, item, stock_stats, &get_rng());
    let config: PriceThreshold = create_config_10_pct_step_per_unit();
    assert_eq!(original_item.item_code, item.item_code);
    assert!(
        item.sell_at
            <= &item.base_value
                + &item
                    .base_value
                    .percent_fraction(config.selling.max_price_increase_pct)
    );
    assert!(original_item.sell_at < item.sell_at);
    assert_eq!(item.sell_at, BigDecimal::from(4.0f32));
    assert_eq!(item.buy_at, BigDecimal::from(2.2f32));
}

#[test]
fn test_expensive_item_high_buys_update_item_price() {
    let config: PriceThreshold = create_config_10_pct_step_per_unit();

    let original_item: Inventory = create_inventory_item(100, 200.0f32);
    let item = original_item.clone();
    let stock_stats: Option<TradeStatsPair> = Some(TradeStatsPair {
        bought: 1000,
        sold: 0,
    });

    let item = update_item_price(config, item, stock_stats, &get_rng());
    assert_eq!(original_item.item_code, item.item_code);
    assert_eq!(item.sell_at, BigDecimal::from(240f32));
    assert_eq!(item.buy_at, BigDecimal::from(220f32));
}

#[test]
fn test_expensive_item_high_buys_clamps_update_item_price() {
    let mut config: PriceThreshold = create_config_10_pct_step_per_unit();
    config.buying.max_steps = 1000;
    config.selling.max_steps = 1000;

    let original_item: Inventory = create_inventory_item(100, 200.0f32);
    let item = original_item.clone();
    let stock_stats: Option<TradeStatsPair> = Some(TradeStatsPair {
        bought: 1000,
        sold: 0,
    });

    let item = update_item_price(config, item, stock_stats, &get_rng());
    let config: PriceThreshold = create_config_10_pct_step_per_unit();
    assert_eq!(original_item.item_code, item.item_code);
    assert!(
        item.sell_at
            <= &item.base_value
                + &item
                    .base_value
                    .percent_fraction(config.selling.max_price_increase_pct)
    );
    assert!(original_item.sell_at < item.sell_at);
    assert_eq!(item.sell_at, BigDecimal::from(400.0f32));
    assert_eq!(item.buy_at, BigDecimal::from(220f32));
}

fn generate_stock_config(max: u32, min: u32) -> StockThreshold {
    StockThreshold {
        chance_to_restock: 0.25,
        max_quantity: max,
        max_restock: max / 2u32,
        min_quantity: min,
        price_end: 0,
        price_start: 0,
        randomness: 0.2,
    }
}

#[test]
fn test_stock_too_many() {
    let max = 50;
    let config = generate_stock_config(max, 10);
    let item = update_item_stock(config, create_inventory_item(100, 2f32), &get_rng());
    assert!(item.quantity <= max as i32);
    assert!(item.quantity > 0);
}

#[test]
fn test_stock_not_enough() {
    let min = 1;
    let max = 50;
    let config = generate_stock_config(max, min);
    let max_restock = config.max_restock;
    let item = update_item_stock(config, create_inventory_item(0, 2f32), &get_rng());
    assert!(item.quantity <= max_restock as i32);
}

#[test]
fn test_stock_jiggle() {
    let config = generate_stock_config(50, 10);
    let og_qty = 25;
    let item = update_item_stock(config, create_inventory_item(og_qty, 2f32), &get_rng());
    assert_ne!(item.quantity, og_qty);
    assert!(item.quantity >= 0);
}

#[test]
fn test_stock_too_many_large_quantities() {
    let max = 5_000_000;
    let config = generate_stock_config(max, 10_000);
    let item = update_item_stock(config, create_inventory_item(100, 2f32), &get_rng());
    assert!(item.quantity <= max as i32);
    assert!(item.quantity > 0);
}

#[test]
fn test_stock_not_enough_large_quantities() {
    let min = 10_000;
    let max = 5_000_000;
    let config = generate_stock_config(max, min);
    let item = update_item_stock(config, create_inventory_item(0, 2f32), &get_rng());
    assert!(item.quantity <= max as i32);
}

#[test]
fn test_stock_jiggle_large_quantities() {
    let config = generate_stock_config(5_000_000, 10_000);
    let og_qty = 2_500_000;
    let item = update_item_stock(config, create_inventory_item(og_qty, 2f32), &get_rng());
    assert_ne!(item.quantity, og_qty);
    assert!(item.quantity > 0);
}
