# rs_db_mem_rest

 Version: 0.9.1

 date    : 2026/10/13
 
 update :

***

Rust Axum API , memory DB SQLite example

* sqlite3 use
* rustc 1.97.1

***
* table add
```
sqlite3 ./data/backup.db < table.sql
```

***
* build
```
cargo build --release
```

***
* start
```
./start.sh
```
***
* Test-code
* INSERT

```
curl -X POST http://localhost:8888/api/update \
  -H "Content-Type: application/json" \
  -d "{\"table\": \"temp\", \"sql\": \"INSERT INTO temp (title) VALUES ('title1');\"}"

```

* select
```
curl -X POST http://localhost:8888/api/select \
  -H "Content-Type: application/json" \
  -d '{"table": "temp", "sql": "SELECT * FROM temp;"}'

```

***
* delete
```
curl -X POST http://localhost:8888/api/update \
  -H "Content-Type: application/json" \
  -d '{"table": "temp", "sql": "DELETE FROM temp WHERE id = 1;"}'
```
***
### blog

https://zenn.dev/knaka0209/scraps/6e295e6a1318a6


***
