CREATE TABLE users (
   user_id serial primary key,
   username text unique not null,
   email text not null,
   password text not null
);
