mod git;

use std::{path::Path, io, str::FromStr};
use clap::{Parser, arg, command};
use git2::{Repository, Oid};
use cli_table::{format::Justify, Cell, Table};
use crate::git::{repo, get_commit_ids, move_git_pointer, checkout_last_commit, get_branch_name, show_commit_info};

#[derive(Parser)]
#[command(name = "Archeologit")]
#[command(author = "Evgeny Tolbakov <ev.tolbakov@gmail.com>")]
#[command(version = "1.0")]
#[command(about = "Tool to play through git commits", long_about = None)]
struct Cli {
    #[arg(long)]
    path: String,
}

#[derive(Debug, PartialEq)]
enum Cmd {
    Back(usize),
    Next(usize),
    Info,
    Quit,
    Help,
    Noop,
}

impl FromStr for Cmd {

    type Err = ();

    fn from_str(input: &str) -> Result<Cmd, Self::Err> {
        let uppercased_input = input.trim().to_uppercase();
        let mut args = uppercased_input.split_whitespace();
        match args.next().unwrap() {
            "B"     => Ok(Cmd::Back(get_step_size(args.next()))),
            "N"     => Ok(Cmd::Next(get_step_size(args.next()))),
            "I"     => Ok(Cmd::Info),
            "Q"     => Ok(Cmd::Quit),
            "H"     => Ok(Cmd::Help),
            _       => Ok(Cmd::Noop),
        }
    }
}

fn get_progress(current: usize, total: usize) -> String {
    format!("[{}/{}]", current + 1, total + 1)
}

fn get_step_size(maybe_step : Option<&str>) -> usize {
    match maybe_step {
        Some(s) => s.parse().unwrap_or(DEFAULT_STEP),
        None => 1,
    }
}

static DEFAULT_STEP: usize = 1;
static QUIT_MESSAGE:&str = "Moving to the latest commit and quiting!";
static HELP_MESSAGE: &str = "Archeologit v1.0.0. Reference
Supported commands are case insensitive\n\
";

fn main() {
    let cli = Cli::parse();
    println!("Repo path: {:?}", cli.path);
    let repo: Repository = repo(Path::new(&cli.path));
    let commit_ids: &[Oid] = &match get_commit_ids(&repo) {
        Ok(ids) => ids,
        Err(e) => panic!("failed to get commits, {:?}", e)
    };
    
    let mut index = 0;
    let max_index = commit_ids.len() - 1;
    let branch_name = get_branch_name(&repo).unwrap();
    println!("Total number of commits = {} on a branch '{}'", commit_ids.len(), branch_name);
    move_git_pointer(&repo, commit_ids[index], get_progress(index, max_index));

    let help_table = vec![
        vec!["b <step>".cell(), "⏭  go to previous <step> commits; <step> is 1 by default".cell().justify(Justify::Left)],
        vec!["n <step>".cell(), "⏮  go to next <step> commits; <step> is 1 by default".cell().justify(Justify::Left)],
        vec!["i".cell(), "🔎 get information about current commit: file path and status".cell().justify(Justify::Left)],
        vec!["h".cell(), "🤓 help, show command options".cell().justify(Justify::Left)],
        vec!["q".cell(), "👋 quit".cell().justify(Justify::Left)],                
    ].table();

    loop {
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        match Cmd::from_str(&command).unwrap() {
            Cmd::Back(n) => {
                index -= n;
                if index <= 0 {
                    index = 0;
                }
                print!("⏭ ");
                move_git_pointer(&repo, commit_ids[index], get_progress(index, max_index));
            }
            Cmd::Next(n) => {
                index += n;
                if index >= max_index {
                    index = max_index;
                }  
                print!("⏮ ");                          
                move_git_pointer(&repo, commit_ids[index], get_progress(index, max_index));
            },
            Cmd::Info => show_commit_info(&repo, commit_ids[index]),            
            Cmd::Quit => {
                println!("👋 {}", QUIT_MESSAGE);            
                checkout_last_commit(&repo, &branch_name).unwrap();
                break;
            },
            Cmd::Help => println!("🤓 {}\n{}", HELP_MESSAGE, help_table.display().unwrap()),
            Cmd::Noop => println!("🙈 The command '{}' is not supported", &command.trim()),
        }
    }    
}
