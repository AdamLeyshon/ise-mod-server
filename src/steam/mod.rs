use steamid_ng::{AccountType, SteamID, Universe};

use crate::db::get_pg_connection;
use crate::db::models::blocked_steam_accounts::BlockedSteamAccount;
use crate::packets::bind::bind_reply::BindErrorReason;
use diesel::prelude::*;

pub trait ParsableSteamID {
    fn to_steam_id(&self) -> Option<SteamID>;
}

impl ParsableSteamID for Option<String> {
    fn to_steam_id(&self) -> Option<SteamID> {
        if self.is_some() {
            if let Ok(id) = parse_string_to_steam_id(&(self.as_ref().unwrap())) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl ParsableSteamID for String {
    fn to_steam_id(&self) -> Option<SteamID> {
        if !self.is_empty() {
            if let Ok(id) = parse_string_to_steam_id(&self) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub fn parse_string_to_steam_id(steam_id: &String) -> Result<SteamID, ()> {
    let steam_id_instance = {
        if let Ok(instance) = SteamID::from_steam3(steam_id.as_str()) {
            instance
        } else if let Ok(instance) = SteamID::from_steam2(steam_id.as_str()) {
            instance
        } else {
            let steam_u64 = steam_id.parse::<u64>().unwrap_or(0u64);
            SteamID::from(steam_u64)
        }
    };

    if steam_id_instance.account_type() != AccountType::Individual
        || steam_id_instance.universe() != Universe::Public
    {
        Err(())
    } else {
        Ok(steam_id_instance)
    }
}

pub fn get_steam_id_block_reason(steam_id: SteamID) -> BindErrorReason {
    let conn = get_pg_connection();
    use crate::db::schema::blocked_steam_accounts as schema;
    let query = schema::table
        .filter(schema::steam_id.eq(u64::from(steam_id).to_string()))
        .limit(1);

    let mut result: Vec<BlockedSteamAccount> = query
        .get_results(&conn)
        .expect("Error getting Blocked Steam accounts!");

    if let Some(block) = result.pop() {
        match block.reason {
            2 => BindErrorReason::SteamIdInvalid,
            3 => BindErrorReason::SteamIdBlocked,
            _ => BindErrorReason::Unknown,
        }
    } else {
        BindErrorReason::None
    }
}
