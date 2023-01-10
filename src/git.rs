use std::{path::Path, collections::{HashMap, hash_map::Entry}};
use git2::{Repository, Error, Revwalk, Oid, Diff, DiffDelta};


pub fn repo(path: &Path) -> Repository {
    match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to open repo at : '{}', err: '{}'", path.to_str().unwrap(), e),
    }
}

pub fn get_branch_name(repo: &Repository) -> Result<String, Error>  {
    let branch = repo
    .branches(Some(git2::BranchType::Local))?
    .flatten()
    .map(|(branch, _)| branch)
    .find(|branch| branch.is_head())
    .ok_or_else(|| git2::Error::from_str("detached HEAD not supported"))?;
    let branch_name = branch.name()?.expect("branch name is invalid UTF-8");
    Ok(String::from(branch_name))
}

pub fn get_commit_ids(repo: &Repository) -> Result<Vec<Oid>, Error> {
    let mut revwalk: Revwalk = repo.revwalk().unwrap();
    revwalk.set_sorting(git2::Sort::REVERSE)?;
    revwalk.push_head()?; 
    revwalk.collect::<Result<Vec<Oid>, Error>>()
}

pub fn checkout_last_commit(repo: &Repository, branch_name: &str) -> Result<(), git2::Error> {
    let refname = &("refs/heads/".to_owned() + branch_name);
    let obj = repo
    .revparse_single(refname)
    .unwrap();
    repo.checkout_tree(&obj, None)?;
    repo.set_head(refname)?;
    Ok(())
}

fn get_filepath_name(delta: DiffDelta) -> &str {
    delta.new_file().path().unwrap().to_str().unwrap()
}

fn display_commit_info(diff: Diff) {
    let mut map = HashMap::new();        
    diff.deltas().for_each(|item| {
        let key = format!("{:?}", item.status());
        match map.entry(key) {
            Entry::Occupied(mut entry) => {
                let vec: &mut Vec<&str> = entry.get_mut();
                vec.push(get_filepath_name(item));
                let value: Vec<&str> = vec.to_vec();
                entry.insert(value);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec!(get_filepath_name(item)));
            }
        }
    });
    for (status, filepaths) in map {        
        let filepaths = filepaths
            .iter()
            .enumerate()
            .map(|(i, &path_str)| format!("[{:?}] {}",i + 1, path_str))
            .collect::<Vec<_>>()
            .join("\n");        
        println!("🔎 \n===== {} =====\n{}",status, filepaths); 
    }
}

// Inspiration:
// https://github.com/rust-lang/git2-rs/issues/588
pub fn show_commit_info(repo: &Repository, commit_id: Oid) {
    let commit = repo.find_commit(commit_id).unwrap();
    // Ignore merge commits (2+ parents) because that's what 'git whatchanged' does.
    // Ignore commit with 0 parents (initial commit) because there's nothing to diff against
    if commit.parent_count() == 1 {
        let prev_commit = commit.parent(0).unwrap();
        let tree = commit.tree().unwrap();
        let prev_tree = prev_commit.tree().unwrap();
        let diff= repo.diff_tree_to_tree(Some(&prev_tree), Some(&tree), None).unwrap();
        display_commit_info(diff);
    }
}


pub fn move_git_pointer(repo: &Repository, commit_id: Oid, progress: String) {
    let commit = repo.find_commit(commit_id).unwrap();
    
    println!("{} '{:?}' | '{}' by '{}' ", progress, commit_id, commit.message().unwrap().trim(), commit.author());
    let new_head = repo.revparse_single(&format!("{}", commit_id)).unwrap();

    match repo
    .checkout_tree(&new_head, None) {
            Ok(_) => {},
            Err(e) => println!("failed to checkout_tree, {:?}", e)
    }
    match repo.set_head_detached(commit_id) {
        Ok(_) => {},
        Err(e) => println!("failed to detach, {:?}", e)
    }
    match repo
    .reset(&new_head, git2::ResetType::Soft, None) {
        Ok(_) => {},
        Err(e) => println!("failed to move HEAD to {:?}", e)
    }
}
