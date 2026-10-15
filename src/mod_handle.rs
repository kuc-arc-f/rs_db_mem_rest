use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use rusqlite::{params, Connection};
use rusqlite::{types::ValueRef};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::fs;
use std::io;
use serde_json::{json, Map, Value};
use uuid::Uuid;

pub async fn select_handle(
    State(state): State<super::AppState>,
    Json(payload): Json<super::UpdataReq>,
) -> (StatusCode, String) {
    let conn = state.lock().unwrap();
    
    let table_name = &payload.table;
    let sql = &payload.sql;
    // ---- 1. テーブルの列情報 (名前, 型) を取得 ----
    // PRAGMA table_info で列名・型を取得
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name)).unwrap();
    let columns: Vec<(String, String)> = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?; // cid=0, name=1, type=2, ...
            let col_type: String = row.get(2)?;
            Ok((name, col_type))
        }).expect("error, stmt.query_map")
        .collect::<rusqlite::Result<Vec<_>>>().unwrap();

    if columns.is_empty() {
        eprintln!("テーブル '{}' が存在しないか、列がありません。", table_name);
        //return Ok(());
        return (StatusCode::CREATED, "[]".to_string())
    }
    // スキーマ情報を JSON にまとめる
    let schema: Vec<Value> = columns
        .iter()
        .map(|(name, ty)| json!({ "name": name, "type": ty }))
        .collect();    
    // ---- 2. 全行を取得 ----
    //let sql = format!("SELECT * FROM {}", table_name);
    let mut stmt = conn.prepare(&sql).unwrap();
    let col_count = stmt.column_count();

    let mut rows_json: Vec<Value> = Vec::new();

    let mut rows = stmt.query([]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        let mut obj = Map::new();
        for i in 0..col_count {
            let col_name = columns[i].0.clone();
            let value = row.get_ref(i).unwrap();

            // ValueRef を serde_json::Value に変換
            let json_val = match value {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(n) => json!(n),
                ValueRef::Real(f) => json!(f),
                ValueRef::Text(t) => {
                    json!(String::from_utf8_lossy(t).to_string())
                }
                ValueRef::Blob(b) => {
                    // Blob は Base64 エンコードせず、バイト配列として出力
                    json!(b)
                }
            };
            obj.insert(col_name, json_val);
        }
        rows_json.push(Value::Object(obj));
    }        

    let output = json!({
        "table": table_name,
        "schema": schema,
        "rows": rows_json,
    });

    //println!("{}", serde_json::to_string_pretty(&output).unwrap());
    let json_str: String = serde_json::to_string_pretty(&output).unwrap();
    println!("{}", &json_str);    
    
    (StatusCode::CREATED, json_str.to_string())
}

pub async fn user_create(
    State(state): State<super::AppState>,
    Json(payload): Json<super::mod_model::UserCreateReq>,
) -> (StatusCode, Json<super::mod_model::UserRes>) {
    let mut user = super::mod_model::UserRes {
        email: "".to_string(),
        password: "".to_string(),
    };

    let conn = state.lock().unwrap();
    println!("{:?}", payload);

    let mut stmt = conn
        .prepare("SELECT id, email FROM users WHERE email = ?1")
        .unwrap();    
    let mut rows = stmt.query(params![payload.email]).unwrap();
    let mut user_ct = 0;
    while let Some(row) = rows.next().unwrap() {
        user_ct += 1;
    }
    println!("user_ct={}", user_ct);
    if user_ct > 0 {
        return (StatusCode::BAD_REQUEST , Json(user));
    }
    let pw = payload.password.clone();

    // get-hash
    let phc = super::mod_auth::hash_password(&pw).expect("error , hash_password");
    println!("phc: {}", phc);

    let ok = super::mod_auth::verify_password(&pw, &phc).expect("error , veryfy password");
    println!("password ok? {}", ok);    
    conn.execute(
        "INSERT INTO users (email, password_hash) VALUES (?1, ?2)",
        params![payload.email, phc],
    )
    .unwrap();
    let id = conn.last_insert_rowid();
    let sql: String = format!("INSERT INTO users (email, password_hash) VALUES ('{0}', '{1}')",
        &payload.email, &phc
    );
    println!("sql={}", sql);
    let mut array = super::GLOBAL_ARRAY.lock().unwrap();
    array.push(sql);
    
    (StatusCode::CREATED, Json(user))
}


pub async fn user_login(
    State(state): State<super::AppState>,
    Json(payload): Json<super::mod_model::UserCreateReq>,
) -> (StatusCode, Json<super::mod_model::UserRes>) {    
    let user = super::mod_model::UserRes {
        email: "".to_string(),
        password: "".to_string(),
    };
    let conn = state.lock().unwrap();
    // SELECT-User
    let mut stmt = conn
        .prepare("SELECT id, email , password_hash FROM users WHERE email = ?1")
        .unwrap();    
    let mut rows = stmt.query(params![payload.email]).unwrap();
    let mut user_ct = 0;
    let mut db_pass = "".to_string();
    while let Some(row) = rows.next().unwrap() {
        user_ct += 1;
        db_pass = row.get(2).unwrap();
    }
    println!("user_ct={}", user_ct);
    println!("db_pass={}", &db_pass);
    if user_ct == 0 {
        return (StatusCode::BAD_REQUEST , Json(user));
    }    

    println!("{:?}", payload);
    let pw = payload.password.clone();

    let phc = "".to_string();
    let phc = db_pass.clone();
    println!("phc: {}", phc);

    let ok = super::mod_auth::verify_password(&pw, &phc).expect("error , veryfy password");
    println!("password ok? {}", ok);    
    if ok == false {
        return (StatusCode::BAD_REQUEST, Json(user));
    }

    (StatusCode::OK, Json(user))
}