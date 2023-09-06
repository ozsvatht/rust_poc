use ctor::ctor;
use once_cell::sync::Lazy;
use std::time::Duration;
use testcontainers::{clients, core::Port, Container, Docker, Image};
use sqlx::postgres::PgPool;
use sqlx::postgres::*;
use sqlx::query;
use sqlx::{Connection, PgConnection};

static ENV: Lazy<TestEnvironment> = Lazy::new(|| TestEnvironment::new());

#[ctor]
fn before_all() {
    println!("This will run before all tests.");
}

#[derive(Debug, Clone)]
struct Postgres {
    pub username: String,
    pub password: String,
    pub db_name: String,
}

impl Default for Postgres {
    fn default() -> Self {
        Self {
            username: "postgres".into(),
            password: "postgres".into(),
            db_name: "postgres".into(),
        }
    }
}

impl Image for Postgres {
    type Args = Vec<String>;
    type EnvVars = Vec<(String, String)>;
    type Volumes = Vec<(String, String)>;
    type EntryPoint = std::convert::Infallible;

    fn descriptor(&self) -> String {
        "postgres:latest".to_string()
    }

    fn wait_until_ready<D: Docker>(&self, _container: &Container<'_, D, Self>) {}

    fn args(&self) -> <Self as Image>::Args {
        Vec::new()
    }

    fn env_vars(&self) -> Self::EnvVars {
        vec![
            ("POSTGRES_USER".into(), self.username.clone()),
            ("POSTGRES_PASSWORD".into(), self.password.clone()),
            ("POSTGRES_DB".into(), self.db_name.clone()),
        ]
    }

    fn volumes(&self) -> Self::Volumes {
        Vec::new()
    }

    fn with_args(self, _arguments: <Self as Image>::Args) -> Self {
        self
    }

    fn ports(&self) -> Option<Vec<Port>> {
        Some(vec![(5432, 5432).into()])
    }
}

struct TestEnvironment<'a> {
    docker: clients::Cli,
    _container: Option<Container<'a, clients::Cli, Postgres>>,
}

impl<'a> TestEnvironment<'a> {
    fn new() -> Self {
        Self {
            docker: clients::Cli::default(),
            _container: None,
        }
    }
}

async fn establish_connection(port: u16) -> PgPool {
    let database_url = format!(
        "postgres://{}:{}@localhost:{}/{}",
        "postgres", "postgres", port, "postgres"
    );

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create pool.");
    
    pool
}

#[tokio::test]
async fn test_sqlx_operations() {
    let docker = clients::Cli::default();
    let postgres = Postgres::default();

    let _container = docker.run(postgres);

    is_postgres_ready(5432, 30).await.expect("Failed to connect to Postgres after waiting");
    let pool = establish_connection(5432).await; // Assuming you're using the default PostgreSQL port

    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS your_table_name(
            id SERIAL PRIMARY KEY,
            your_column TEXT NOT NULL
        )
    "#)
    .execute(&pool)
    .await
    .expect("Failed to create table");

    // Insert operation
    sqlx::query("INSERT INTO your_table_name(your_column) VALUES ($1)")
        .bind("SomeValue")
        .execute(&pool)
        .await
        .expect("Failed to insert");

    // Select operation
    let result: (String,) = sqlx::query_as("SELECT your_column FROM your_table_name WHERE your_column = $1")
        .bind("SomeValue")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch");

    println!("result: {}", result.0);

    assert_eq!(result.0, "SomeValue");
}

async fn is_postgres_ready(port: u16, max_retries: u32) -> Result<(), Box<dyn std::error::Error>> {
    let connection_string = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
    for _ in 0..max_retries {
        if sqlx::PgConnection::connect(&connection_string).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err("Max retries reached while waiting for Postgres connection".into())
}
