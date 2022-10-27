fn main() -> fpm::Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

async fn async_main() -> fpm::Result<()> {
    use colored::Colorize;

    let matches = app(authors(), version()).get_matches();

    if let Some(project) = matches.subcommand_matches("create-package") {
        // project-name => required field (any package Url or standard project name)
        let name = project.value_of_("name").unwrap();
        // project-path is optional
        let path = project.value_of_("path");
        return fpm::create_package(name, path).await;
    }

    if let Some(mark) = matches.subcommand_matches("serve") {
        let port = mark.value_of_("port").map(|p| match p.parse::<u16>() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Provided port {} is not a valid port.", p.to_string().red());
                std::process::exit(1);
            }
        });

        let bind = mark.value_of_("bind").unwrap_or("127.0.0.1").to_string();
        let download_base_url = mark.value_of_("download-base-url");

        return fpm::listen(
            bind.as_str(),
            port,
            download_base_url.map(ToString::to_string),
        )
        .await;
    }

    if let Some(clone) = matches.subcommand_matches("clone") {
        return fpm::clone(clone.value_of_("source").unwrap()).await;
    }

    let mut config = fpm::Config::read(None, true, None).await?;

    if matches.subcommand_matches("update").is_some() {
        return fpm::update(&config).await;
    }

    if let Some(edit) = matches.subcommand_matches("edit") {
        return fpm::edit(
            &config,
            edit.value_of_("file").unwrap(),
            edit.value_of_("cr").unwrap(),
        )
        .await;
    }

    if let Some(add) = matches.subcommand_matches("add") {
        // TODO: support multiple files
        return fpm::add(&config, add.value_of_("file").unwrap(), add.value_of_("cr")).await;
    }

    if let Some(rm) = matches.subcommand_matches("rm") {
        return fpm::rm(&config, rm.value_of_("file").unwrap(), rm.value_of_("cr")).await;
    }

    if let Some(merge) = matches.subcommand_matches("merge") {
        return fpm::merge(
            &config,
            merge.value_of_("src"),
            merge.value_of_("dest").unwrap(),
            merge.value_of_("file"), // TODO: support multiple files
        )
        .await;
    }

    if let Some(build) = matches.subcommand_matches("build") {
        if matches.get_flag("verbose") {
            println!("{}", fpm::debug_env_vars());
        }

        return fpm::build(
            &mut config,
            build.value_of_("file"), // TODO: handle more than one files
            build.value_of_("base").unwrap_or("/"),
            build.get_flag("ignore_failed"),
        )
        .await;
    }

    if let Some(mark_resolve) = matches.subcommand_matches("mark-resolved") {
        return fpm::mark_resolved(&config, mark_resolve.value_of_("path").unwrap()).await;
    }

    if let Some(abort_merge) = matches.subcommand_matches("abort-merge") {
        return fpm::abort_merge(&config, abort_merge.value_of_("path").unwrap()).await;
    }

    if let Some(revert) = matches.subcommand_matches("revert") {
        return fpm::revert(&config, revert.value_of_("path").unwrap()).await;
    }

    if let Some(sync) = matches.subcommand_matches("sync") {
        return if let Some(source) = sync.get_many::<String>("file") {
            let sources = source.map(|v| v.to_string()).collect();
            fpm::sync2(&config, Some(sources)).await
        } else {
            fpm::sync2(&config, None).await
        };
    }
    if let Some(status) = matches.subcommand_matches("sync-status") {
        // TODO: handle multiple files
        return fpm::sync_status(&config, status.value_of_("file")).await;
    }
    if let Some(create_cr) = matches.subcommand_matches("create-cr") {
        return fpm::create_cr(&config, create_cr.value_of_("title")).await;
    }
    if let Some(close_cr) = matches.subcommand_matches("close-cr") {
        return fpm::close_cr(&config, close_cr.value_of_("cr").unwrap()).await;
    }
    if let Some(status) = matches.subcommand_matches("status") {
        // TODO: handle multiple files
        return fpm::status(&config, status.value_of_("file")).await;
    }
    if matches.subcommand_matches("translation-status").is_some() {
        return fpm::translation_status(&config).await;
    }
    if let Some(diff) = matches.subcommand_matches("diff") {
        let all = diff.get_flag("all");
        return if let Some(source) = diff.get_many::<String>("source") {
            fpm::diff(&config, Some(source.map(|v| v.to_string()).collect()), all).await
        } else {
            fpm::diff(&config, None, all).await
        };
    }

    if let Some(resolve_conflict) = matches.subcommand_matches("resolve-conflict") {
        let use_ours = resolve_conflict.get_flag("use-ours");
        let use_theirs = resolve_conflict.get_flag("use-theirs");
        let print = resolve_conflict.get_flag("print");
        let revive_it = resolve_conflict.get_flag("revive-it");
        let delete_it = resolve_conflict.get_flag("delete-it");
        let source = resolve_conflict.value_of_("source").unwrap();
        return fpm::resolve_conflict(
            &config, source, use_ours, use_theirs, print, revive_it, delete_it,
        )
        .await;
    }
    if let Some(tracks) = matches.subcommand_matches("start-tracking") {
        let source = tracks.value_of_("source").unwrap();
        let target = tracks.value_of_("target").unwrap();
        return fpm::start_tracking(&config, source, target).await;
    }
    if let Some(mark) = matches.subcommand_matches("mark-upto-date") {
        let source = mark.value_of_("source").unwrap();
        let target = mark.value_of_("target");
        return fpm::mark_upto_date(&config, source, target).await;
    }
    if let Some(mark) = matches.subcommand_matches("stop-tracking") {
        let source = mark.value_of_("source").unwrap();
        let target = mark.value_of_("target");
        return fpm::stop_tracking(&config, source, target).await;
    }

    unreachable!("No subcommand matched");
}

