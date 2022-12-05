use ad4m_client::Ad4mClient;
use anyhow::{Context, Result};
use clap::Subcommand;
use rustyline::Editor;
use std::fs;
use std::sync::mpsc::channel;

use crate::bootstrap_publish::*;
use crate::startup::executor_data_path;

#[derive(Debug, Subcommand)]
pub enum LanguageFunctions {
    /// List all languages
    All,
    /// Show information about a language
    ByAddress { address: String },
    /// List all languages that implement a given interface
    ByFilter { filter: String },
    /// Set language settings (JSON) string
    WriteSettings { address: String, settings: String },
    /// Clone a source language, set template parameters and publish the result
    ApplyTemplateAndPublish {
        /// Address of the source language to clone
        source: String,
        /// JSON string of template parameters
        template_data: String,
    },
    /// Publish AD4M Language from local file path
    Publish { path: String },
    /// Show meta information about a language
    Meta { address: String },
    /// Show source code of a language
    Source { address: String },
    /// Uninstall given language
    Remove { address: String },
    /// Generate bootstrap seed from a local prototype JSON file declaring languages to be published
    GenerateBootstrap {
        agent_path: String,
        passphrase: String,
        ad4m_host_path: String,
        seed_proto: String,
    },
}

pub async fn run(ad4m_client: Ad4mClient, command: Option<LanguageFunctions>) -> Result<()> {
    if command.is_none() {
        let all_languages = ad4m_client.languages.by_filter(None).await?;
        for language in all_languages {
            println!("\x1b[36mName: \x1b[97m{}", language.name);
            println!("\x1b[36mAddress: \x1b[97m{}", language.address);

            if let Some(settings) = language.settings {
                if settings != "{}" {
                    println!("\x1b[36mSettings: \x1b[97m{}", settings);
                } else {
                    println!("\x1b[36mSettings: \x1b[90m<empty>");
                }
            } else {
                println!("\x1b[36mSettings: \x1b[90m<undefined>");
            }
            println!()
        }
        return Ok(());
    }

    match command.unwrap() {
        LanguageFunctions::All => {
            let all_perspectives = ad4m_client.languages.by_filter(None).await?;
            println!("{:#?}", all_perspectives);
        }
        LanguageFunctions::ByFilter { filter } => {
            let languages = ad4m_client.languages.by_filter(Some(filter)).await?;
            println!("{:#?}", languages);
        }
        LanguageFunctions::ByAddress { address } => {
            let maybe_language = ad4m_client.languages.by_address(address).await?;
            if let Some(language) = maybe_language {
                println!("{:#?}", language);
            } else {
                println!("Language not found");
            }
        }
        LanguageFunctions::WriteSettings { address, settings } => {
            ad4m_client
                .languages
                .write_settings(address, settings)
                .await?;
            println!("Language settings written");
        }
        LanguageFunctions::ApplyTemplateAndPublish {
            source,
            template_data,
        } => {
            let new_language = ad4m_client
                .languages
                .apply_template_and_publish(source, template_data)
                .await?;
            println!("Language template applied and published!");
            println!("Name: {}", new_language.name);
            println!("Address: {}", new_language.address);
        }
        LanguageFunctions::Meta { address } => {
            let meta = ad4m_client.languages.meta(address).await?;
            println!("{:#?}", meta);
        }
        LanguageFunctions::Publish { path } => {
            let _ = std::fs::read_to_string(path.clone())
                .with_context(|| format!("Could not read language file `{}`!", path))?;

            println!("Publishing language found in file `{}`...", path);
            println!("Please enter meta-information for this language: ");
            let mut rl = Editor::<()>::new()?;
            let name = rl.readline("Name (should match name in code): ")?;
            let description = rl.readline("Description: ")?;
            let possible_template_params_string =
                rl.readline("Template parameters (comma spearated list): ")?;
            let possible_template_params: Vec<String> = possible_template_params_string
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let source_code_link = rl.readline("Source code link: ")?;

            let description = if description.is_empty() {
                None
            } else {
                Some(description)
            };
            let possible_template_params = if possible_template_params_string.is_empty() {
                None
            } else {
                Some(possible_template_params)
            };
            let source_code_link = if source_code_link.is_empty() {
                None
            } else {
                Some(source_code_link)
            };

            let publish_result = ad4m_client
                .languages
                .publish(
                    path,
                    name,
                    description,
                    possible_template_params,
                    source_code_link,
                )
                .await?;
            println!(
                "Language published with address: {}",
                publish_result.address
            );
        }
        LanguageFunctions::Source { address } => {
            let source = ad4m_client.languages.source(address).await?;
            println!("{}", source);
        }
        LanguageFunctions::Remove { address } => {
            ad4m_client.languages.remove(address).await?;
            println!("Language removed");
        }
        LanguageFunctions::GenerateBootstrap {
            agent_path,
            passphrase,
            ad4m_host_path,
            seed_proto,
        } => {
            println!(
                "Attempting to generate a new bootstrap seed using ad4m-host path: {:?} and agent path: {:?}\n",
                ad4m_host_path,
                agent_path
            );

            //Warn the user about deleting ad4m agent during publishing
            println!("WARNING... THIS WILL DELETE YOUR EXISTING AD4M AGENT AND REPLACE WITH SUPPLIED PUBLISHING AGENT, PLEASE BACKUP BEFORE PROCEEDING\n\n");
            let mut rl = Editor::<()>::new()?;
            let continue_response = rl.readline("y/n to continue...")?;
            if continue_response == String::from("n") || continue_response == String::from("N") {
                return Ok(());
            };

            //Load the seed proto first so we know that works before making new agent path
            let seed_proto = fs::read_to_string(seed_proto)?;
            let seed_proto: SeedProto = serde_json::from_str(&seed_proto)?;
            println!("Loaded seed prototype file!\n");

            //Create a new ~/.ad4m path with agent.json file supplied
            //Backup any existing ad4m agent to ~/.ad4m.old
            let data_path = executor_data_path();
            let data_path_files = std::fs::read_dir(&data_path);
            if data_path_files.is_ok() {
                fs::remove_dir_all(&data_path)?;
            }
            //Create the ad4m directory
            fs::create_dir(&data_path)?;
            let ad4m_data_path = data_path.join("ad4m");
            fs::create_dir(&ad4m_data_path)?;
            let data_data_path = data_path.join("data");
            fs::create_dir(&data_data_path)?;

            //Read the agent file
            let agent_file = fs::read_to_string(agent_path)?;
            //Copy the agent file to correct directory
            fs::write(ad4m_data_path.join("agent.json"), agent_file)?;
            fs::write(data_data_path.join("DIDCache.json"), String::from("{}"))?;
            println!("Publishing agent directory setup\n");

            println!("Creating temporary bootstrap seed for publishing purposes...\n");
            let lang_lang_source = fs::read_to_string(&seed_proto.language_language_ref)?;
            let temp_bootstrap_seed = BootstrapSeed {
                trusted_agents: vec![],
                known_link_languages: vec![],
                language_language_bundle: lang_lang_source.clone(),
                direct_message_language: String::from(""),
                agent_language: String::from(""),
                perspective_language: String::from(""),
                neighbourhood_language: String::from(""),
            };
            let temp_publish_bootstrap_path = data_path.join("publishing_bootstrap.json");
            fs::write(
                &temp_publish_bootstrap_path,
                serde_json::to_string(&temp_bootstrap_seed)?,
            )?;

            //start ad4m-host with publishing bootstrap
            let ad4m_host_init = std::process::Command::new(&ad4m_host_path)
                .arg("init")
                .arg("--networkBootstrapSeed")
                .arg(temp_publish_bootstrap_path.to_str().unwrap())
                .arg("--overrideConfig")
                .output()?;

            println!(
                "ad4m-host init output: {}\n",
                String::from_utf8_lossy(&ad4m_host_init.stdout)
            );

            println!(
                "Starting publishing with bootstrap path: {}\n",
                temp_publish_bootstrap_path.to_str().unwrap()
            );

            let (tx, rx) = channel();
            serve_ad4m_host(ad4m_host_path, tx)?;

            println!("ad4m-host serve started");
            println!("Listening for stdout...\n");

            for line in &rx {
                println!("{}", line);
                if line.contains("GraphQL server started, Unlock the agent to start holohchain") {
                    println!("AD4M Host ready for publishing\n");
                    //Spawn in a new thread so we can continue reading logs in loop below, whilst publishing is happening
                    tokio::spawn(async move {
                        start_publishing(
                            passphrase.clone(),
                            seed_proto.clone(),
                            lang_lang_source.clone(),
                        )
                        .await;
                    });
                    break;
                }
            }

            for line in rx {
                println!("{}", line);
            }
        }
    };
    Ok(())
}
