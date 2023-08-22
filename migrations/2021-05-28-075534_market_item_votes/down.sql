drop view summary_inventory_votes;

drop table new_inventory_vote_tracker;

drop table new_inventory;

alter table inventory
    drop column version;
