use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::sync::{LazyLock};
use std::fs;
use std::io;
use std::thread;
use std::time::Duration;

mod mod_model;
mod mod_handle;

const BACKUP_DB_PATH: &str = "./data/backup.db";
const BACKUP_SQL_PATH: &str = "./data/backup.sql";

static GLOBAL_ARRAY: LazyLock<Mutex<Vec<String>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdataReq {
    pub table: String,
    pub sql: String,
}

// アプリケーションの状態
type AppState = Arc<Mutex<Connection>>;

fn add_backup(sql: String) -> anyhow::Result<()> 
{
    let ret: bool = false;
    let conn = Connection::open(BACKUP_DB_PATH).expect("Failed to open database");
    conn.execute(
        &sql,
        [],
    )?;

    return Ok(());
}

#[tokio::main]
async fn main() {
    let sql_text: String = fs::read_to_string(BACKUP_SQL_PATH).unwrap();
    println!("sql_text:\n{}", sql_text);

    // SQLiteデータベースの初期化
    let conn = Connection::open_in_memory().expect("Failed to open database");
    
    // init-sql memDB
    conn.execute_batch(&sql_text).unwrap();
    // 5. 件数を確認
    let todoCount: i64 = conn.query_row(
        "SELECT COUNT(*) FROM temp",
        [],
        |row| row.get(0),
    ).unwrap();
    println!("登録件数: {}", todoCount);
    
    let state = Arc::new(Mutex::new(conn));

    // ルーティングの設定
    let app = Router::new()
        .route("/api/update", post(update))
        .route("/api/select", post(mod_handle::select_handle))
        .with_state(state);

    // thread - backup
    let mut t_count :i32 = 0;
    let handle = thread::spawn(move|| {
        while true  {
            //println!("スレッド内: {}" , t_count);
            t_count += 1;
            {
                // グローバル配列に追加
                let mut array = GLOBAL_ARRAY.lock().unwrap();
                //println!("配列の件数: {}", array.len());
                if array.len() == 0 {
                    println!("array.len() = 0");
                }else{
                    for item in array.iter() {
                        println!("{}", item); 
                        let resBackup = add_backup(item.clone());
                        match resBackup {
                            Ok(value) => {},
                            Err(err) => println!("error, add_backup: {}", err),
                        }                        
                    }
                    while array.len() > 0 {
                        array.remove(0);
                    }
                }

            }// <-- ここで変数 lock が, スコープを外れるため、自動的にロックが解除
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // サーバーの起動
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888")
        .await
        .expect("Failed to bind");
    
    println!("Server running on http://localhost:8888");
    
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

async fn update(
    State(state): State<AppState>, Json(payload): Json<UpdataReq>
) -> (StatusCode, String) {
    let conn = state.lock().unwrap();    
    conn.execute(
        &payload.sql,
        [],
    )
    .unwrap();
    // グローバル配列の内容を表示
    let mut array = GLOBAL_ARRAY.lock().unwrap();
    array.push(payload.sql);

    (StatusCode::CREATED, "OK".to_string())
}

