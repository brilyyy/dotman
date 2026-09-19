use clap_complete::Shell;
use dotman::cli::{AddArgs, BannerArgs, CompletionsArgs, DeployArgs, InstallDepsArgs, RemoveArgs};
use dotman::commands;
use dotman::config::{DeployMethod, DotConfig, DEFAULT_BACKUP_DIR, DEFAULT_MANIFEST_NAME};
use dotman::fs::{check_copy_status, check_symlink_status, compute_diff, CopyStatus, SymlinkStatus};
use dotman::hooks::run_hook;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_init_creates_files() {
    let temp = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    let res = commands::init::execute();
    std::env::set_current_dir(original_cwd).unwrap();

    assert!(res.is_ok());
    assert!(temp.path().join(DEFAULT_MANIFEST_NAME).exists());
    assert!(temp.path().join(DEFAULT_BACKUP_DIR).exists());

    let cfg = DotConfig::load_from_path(&temp.path().join(DEFAULT_MANIFEST_NAME)).unwrap();
    assert!(cfg.settings.backup_enabled);
    assert_eq!(cfg.settings.backup_dir, ".bak");
    assert!(cfg.items.contains_key("zsh/zshrc"));
    assert!(cfg.items.contains_key("nvim"));
}

#[test]
fn test_add_file_and_symlink() {
    let temp = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    commands::init::execute().unwrap();

    let user_home = temp.path().join("fake_home");
    fs::create_dir_all(&user_home).unwrap();
    let mock_file = user_home.join(".bashrc");
    fs::write(&mock_file, "alias ll='ls -la'").unwrap();

    let add_args = AddArgs {
        path: mock_file.clone(),
        name: Some("shell/bashrc".to_string()),
        tags: vec!["shell".to_string()],
        copy: false,
    };

    let res = commands::add::execute(add_args);
    std::env::set_current_dir(original_cwd).unwrap();

    assert!(res.is_ok());

    let repo_file = temp.path().join("shell/bashrc");
    assert!(repo_file.exists());
    assert_eq!(fs::read_to_string(&repo_file).unwrap(), "alias ll='ls -la'");

    let symlink_meta = fs::symlink_metadata(&mock_file).unwrap();
    assert!(symlink_meta.file_type().is_symlink());
    assert_eq!(check_symlink_status(&repo_file, &mock_file), SymlinkStatus::Valid);

    let cfg = DotConfig::load_from_path(&temp.path().join(DEFAULT_MANIFEST_NAME)).unwrap();
    assert!(cfg.items.contains_key("shell/bashrc"));
}

#[test]
fn test_remove_demigrates_real_file() {
    let temp = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    commands::init::execute().unwrap();

    let user_home = temp.path().join("home");
    fs::create_dir_all(&user_home).unwrap();
    let test_file = user_home.join(".gitconfig");
    fs::write(&test_file, "[user]\nname = Tester").unwrap();

    commands::add::execute(AddArgs {
        path: test_file.clone(),
        name: Some("gitconfig".to_string()),
        tags: vec![],
        copy: false,
    }).unwrap();

    assert!(fs::symlink_metadata(&test_file).unwrap().file_type().is_symlink());

    // Now remove without purge -> should restore real file and delete repo copy
    let res = commands::remove::execute(RemoveArgs {
        item: "gitconfig".to_string(),
        purge: false,
    });
    std::env::set_current_dir(original_cwd).unwrap();

    assert!(res.is_ok());
    assert!(!fs::symlink_metadata(&test_file).unwrap().file_type().is_symlink(), "Target should be a regular file now");
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "[user]\nname = Tester");
    assert!(!temp.path().join("gitconfig").exists(), "Repo file should be removed");

    let cfg = DotConfig::load_from_path(&temp.path().join(DEFAULT_MANIFEST_NAME)).unwrap();
    assert!(!cfg.items.contains_key("gitconfig"), "Item should be removed from manifest");
}

