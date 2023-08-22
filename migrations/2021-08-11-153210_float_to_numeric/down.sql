drop view summary_inventory_votes;

alter table new_inventory alter column base_value type real using base_value::real;
alter table new_inventory alter column weight type real using weight::real;

alter table inventory alter column base_value type real using base_value::real;
alter table inventory alter column buy_at type real using buy_at::real;
alter table inventory alter column sell_at type real using sell_at::real;
alter table inventory alter column weight type real using weight::real;

alter table price_tracker alter column value type real using value::real;

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
         LEFT JOIN (
    SELECT nivt.version, count(nivt.version) as votes
    FROM new_inventory_vote_tracker nivt
    GROUP BY nivt.version
) as v on v.version = ni.version
         LEFT JOIN inventory i on ni.version = i.version
WHERE i.version IS NULL
    );
