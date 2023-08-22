use std::collections::HashMap;
use std::convert::TryFrom;

use diesel::prelude::*;
use uuid::Uuid;

use crate::db::models::bank::BankBalance;
use crate::db::models::inventory::Inventory;
use crate::db::Ppc;
use crate::packets::common::CurrencyEnum;
use crate::packets::order::OrderItem;
use crate::structs::order::OrderStats;
use crate::traits::numerical::CanRound;
use bigdecimal::{BigDecimal, ToPrimitive};

pub fn get_bank_balance(colony_id: Uuid, currency: i32, conn: &Ppc) -> Result<BankBalance, ()> {
    use crate::db::schema::bank_balances as schema;

    // Check that it's a valid enum conversion
    if CurrencyEnum::try_from(currency).is_err() {
        return Err(());
    }

    match schema::table
        .filter(schema::colony_id.eq(colony_id))
        .filter(schema::currency.eq(currency))
        .get_result::<BankBalance>(conn)
    {
        Err(diesel::NotFound) => {
            match diesel::insert_into(schema::table)
                .values(BankBalance {
                    colony_id: colony_id.clone(),
                    currency,
                    balance: 0,
                })
                .get_result(conn)
            {
                Err(_) => Err(()),
                Ok(v) => Ok(v),
            }
        }
        Err(_) => Err(()),
        Ok(a) => Ok(a),
    }
}

pub fn update_bank(
    os: &OrderStats,
    db_inventory: &HashMap<String, Inventory>,
    additional_funds: i32,
    refund: Option<&Vec<OrderItem>>,
    bank_balance: &mut BankBalance,
    conn: &Ppc,
) -> Result<(i32, i32), ()> {
    // Add money we need to refund for missing items
    let mut refunded: BigDecimal = BigDecimal::default();
    let starting_balance = bank_balance.balance;

    if let Some(refund) = refund {
        for item in refund {
            // Add what ever we refunded to the account
            let item = db_inventory.get(&item.item_code).unwrap();
            refunded += (&item.sell_at * BigDecimal::from(item.quantity)).round_2dp();
        }
    }

    // Round then truncate, any partial amount or < 1, will be lost.
    let total_refund = refunded.round_2dp().to_i32().unwrap();
    bank_balance.balance += total_refund;

    // Add any funds they sent with the order
    bank_balance.balance += additional_funds;

    // Add what they sold to the balance
    bank_balance.balance += os.total_sell_cost.round_2dp().to_i32().unwrap();

    // Deduct expenditures
    bank_balance.balance -= os.total_buy_cost.round_2dp().to_i32().unwrap();

    if bank_balance.save_changes::<BankBalance>(&**conn).is_err() {
        Err(())
    } else {
        // Return the difference between the new and old balance
        Ok((total_refund, bank_balance.balance - starting_balance))
    }
}
