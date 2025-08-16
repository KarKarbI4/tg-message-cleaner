use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use dialoguer::{Input, Password};
use dotenvy::dotenv;
use grammers_client::session::Session;
use grammers_client::{
    client::SignInError,
    types::{Chat, Dialog, PackedChat},
    Client, Config, InitParams,
};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;

/// Find and delete Telegram messages by keyword across your dialogs.
#[derive(Parser, Debug)]
#[command(name = "tg-message-cleaner")]
#[command(version, about = "Clean Telegram messages by keyword", long_about = None)]
struct CliArgs {
    /// Keyword to search for
    keyword: String,
}

#[derive(Serialize)]
struct DialogMessages {
    #[serde(flatten)]
    dialogs: BTreeMap<i64, Vec<i32>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let args: CliArgs = CliArgs::parse();

    let api_id: i32 = std::env::var("TG_API_ID")
        .context("TG_API_ID is required in environment")?
        .parse()
        .context("TG_API_ID must be an integer")?;
    let api_hash: String =
        std::env::var("TG_API_HASH").context("TG_API_HASH is required in environment")?;

    let mut client = connect_or_login(api_id, &api_hash).await?;

    // Fetch dialogs
    let mut dialogs_iter = client.iter_dialogs();
    let mut dialogs = Vec::new();
    while let Some(dialog) = dialogs_iter.next().await? {
        dialogs.push(dialog);
    }
    println!("Найдено {} диалогов", dialogs.len());

    // Build a map from dialog id to PackedChat for later deletions
    let mut dialog_id_to_packed: HashMap<i64, PackedChat> = HashMap::new();
    for d in &dialogs {
        dialog_id_to_packed.insert(d.chat().id(), d.chat().pack());
    }

    // 1) Remove my messages for everyone (revoke = true)
    find_and_clear_messages(
        &mut client,
        &args.keyword,
        &dialogs,
        MessageOwner::OnlyMine,
        true,
        &dialog_id_to_packed,
    )
    .await?;

    // 2) Remove others' messages for me (revoke = false) in user dialogs
    let user_dialogs: Vec<_> = dialogs
        .into_iter()
        .filter(|d| matches!(d.chat(), Chat::User(_)))
        .collect();
    find_and_clear_messages(
        &mut client,
        &args.keyword,
        &user_dialogs,
        MessageOwner::OnlyOthers,
        false,
        &dialog_id_to_packed,
    )
    .await?;

    Ok(())
}

async fn connect_or_login(api_id: i32, api_hash: &str) -> Result<Client> {
    let session_dir = PathBuf::from("sessionStorage");
    if !session_dir.exists() {
        fs::create_dir_all(&session_dir).context("failed to create sessionStorage directory")?;
    }

    let session_path = session_dir.join("session");
    let session = Session::load_file_or_create(session_path).context("failed to open session")?;

    let config = Config {
        session,
        api_id,
        api_hash: api_hash.to_string(),
        params: InitParams {
            // similar to connection retries in JS
            ..Default::default()
        },
    };

    let mut client = Client::connect(config).await.context("failed to connect")?;

    if !client.is_authorized().await? {
        println!("Starting login...");
        let phone: String = Input::new()
            .with_prompt("Please enter your number")
            .interact_text()
            .context("phone input failed")?;

        let token = client
            .request_login_code(&phone)
            .await
            .map_err(|e| anyhow!("failed requesting login code: {e}"))?;

        let code: String = Input::new()
            .with_prompt("Please enter the code you received")
            .interact_text()
            .context("code input failed")?;

        match client.sign_in(&token, &code).await {
            Ok(_user) => {
                println!("You should now be connected.");
            }
            Err(SignInError::PasswordRequired(password_token)) => {
                let password: String = Password::new()
                    .with_prompt("Please enter your password")
                    .interact()
                    .context("password input failed")?;
                client
                    .check_password(password_token, password)
                    .await
                    .context("password sign-in failed")?;
                println!("You should now be connected.");
            }
            Err(SignInError::InvalidCode) => return Err(anyhow!("Invalid code provided")),
            Err(e) => return Err(anyhow!("sign-in failed: {e}")),
        }
    }

    Ok(client)
}

