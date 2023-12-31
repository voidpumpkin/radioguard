CREATE TABLE run(
   id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
   name TEXT NOT NULL,
-- RFC 3339
   created_at TEXT NOT NULL,
   UNIQUE(name)
);

CREATE TABLE test_case(
   id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
   run_id INTEGER NOT NULL,
   name TEXT NOT NULL,
-- json [[number, number], [number, number]][]
   ignore_areas TEXT NOT NULL DEFAULT '[]',
-- RFC 3339
   created_at TEXT NOT NULL,
   FOREIGN KEY(run_id) REFERENCES run(id),
   UNIQUE(name, run_id)
);

CREATE TABLE step(
   id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
   data_uri TEXT NOT NULL,
   test_case_id INTEGER NOT NULL,
   parent_step_id INTEGER,
   name TEXT NOT NULL,
-- RFC 3339
   created_at TEXT NOT NULL,
   FOREIGN KEY(test_case_id) REFERENCES test_case(id),
   FOREIGN KEY(parent_step_id) REFERENCES step(id),
   UNIQUE(name, test_case_id)
);

CREATE TABLE tag(
   id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
   value TEXT NOT NULL,
   UNIQUE(value)
);

CREATE TABLE step_tag(
   id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
   step_id INTEGER NOT NULL,
   tag_id INTEGER NOT NULL,
   FOREIGN KEY(step_id) REFERENCES step(id),
   FOREIGN KEY(tag_id) REFERENCES tag(id),
   UNIQUE(step_id, tag_id)
);

CREATE TABLE test_case_tag(
   id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
   test_case_id INTEGER NOT NULL,
   tag_id INTEGER NOT NULL,
   FOREIGN KEY(test_case_id) REFERENCES test_case(id),
   FOREIGN KEY(tag_id) REFERENCES tag(id),
   UNIQUE(test_case_id, tag_id)
);

CREATE TABLE run_tag(
   id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
   run_id INTEGER NOT NULL,
   tag_id INTEGER NOT NULL,
   FOREIGN KEY(run_id) REFERENCES run(id),
   FOREIGN KEY(tag_id) REFERENCES tag(id),
   UNIQUE(run_id, tag_id)
);
