use ctor::ctor;
use once_cell::sync::Lazy;
use std::time::Duration;
use testcontainers::{clients, core::Port, Container, Docker, Image};

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

#[tokio::test]
async fn test_run_postgres_container() {
    let docker = clients::Cli::default();
    let postgres = Postgres::default();

    // Run the postgres container
    // Once _container goes out of scope and is dropped, the container is stopped and removed.
    //make _container a global variable to be available for all tests
    let _container = docker.run(postgres);

    std::thread::sleep(Duration::from_secs(120));
}
