use clap::{Parser, Subcommand};

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
  /// Serve a port via a rendrok service
  Serve {
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    /// The host to serve from
    host: String,

    /// The port to serve from
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
      println!("ID                       Name    Region        URL");
      for list_service in list_services.as_array().unwrap() {
        if list_service["service"]["repo"]
          .as_str()
          .unwrap()
          .contains("evshiron/rendrok")
        {
          println!(
            "{} {} {} {}",
            list_service["service"]["id"].as_str().unwrap(),
            list_service["service"]["name"].as_str().unwrap(),
            list_service["service"]["serviceDetails"]["region"]
              .as_str()
              .unwrap(),
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
    Command::Serve { host, port } => {
      // free account can not use api to create services
      // a deploy to render button should be placed in readme
      // to easily create services in advance

      let list_services = render_api::list_services().await;
      for list_service in list_services.as_array().unwrap() {
        if list_service["service"]["repo"]
          .as_str()
          .unwrap()
          .contains("evshiron/rendrok")
        {
          let service_id = list_service["service"]["id"].as_str().unwrap();
          let service_url = list_service["service"]["serviceDetails"]["url"]
            .as_str()
            .unwrap();

          let list_env_vars = render_api::list_env_vars(service_id).await;
          for list_env_var in list_env_vars.as_array().unwrap() {
            if list_env_var["envVar"]["key"].as_str().unwrap() == "RENDROK_PASS" {
              let pass = list_env_var["envVar"]["value"].as_str().unwrap();
              let auth = format!("rendrok:{pass}");

              std::process::Command::new("chisel")
                .args([
                  "client",
                  "--auth",
                  &auth,
                  service_url,
                  format!("R:127.0.0.1:3000:{host}:{port}").as_str(),
                ])
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
            }
          }

          return Ok(());
        }
      }

      println!("Serve failed: service not found.");
    },
    Command::Logout {} => {
      let config = config::Config::default();
      config.save();
      println!("Logout succeeded.");
    },
  }

  Ok(())
}