#[test]
fn test_remove_purge() {
    let temp = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    commands::init::execute().unwrap();

    let user_home = temp.path().join("home");
    fs::create_dir_all(&user_home).unwrap();
    let test_file = user_home.join(".tmux.conf");
    fs::write(&test_file, "set -g mouse on").unwrap();

    commands::add::execute(AddArgs {
        path: test_file.clone(),
        name: Some("tmux.conf".to_string()),
        tags: vec![],
        copy: false,
    }).unwrap();

    let res = commands::remove::execute(RemoveArgs {
        item: "tmux.conf".to_string(),
        purge: true,
    });
    std::env::set_current_dir(original_cwd).unwrap();

    assert!(res.is_ok());
    assert!(!test_file.exists(), "Target file should be deleted");
    assert!(!temp.path().join("tmux.conf").exists(), "Repo file should be deleted");
}

#[test]
fn test_diff_computation() {
    let temp = tempdir().unwrap();
    let file_a = temp.path().join("a.txt");
    let file_b = temp.path().join("b.txt");

    fs::write(&file_a, "line 1\nline 2\n").unwrap();
    fs::write(&file_b, "line 1\nline 3\n").unwrap();

    let diff = compute_diff(&file_a, &file_b).unwrap();
    assert!(diff.contains("-line 2"));
    assert!(diff.contains("+line 3"));
}

#[test]
fn test_deploy_dry_run() {
    let temp = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    commands::init::execute().unwrap();

    let res = commands::deploy::execute(DeployArgs {
        tag: None,
        dry_run: true,
        force: false,
        copy: false,
    });
    std::env::set_current_dir(original_cwd).unwrap();

    assert!(res.is_ok());
}

#[test]
fn test_install_deps_dry_run() {
    let temp = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    commands::init::execute().unwrap();

    let res = commands::install_deps::execute(InstallDepsArgs {
        category: Some("core".to_string()),
        manager: None,
        cmd: None,
        script: None,
        dry_run: true,
    });
    std::env::set_current_dir(original_cwd).unwrap();

    assert!(res.is_ok());
}