fn app(authors: &'static str, version: &'static str) -> clap::Command {
    clap::Command::new("fpm: FTD Package Manager")
        .version(version)
        .author(authors)
        .arg_required_else_help(true)
        .arg(clap::arg!(verbose: -v "Sets the level of verbosity"))
        .arg(clap::arg!(test: --test "Runs the command in test mode").hide(true))
        .subcommand(
            // Initial subcommand format
            // fpm create-package <project-name> [project-path]
            //                   -n or --name   -p or --path
            // Necessary <project-name> with Optional [project-path]
            clap::Command::new("create-package")
                .about("Create a new FPM package")
                .arg(clap::arg!(name: <NAME> "The name of the package to create"))
                .arg(clap::arg!(path: -p --path [PATH] "Where to create the package (relative or absolute path, default value: the name)"))
        )
        .subcommand(
            clap::Command::new("build")
                .about("Build static site from this fpm package")
                .arg(clap::arg!(file: [FILE]... "The file to build (if specified only these are built, else entire package is built)"))
                .arg(clap::arg!(base: -b --base [BASE] "The base path.").default_value("/"))
                .arg(clap::arg!(ignore_failed: --"ignore-failed" "Ignore failed files."))
        )
        .subcommand(
            clap::Command::new("mark-resolved")
                .about("Marks the conflicted file as resolved")
                .arg(clap::arg!(path: <PATH> "The path of the conflicted file"))
                .hide(true), // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("abort-merge")
                .about("Aborts the remote changes")
                .arg(clap::arg!(path: <PATH> "The path of the conflicted file"))
                .hide(true), // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("clone")
                .about("Clone a package into a new directory")
                .arg(clap::arg!(source: <SOURCE> "The source package to clone"))
                .hide(true)
        )
        .subcommand(
            clap::Command::new("edit")
                .about("Edit a file in CR workspace")
                .arg(clap::arg!(file: <FILE> "The file to edit"))
                .arg(clap::arg!(cr: --cr <CR> "The CR to edit the file in").required(true))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("add")
                .about("Add one or more files in the workspace")
                .arg(clap::arg!(file: <FILE>... "The file(s) to add"))
                .arg(clap::arg!(cr: --cr <CR> "The CR to add the file(s) in"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("rm")
                .about("Removes one or more files from the workspace")
                .arg(clap::arg!(file: <FILE>... "The file(s) to remove"))
                .arg(clap::arg!(cr: --cr <CR> "The CR to remove the file(s) from"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("merge")
                .about("Merge two manifests together")
                .arg(clap::arg!(src: <SRC> "The source manifest to merge"))
                .arg(clap::arg!(dest: <DEST> "The destination manifest to merge"))
                .arg(clap::arg!(file: <FILE>... "The file(s) to merge"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("revert")
                .about("Reverts the local changes")
                .arg(clap::arg!(path: <PATH> "The path of the conflicted file"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("update")
                .about("Reinstall all the dependency packages")
        )
        .subcommand(
            clap::Command::new("sync")
                .about("Sync with fpm-repo (or .history folder if not using fpm-repo)")
                .arg(clap::arg!(file: <FILE>... "The file(s) to sync (leave empty to sync entire package)"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("status")
                .about("Show the status of files in this fpm package")
                .arg(clap::arg!(file: <FILE>... "The file(s) to see status of (leave empty to see status of entire package)").required(false))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("sync-status")
                .about("Show the sync status of files in this fpm package")
                .arg(clap::arg!(file: <FILE>... "The file(s) to see status of (leave empty to see status of entire package)").required(false))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("create-cr")
                .about("Create a Change Request")
                .arg(clap::arg!(title: <TITLE> "The title of the new CR"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("close-cr")
                .about("Create a Change Request")
                .arg(clap::arg!(cr: <CR> "The CR to Close"))
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("translation-status")
                .about("Show the translation status of files in this fpm package")
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("diff")
                .about("Show un-synced changes to files in this fpm package")
                .args(&[
                    clap::Arg::new("source").action(clap::ArgAction::Append),
                    clap::Arg::new("all").long("all").short('a'),
                ])
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("resolve-conflict")
                .about("Show un-synced changes to files in this fpm package")
                .args(&[
                    clap::Arg::new("use-ours")
                        .long("use-ours")
                        .action(clap::ArgAction::SetTrue),
                    clap::Arg::new("use-theirs")
                        .long("use-theirs")
                        .action(clap::ArgAction::SetTrue),
                    clap::Arg::new("revive-it")
                        .long("revive-it")
                        .action(clap::ArgAction::SetTrue),
                    clap::Arg::new("delete-it")
                        .long("delete-it")
                        .action(clap::ArgAction::SetTrue),
                    clap::Arg::new("print")
                        .long("print")
                        .action(clap::ArgAction::SetTrue),
                    clap::Arg::new("source").required(true),
                ])
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("check")
                .about("Check if everything is fine with current fpm package")
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("mark-upto-date")
                .about("Marks file as up to date.")
                .args(&[
                    clap::Arg::new("source").required(true),
                    clap::Arg::new("target")
                        .long("target")
                        .action(clap::ArgAction::Set),
                ])
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("start-tracking")
                .about("Add a tracking relation between two files")
                .args(&[
                    clap::Arg::new("source").required(true),
                    clap::Arg::new("target")
                        .long("target")
                        .action(clap::ArgAction::Set)
                        .required(true),
                ])
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(
            clap::Command::new("stop-tracking")
                .about("Remove a tracking relation between two files")
                .args(&[
                    clap::Arg::new("source").required(true),
                    clap::Arg::new("target")
                        .long("target")
                        .action(clap::ArgAction::Set),
                ])
                .hide(true) // hidden since the feature is not being released yet.
        )
        .subcommand(sub_command::serve())
}

mod sub_command {
    pub fn serve() -> clap::Command {
        let serve = clap::Command::new("serve")
            .about("Create an http server and serves static files")
            .arg(clap::arg!(port: --port <PORT> "The port to listen on").default_value("8080"))
            .arg(clap::arg!(bind: --bind <ADDRESS> "The address to bind to").default_value("127.0.0.1"))
            .arg(clap::arg!(download_base_url: --"download-base-url" <URL> "If running without files locally, download requested files from here."));
        if cfg!(feature = "remote") {
            serve
        } else {
            serve
                .arg(
                    clap::arg!(identities: --identities <IDENTITIES> "Http request identities, fpm allows these identities to access documents")
                        .hide(true) // this is only for testing purpose
                )
        }
    }
}

pub fn version() -> &'static str {
    if std::env::args().any(|e| e == "--test") {
        env!("CARGO_PKG_VERSION")
    } else {
        match option_env!("GITHUB_SHA") {
            Some(sha) => {
                Box::leak(format!("{} [{}]", env!("CARGO_PKG_VERSION"), sha).into_boxed_str())
            }
            None => env!("CARGO_PKG_VERSION"),
        }
    }
}

pub fn authors() -> &'static str {
    Box::leak(
        env!("CARGO_PKG_AUTHORS")
            .split(':')
            .map(|v| v.split_once('<').map(|(v, _)| v.trim()).unwrap_or_default())
            .collect::<Vec<_>>()
            .join(", ")
            .into_boxed_str(),
    )
}

trait ValueOf {
    fn value_of_(&self, name: &str) -> Option<&str>;
}

impl ValueOf for clap::ArgMatches {
    fn value_of_(&self, name: &str) -> Option<&str> {
        self.get_one::<String>(name).map(|v| v.as_str())
    }
}
