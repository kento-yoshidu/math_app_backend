use actix_web::{App, HttpResponse, HttpServer, get, web};
use serde::Serialize;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::types::{RecordId, SurrealValue, ToSql};

/// DB から取得する生のレコード形
#[derive(Debug, SurrealValue)]
struct UnitRow {
    id: RecordId,
    title: String,
    description: Option<String>,
}

/// API レスポンス用の形
/// id は `unit:xxx` 形式の文字列にして返す
#[derive(Debug, Serialize)]
struct UnitResponse {
    id: String,
    title: String,
    description: Option<String>,
}

impl From<UnitRow> for UnitResponse {
    fn from(row: UnitRow) -> Self {
        Self {
            id: row.id.to_sql(),
            title: row.title,
            description: row.description,
        }
    }
}

/// DB接続の死活監視用
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    message: Option<String>,
}

#[get("/db-health")]
async fn db_health(db: web::Data<Surreal<Any>>) -> HttpResponse {
    match db.health().await {
        Ok(()) => HttpResponse::Ok().json(HealthResponse {
            status: "ok",
            message: None,
        }),
        Err(err) => HttpResponse::ServiceUnavailable().json(HealthResponse {
            status: "error",
            message: Some(err.to_string()),
        }),
    }
}

#[get("/api/units")]
async fn list_units(db: web::Data<Surreal<Any>>) -> HttpResponse {
    let mut res = match db.query("SELECT id, title, description FROM unit ORDER BY title;").await {
        Ok(res) => res,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    let rows: Vec<UnitRow> = match res.take(0) {
        Ok(rows) => rows,
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
    };

    let units: Vec<UnitResponse> = rows.into_iter().map(Into::into).collect();
    HttpResponse::Ok().json(units)
}

/// 環境変数を読む
/// 無ければローカル開発用のデフォルトにフォールバックする
fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// DB に接続し、認証・namespace/database の選択まで済ませる(接続だけが責務)。
///
/// 接続先は環境変数で切り替える(ローカルの `surreal start` サーバーと
/// 本番の Surreal Cloud の両方を同じコードで扱えるようにするため)
///
/// - SURREAL_ENDPOINT (例: "ws://127.0.0.1:8000" / "wss://xxx.surreal.cloud")
/// - SURREAL_NS / SURREAL_DB
/// - SURREAL_USER / SURREAL_PASS
async fn connect_db() -> surrealdb::Result<Surreal<Any>> {
    let endpoint = env_or("SURREAL_ENDPOINT", "ws://127.0.0.1:8000");
    let ns = env_or("SURREAL_NS", "math_app");
    let db_name = env_or("SURREAL_DB", "dev");
    let username = env_or("SURREAL_USER", "root");
    let password = env_or("SURREAL_PASS", "root");

    let db = surrealdb::engine::any::connect(endpoint).await?;
    db.signin(Root { username, password }).await?;
    db.use_ns(ns).use_db(db_name).await?;

    Ok(db)
}

/// スキーマ定義と動作確認用のシードデータ投入。
///
/// OVERWRITE / UPSERT にしてあるので毎回実行しても安全だが、本来は
/// 「毎起動で走らせる」ものと「1回きりの初期データ投入」は別物なので、
/// 実データを扱うようになったらここは切り離すこと(README/SPEC.md 未決事項 参照)。
async fn ensure_schema(db: &Surreal<Any>) -> surrealdb::Result<()> {
    db.query(
        r#"
        DEFINE TABLE OVERWRITE unit SCHEMALESS;
        UPSERT unit:pythagorean_parity SET
            title = '原始ピタゴラス数の偶奇性を証明する',
            description = 'a, b の偶奇を mod 4 の議論から導く';
        "#,
    )
    .await?
    .check()?;

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ローカル開発用。.env が無い場合(本番/Renderなど)は無視して環境変数をそのまま使う。
    dotenvy::dotenv().ok();

    let db = connect_db()
        .await
        .expect("failed to connect to SurrealDB (ローカルサーバーが起動していますか? scripts/start-db.bat)");

    ensure_schema(&db).await.expect("failed to ensure schema");

    let db = web::Data::new(db);

    // ローカルでも Render でも 0.0.0.0 以外を使う理由が無いのでハードコードする。
    let bind_port: u16 = env_or("PORT", "8080").parse().expect("PORT must be a number");

    HttpServer::new(move || App::new()
        .app_data(db.clone())
        .service(db_health)
        .service(list_units))
        .bind(("0.0.0.0", bind_port))?
        .run()
        .await
}