#[test]
fn test_install_deps_custom_command_and_script() {
    let temp = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    commands::init::execute().unwrap();

    // 1. Test custom command template override
    let res = commands::install_deps::execute(InstallDepsArgs {
        category: Some("core".to_string()),
        manager: None,
        cmd: Some("cargo binstall -y {packages}".to_string()),
        script: None,
        dry_run: true,
    });
    assert!(res.is_ok());

    // 2. Test custom script execution
    let script_path = temp.path().join("my_installer.sh");
    fs::write(&script_path, "#!/bin/sh\necho \"installed: $@\" > receipt.txt\n").unwrap();

    let res = commands::install_deps::execute(InstallDepsArgs {
        category: Some("core".to_string()),
        manager: None,
        cmd: None,
        script: Some(script_path.clone()),
        dry_run: false,
    });
    assert!(res.is_ok());
    assert!(temp.path().join("receipt.txt").exists());

    // 3. Test manager override to cargo
    let res = commands::install_deps::execute(InstallDepsArgs {
        category: Some("rust_tools".to_string()),
        manager: Some("cargo".to_string()),
        cmd: None,
        script: None,
        dry_run: true,
    });
    assert!(res.is_ok());

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_completions_execution() {
    let res = commands::completions::execute(CompletionsArgs {
        shell: Shell::Bash,
    });
    assert!(res.is_ok());

    let res = commands::completions::execute(CompletionsArgs {
        shell: Shell::Zsh,
    });
    assert!(res.is_ok());

    let res = commands::completions::execute(CompletionsArgs {
        shell: Shell::Fish,
    });
    assert!(res.is_ok());
}

#[test]
fn test_hook_runner() {
    let temp = tempdir().unwrap();
    let token = temp.path().join("token.txt");
    let cmd = format!("echo 'hook executed' > {}", token.display());

    let res = run_hook(&cmd);
    assert!(res.is_ok());
    assert_eq!(fs::read_to_string(&token).unwrap().trim(), "hook executed");
}

#[test]
fn test_add_and_deploy_copy_mode() {
    let temp = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    commands::init::execute().unwrap();

    let user_home = temp.path().join("home");
    fs::create_dir_all(&user_home).unwrap();
    let ssh_config = user_home.join(".ssh_config");
    fs::write(&ssh_config, "Host github.com\n  User git\n").unwrap();

    // 1. Add with --copy
    let add_args = AddArgs {
        path: ssh_config.clone(),
        name: Some("ssh/config".to_string()),
        tags: vec!["ssh".to_string()],
        copy: true,
    };
    let res = commands::add::execute(add_args);
    assert!(res.is_ok());

    // Repo file exists
    let repo_file = temp.path().join("ssh/config");
    assert!(repo_file.exists());
    assert_eq!(fs::read_to_string(&repo_file).unwrap(), "Host github.com\n  User git\n");

    // Target file is NOT a symlink, it's a real file
    let meta = fs::symlink_metadata(&ssh_config).unwrap();
    assert!(!meta.file_type().is_symlink());

    // Manifest contains item with method = DeployMethod::Copy
    let cfg = DotConfig::load_from_path(&temp.path().join(DEFAULT_MANIFEST_NAME)).unwrap();
    assert_eq!(cfg.items.get("ssh/config").unwrap().method, DeployMethod::Copy);

    // Status check is in sync
    assert_eq!(check_copy_status(&repo_file, &ssh_config), CopyStatus::InSync);

    // 2. Modify target and verify copy status becomes Modified
    fs::write(&ssh_config, "Host github.com\n  User custom\n").unwrap();
    assert!(matches!(check_copy_status(&repo_file, &ssh_config), CopyStatus::Modified(_)));

    // 3. Deploy without --copy (manifest item has method = copy) with force: true
    let deploy_res = commands::deploy::execute(DeployArgs {
        tag: None,
        dry_run: false,
        force: true,
        copy: false,
    });
    assert!(deploy_res.is_ok());
    // After deploy, target is in sync with repo file and still regular file
    assert_eq!(fs::read_to_string(&ssh_config).unwrap(), "Host github.com\n  User git\n");
    assert!(!fs::symlink_metadata(&ssh_config).unwrap().file_type().is_symlink());

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_deploy_override_copy_flag() {
    let temp = tempdir().unwrap();
    let original_cwd = std::env::current_dir().unwrap();

    std::env::set_current_dir(temp.path()).unwrap();
    commands::init::execute().unwrap();

    let user_home = temp.path().join("home");
    fs::create_dir_all(&user_home).unwrap();
    let test_file = user_home.join(".config_file");
    fs::write(&test_file, "original content").unwrap();

    // Add with standard symlink
    commands::add::execute(AddArgs {
        path: test_file.clone(),
        name: Some("test_file".to_string()),
        tags: vec![],
        copy: false,
    }).unwrap();

    assert!(fs::symlink_metadata(&test_file).unwrap().file_type().is_symlink());

    // Deploy with --copy --force -> converts symlink to regular file copy
    let res = commands::deploy::execute(DeployArgs {
        tag: None,
        dry_run: false,
        force: true,
        copy: true,
    });
    assert!(res.is_ok());

    let meta = fs::symlink_metadata(&test_file).unwrap();
    assert!(!meta.file_type().is_symlink(), "Deploy --copy should deploy as regular file");
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "original content");

    std::env::set_current_dir(original_cwd).unwrap();
}

#[test]
fn test_banner_execution() {
    let res = commands::banner::execute(BannerArgs::default());
    assert!(res.is_ok());

    let res = commands::banner::execute(BannerArgs {
        cat: true,
        plain: true,
        ..Default::default()
    });
    assert!(res.is_ok());

    let res = commands::banner::execute(BannerArgs {
        bot: true,
        plain: true,
        ..Default::default()
    });
    assert!(res.is_ok());

    let res = commands::banner::execute(BannerArgs {
        font: true,
        plain: true,
        ..Default::default()
    });
    assert!(res.is_ok());

    let res = commands::banner::execute(BannerArgs {
        mini: true,
        plain: true,
        ..Default::default()
    });
    assert!(res.is_ok());
}


