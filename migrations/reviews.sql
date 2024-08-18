CREATE TABLE reviews (
   review_id serial primary key,
   rating smallint not null,
   comments text,
   returned_on date default current_date, 
   by_id integer not null,
   recommendation_id integer not null
);
