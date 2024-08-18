CREATE TABLE recommendations (
   recommendation_id serial primary key,
   type text not null,
   name text not null,
   artist text not null,
   recommended_on date default current_date,
   for_id integer not null,
   from_id integer not null
);
