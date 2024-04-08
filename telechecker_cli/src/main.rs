use anyhow::{Context, Result};
use clap::Parser;
use dotenvy::dotenv;
use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};
use telechecker_lib::{UserRetriever, SESSION_FILE};
use tracing::{debug, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const API_ID_ENV: &str = "API_ID";
const API_HASH_ENV: &str = "API_HASH";
const PHONE_NUMBER_ENV: &str = "PHONE_NUMBER";
const DEFAULT_OUTPUT_FILE: &str = "results.json";

/// Whether the added contact should be kept as the user's contact after retrieval
#[cfg(debug_assertions)]
const PRESERVE_CONTACT_DEFAULT: bool = true;
#[cfg(not(debug_assertions))]
const PRESERVE_CONTACT_DEFAULT: bool = false;

/// Default verbosity level for tracing
#[cfg(debug_assertions)]
const DEFAULT_VERBOSITY: u8 = u8::MAX;
#[cfg(not(debug_assertions))]
const DEFAULT_VERBOSITY: u8 = u8::MIN;

/// Whether the JSON output should be printed to stdout
#[cfg(debug_assertions)]
const DEFAULT_PRINT: bool = true;
#[cfg(not(debug_assertions))]
const DEFAULT_PRINT: bool = false;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// User's phone number (associated with a Telegram account)
    #[arg(short = 'u', long, value_name = "USER_PHONE_NUMBER")]
    user_phone: Option<String>,

    #[clap(flatten)]
    input: Input,

    /// User's api id. If not provided, the API_ID must be found inside the `.env` file
    #[arg(long, value_name = API_ID_ENV)]
    api_id: Option<String>,

    /// User's api hash. If not provided, the API_HASH must be found inside the `.env` file
    #[arg(long, value_name = API_HASH_ENV)]
    api_hash: Option<String>,

    /// Output file name. Default: `results.json`
    #[arg(short = 'o', long, value_name = "OUTPUT_FILE")]
    output: Option<String>,

    /// Whether the provided input numbers should be preserved as contacts after info retrieval. Default: false
    #[arg(long, default_value_t = PRESERVE_CONTACT_DEFAULT)]
    preserve_contact: bool,

    /// Whether the JSON output should be printed to stdout. Default: false
    #[arg(long, short, default_value_t = DEFAULT_PRINT)]
    print: bool,

    /// Dry run: Logs the user in but does not retrieve any data (useful with the verbosity flag
    /// to see configuration parameters)
    #[arg(short, long, default_value_t = false)]
    dry_run: bool,

    /// Verbosity flag (counted): i.e. `-v`: little output -vvvv: lots of output
    #[arg(short, default_value_t = DEFAULT_VERBOSITY, action = clap::ArgAction::Count)]
    verbosity: u8,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
pub struct Input {
    /// Phone numbers to check (provided as file)
    #[arg(value_name = "PHONE_NUMBERS_FROM_FILE")]
    phone_numbers_file: Option<PathBuf>,

    /// Phone numbers to check (provided as arguments)
    #[arg(
        short = 'n',
        long = "phone-numbers",
        value_name = "PHONE_NUMBERS_ARGS",
        value_delimiter = ','
    )]
    phone_numbers_args: Option<Vec<String>>,
}

async fn prompt(message: &str) -> Result<String> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;

    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.read_line(&mut buf)?;
    Ok(buf)
}

async fn prompt_pass(hint: String) -> Result<String> {
    match rpassword::prompt_password(format!("Enter the password (hint: {}): ", hint))
        .context("rpassword input")
    {
        Ok(s) => Ok(s),
        Err(_) => anyhow::bail!("Unable to retrieve password"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set the directory of the process to the directory of the executable,
    // so that the `.env` will always be read from where the program is
    #[cfg(not(debug_assertions))]
    std::env::set_current_dir(
        std::env::current_exe()
            .context("couldn't get current_exe")?
            .parent()
            .context("couldn't find parent to exe")?,
    )
    .context("couldn't set current dir")?;

    let cli = Cli::parse();

    let verbosity = match cli.verbosity {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        _ => "trace",
    };

    #[cfg(windows)]
    let colors = false;
    #[cfg(not(windows))]
    let colors = true;

    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(colors).without_time())
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| format!("{}={}", env!("CARGO_PKG_NAME").replace('-', "_"), verbosity),
        )))
        .init();

    debug!("{cli:?}");

    let input = match (cli.input.phone_numbers_args, cli.input.phone_numbers_file) {
        (Some(v), _) => v,
        (_, Some(path)) => {
            let mut file = File::options().read(true).open(path)?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            buf.trim().lines().map(str::to_string).collect()
        }
        _ => anyhow::bail!("Must provide phone numbers"),
    };

    debug!("Input: {input:?}");

    if dotenv().is_err() {
        debug!("Unable to read from `.env` file");
    }

    let (api_id_env, api_hash_env) = (env::var(API_ID_ENV).ok(), env::var(API_HASH_ENV).ok());

    let (api_id, api_hash) = match (cli.api_id, cli.api_hash) {
        (Some(id), Some(hash)) => (Some(id), Some(hash)),
        (Some(id), None) => (Some(id), api_hash_env),
        (None, Some(hash)) => (api_id_env, Some(hash)),
        _ => (api_id_env, api_hash_env),
    };

    let (api_id, api_hash) = match (api_id, api_hash) {
        (Some(id), Some(hash)) => (id, hash),
        _ => {
            anyhow::bail!("User must provide both {API_ID_ENV} and {API_HASH_ENV}, either as command line arguments or within an `.env` file.")
        }
    };

    let provided_user_phone_number = cli
        .user_phone
        .map_or_else(|| env::var(PHONE_NUMBER_ENV).ok(), Some);

    debug!(
        "{API_ID_ENV}='{api_id}'\t{API_HASH_ENV}='{api_hash}'\t{PHONE_NUMBER_ENV}='{}'",
        provided_user_phone_number.as_deref().unwrap_or("None")
    );

    let api_id = api_id.parse::<i32>().context("parse API_ID as i32")?;

    info!("Connecting to Telegram...");

    let mut user_retriever = UserRetriever::new(api_id, api_hash, SESSION_FILE).await?;

    let phone_input_handler = || async {
        match provided_user_phone_number {
            Some(pn) => Ok(pn),
            None => prompt("Enter your phone number: ").await,
        }
    };
    let code_input_handler = || prompt("Enter the code you received: ");
    let password_input_handler = |hint: String| prompt_pass(hint);

    user_retriever
        .sign_in(
            phone_input_handler,
            code_input_handler,
            password_input_handler,
        )
        .await?;

    if cli.dry_run {
        info!("Dry run complete. Input numbers: {:?}", input);
        return Ok(());
    }

    let users: Vec<_> = user_retriever.get_users(input).await?.collect();

    let output_file = cli.output.as_deref().unwrap_or(DEFAULT_OUTPUT_FILE);
    let out = serde_json::to_string_pretty(&users).context("serde_json to_string")?;

    if !cli.preserve_contact {
        user_retriever.delete_contacts(users.as_slice()).await?;
    } else {
        debug!("Contacts preserved");
    }
    drop(user_retriever);

    if cli.print {
        println!("{out}");
    }

    info!("Writing output to '{output_file}'");
    let mut file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)?;

    file.write_all(out.as_bytes())?;
    info!("Results saved as '{output_file}'");

    Ok(())
}
