use crate::jtd::api_config::structure::ApiConfigDataDelivery;

impl Default for ApiConfigDataDelivery {
    fn default() -> Self {
        ApiConfigDataDelivery {
            collect_cost_per_kg: 1,
            delivery_cost_per_kg: 1,
        }
    }
}
