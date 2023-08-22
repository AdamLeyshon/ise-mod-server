drop view summary_inventory_votes;

alter table new_inventory alter column base_value type numeric(10,2) using base_value::numeric(10,2);
alter table new_inventory alter column weight type numeric(10,2) using weight::numeric(10,2);

alter table inventory alter column base_value type numeric(10,2) using base_value::numeric(10,2);
alter table inventory alter column buy_at type numeric(10,2) using buy_at::numeric(10,2);
alter table inventory alter column sell_at type numeric(10,2) using sell_at::numeric(10,2);
alter table inventory alter column weight type numeric(10,2) using weight::numeric(10,2);

alter table price_tracker alter column value type numeric(10,2) using value::numeric(10,2);

create view summary_inventory_votes as
(
SELECT ni.item_code,
       ni.thing_def,
       ni.quality,
       ni.minified,
       ni.base_value,
       ni.stuff,
       ni.weight,
       ni.version,
       v.votes
FROM new_inventory ni
         LEFT JOIN (SELECT nivt.version,
                           count(nivt.version) AS votes
                    FROM new_inventory_vote_tracker nivt
                    GROUP BY nivt.version) v ON v.version::text = ni.version::text
         LEFT JOIN inventory i ON ni.version::text = i.version::text
WHERE i.version IS NULL
    );