#[derive(Copy, Clone)]
enum MessageOwner {
    OnlyMine,
    OnlyOthers,
}

async fn find_and_clear_messages(
    client: &mut Client,
    keyword: &str,
    dialogs: &[Dialog],
    owner: MessageOwner,
    revoke: bool,
    dialog_id_to_packed: &HashMap<i64, PackedChat>,
) -> Result<()> {
    println!(
        "Finding messages in dialogs by keyword {}. owner: {:?}, revoke: {}.",
        keyword, owner as u8, revoke
    );

    let mut dialog_to_message_ids: BTreeMap<i64, Vec<i32>> = BTreeMap::new();

    let progress = create_progress_bar("Ищем в диалогах", dialogs.len() as u64);

    for dialog in dialogs {
        progress.inc(1);
        let chat = dialog.chat();
        let packed: PackedChat = chat.pack();

        let mut msgs = client.search_messages(packed).query(keyword);
        match owner {
            MessageOwner::OnlyMine => msgs = msgs.sent_by_self(),
            MessageOwner::OnlyOthers => msgs = msgs,
        }

        while let Some(message) = msgs.next().await? {
            let is_mine = message.outgoing();
            let should_keep = match owner {
                MessageOwner::OnlyMine => is_mine,
                MessageOwner::OnlyOthers => !is_mine,
            };
            if should_keep {
                let id = message.id();
                let entry = dialog_to_message_ids.entry(dialog.chat().id()).or_default();
                entry.push(id);
            }
        }
    }

    progress.finish_and_clear();

    let total_messages: usize = dialog_to_message_ids.values().map(|v| v.len()).sum();
    println!(
        "Found {} messages in {} dialogs",
        total_messages,
        dialog_to_message_ids.len()
    );

    write_tmp_json("messages.json", &dialog_to_message_ids).await?;

    println!("Deleting...");
    delete_messages(client, &dialog_to_message_ids, revoke, dialog_id_to_packed).await?;
    println!("Successfully deleted messages with keyword {}", keyword);

    Ok(())
}

async fn delete_messages(
    client: &mut Client,
    messages: &BTreeMap<i64, Vec<i32>>,
    revoke: bool,
    dialog_id_to_packed: &HashMap<i64, PackedChat>,
) -> Result<()> {
    let pb = create_progress_bar("Удаляем в диалогах", messages.len() as u64);

    for (dialog_id, ids) in messages {
        pb.inc(1);
        let Some(packed) = dialog_id_to_packed.get(dialog_id).copied() else {
            return Err(anyhow!("unknown dialog id {dialog_id}"));
        };
        client
            .delete_messages(packed, &ids[..])
            .await
            .with_context(|| format!("failed to delete in dialog {dialog_id}"))?;
    }

    pb.finish_and_clear();
    Ok(())
}

fn create_progress_bar(msg: &str, len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.set_message(msg.to_string());
    pb
}

async fn write_tmp_json<T: ?Sized + Serialize>(filename: &str, data: &T) -> Result<()> {
    let tmp_dir = PathBuf::from("tmp");
    if !tmp_dir.exists() {
        fs::create_dir_all(&tmp_dir).context("failed to create tmp directory")?;
    }
    let tmp_path = tmp_dir.join(filename);

    // Basic path traversal prevention: ensure path is inside tmp
    let canon_tmp = tmp_dir.canonicalize().unwrap_or(tmp_dir.clone());
    let canon_target = tmp_path.canonicalize().unwrap_or(tmp_path.clone());
    if !canon_target.starts_with(&canon_tmp) {
        return Err(anyhow!("Trying to write file not in tmp folder"));
    }

    let serialized = serde_json::to_vec_pretty(data)?;
    let mut file = async_fs::File::create(tmp_path).await?;
    file.write_all(&serialized).await?;
    Ok(())
}
