use std::io::ErrorKind;

use clap::{Parser, Subcommand};
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};

use futures::stream::StreamExt;

mod config;
mod render_api;

#[derive(Subcommand)]
enum Command {
  /// Log in with render.com api key
  Login {},
  /// List existing rendrok services
  Ls {},
  /// Remove a rendrok service
  Rm { service_id: String },
  /// Serve a port via rendrok service
  Serve {
    #[arg(short, long)]
    /// The name of selected service
    service_name: Option<String>,

    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    /// The host to be exposed
    host: String,

    /// The port to be exposed
    port: u16,
  },
  /// Log out and clean up config
  Logout {},
}

#[derive(Parser)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut config = config::Config::load();

  let cli = Cli::parse();

  match &cli.command {
    Command::Login {} => {
      println!("Please enter your render.com API key here:");
      let mut line = String::new();
      std::io::stdin().read_line(&mut line).unwrap();
      let api_key = line.trim();

      let list_owners = render_api::list_owners().await;
      let owner_email = list_owners[0]["owner"]["email"].as_str().unwrap();
      let owner_id = list_owners[0]["owner"]["id"].as_str().unwrap();

      config.api_key = String::from(api_key);
      config.owner_email = String::from(owner_email);
      config.owner_id = String::from(owner_id);

      config.save();
      println!("Login succeeded as {owner_email}.");
    },
    Command::Ls {} => {
      let list_services = render_api::list_services().await;
      println!(
        "{:24} {:16} {:12} {:12} {}",
        "ID", "Name", "Region", "Active", "URL"
      );
      for list_service in list_services.as_array().unwrap() {
        if list_service["service"]["repo"]
          .as_str()
          .unwrap()
          .contains("evshiron/rendrok")
        {
          println!(
            "{:24} {:16} {:12} {:12} {}",
            list_service["service"]["id"].as_str().unwrap(),
            list_service["service"]["name"].as_str().unwrap(),
            list_service["service"]["serviceDetails"]["region"]
              .as_str()
              .unwrap(),
            list_service["service"]["suspended"].as_str().unwrap() != "suspended",
            list_service["service"]["serviceDetails"]["url"]
              .as_str()
              .unwrap()
          );
        }
      }
    },
    Command::Rm { service_id } => {
      println!("Are you really sure to delete service {service_id}? [Yes]");
      let mut line = String::new();
      std::io::stdin().read_line(&mut line).unwrap();
      if line.trim() == "Yes" {
        render_api::delete_service(service_id).await;
      }
    },
    Command::Serve {
      service_name,
      host,
      port,
    } => {
      // free account can not use api to create services
      // a deploy to render button should be placed in readme
      // to easily create services in advance

      let list_services = render_api::list_services().await;
      for list_service in list_services.as_array().unwrap() {
        if list_service["service"]["repo"]
          .as_str()
          .unwrap()
          .contains("evshiron/rendrok")
          && list_service["service"]["suspended"].as_str().unwrap() != "suspended"
        {
          let service_id = list_service["service"]["id"].as_str().unwrap();

          let _service_name = list_service["service"]["name"].as_str().unwrap();
          if let Some(service_name) = service_name {
            if _service_name != service_name.as_str() {
              continue;
            }
          }

          let service_url = list_service["service"]["serviceDetails"]["url"]
            .as_str()
            .unwrap();

          let list_env_vars = render_api::list_env_vars(service_id).await;
          for list_env_var in list_env_vars.as_array().unwrap() {
            if list_env_var["envVar"]["key"].as_str().unwrap() == "RENDROK_PASS" {
              let pass = list_env_var["envVar"]["value"].as_str().unwrap();
              let auth = format!("rendrok:{pass}");

              let mut command_args: Vec<&str> = Vec::new();

              command_args.push("client");
              command_args.push("--auth");
              command_args.push(&auth);

              let proxy: String = {
                if let Ok(https_proxy) = std::env::var("HTTPS_PROXY") {
                  https_proxy
                } else if let Ok(all_proxy) = std::env::var("ALL_PROXY") {
                  all_proxy
                } else if let Ok(http_proxy) = std::env::var("HTTP_PROXY") {
                  http_proxy
                } else {
                  String::from("")
                }
              };

              if !proxy.is_empty() {
                command_args.push("--proxy");
                command_args.push(&proxy);
              }

              command_args.push(service_url);

              let remote_arg = format!("R:127.0.0.1:3000:{host}:{port}");
              command_args.push(remote_arg.as_str());

              match tokio::process::Command::new("chisel")
                .args(command_args)
                .spawn()
              {
                Ok(mut child) => {
                  println!("\n==== {host}:{port} is now exposed on {service_url} ====\n");

                  let mut signals =
                    signal_hook_tokio::Signals::new(&[SIGINT, SIGQUIT, SIGTERM]).unwrap();
                  let handle = signals.handle();

                  loop {
                    tokio::select! {
                      _ = child.wait() => {
                        break;
                      },
                      Some(signal) = signals.next() => {
                        match signal {
                          SIGINT | SIGQUIT | SIGTERM => {
                            nix::sys::signal::kill(
                              nix::unistd::Pid::from_raw(
                                child.id().unwrap().try_into().unwrap()
                              ),
                              nix::sys::signal::Signal::SIGINT,
                            ).unwrap();
                          },
                          _ => unreachable!(),
                        }
                      },
                    }
                  }

                  handle.close();
                },
                Err(err) => {
                  if err.kind() == ErrorKind::NotFound {
                    println!("Serve failed: chisel is not found in $PATH.");
                    println!("See https://github.com/evshiron/rendrok for details.");
                  } else {
                    panic!("{}", err);
                  }
                },
              }
            }
          }

          return Ok(());
        }
      }

      println!("Serve failed: service not found.");
      println!("See https://github.com/evshiron/rendrok for details.");
    },
    Command::Logout {} => {
      let config = config::Config::default();
      config.save();
      println!("Logout succeeded.");
    },
  }

  Ok(())
}
