use std::io;
use std::process::ExitCode;

use rs_arrow_ipc2sqlite::Config;

fn io_envkey2val(key: &'static str) -> impl FnMut() -> String {
    move || std::env::var(key).unwrap_or_default()
}

fn io_create_sql() -> impl FnMut() -> String {
    io_envkey2val("ENV_CREATE_SQL")
}

fn io_insert_sql() -> impl FnMut() -> String {
    io_envkey2val("ENV_INSERT_SQL")
}

fn io_path2sqlite() -> impl FnMut() -> String {
    io_envkey2val("ENV_SQLITE_DB_FILE_PATH")
}

fn io_config() -> impl FnMut() -> Config {
    || {
        let create_sql: String = io_create_sql()();
        let insert_sql: String = io_insert_sql()();
        let path2sqlite: String = io_path2sqlite()();
        Config {
            create_sql,
            insert_sql,
            path2sqlite,
        }
    }
}

fn io_print_chg(chg: usize) -> impl FnOnce() -> Result<(), io::Error> {
    move || {
        println!("changed: {chg}");
        Ok(())
    }
}

fn io_main() -> impl FnOnce() -> Result<(), io::Error> {
    || {
        let cfg: Config = io_config()();
        let chg: usize = cfg.stdin2ipc2sqlite()?;
        io_print_chg(chg)()
    }
}

fn sub() -> Result<(), io::Error> {
    io_main()()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");

        let cfg: Config = io_config()();

        let Config {
            create_sql,
            insert_sql,
            path2sqlite,
        } = cfg;

        eprintln!("ENV_CREATE_SQL: {create_sql}");
        eprintln!("ENV_INSERT_SQL: {insert_sql}");
        eprintln!("ENV_SQLITE_DB_FILE_PATH: {path2sqlite}");

        ExitCode::FAILURE
    })
}
