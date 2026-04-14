SELECT setval(
  pg_get_serial_sequence('tags', 'id'),
  COALESCE((SELECT MAX(id) FROM tags), 1)
);
