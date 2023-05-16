use sudo_test::{Command, Env, User};

use crate::{Result, USERNAME};

#[test]
fn sets_the_working_directory_of_the_executed_command() -> Result<()> {
    let expected_path = "/root";
    let env = Env(format!("ALL ALL=(ALL:ALL) CWD={expected_path} ALL")).build()?;

    let stdout = Command::new("sh")
        .args(["-c", "cd /; sudo pwd"])
        .exec(&env)?
        .stdout()?;

    assert_eq!(expected_path, stdout);

    Ok(())
}

#[test]
fn glob_has_no_effect_on_its_own() -> Result<()> {
    let env = Env("ALL ALL=(ALL:ALL) CWD=* ALL").build()?;

    let expected_path = "/";
    let stdout = Command::new("sh")
        .args(["-c", "cd {expected_path}; sudo pwd"])
        .exec(&env)?
        .stdout()?;

    assert_eq!(expected_path, stdout);

    Ok(())
}

#[test]
fn non_absolute_path_is_rejected() -> Result<()> {
    let env = Env("ALL ALL=(ALL:ALL) CWD=usr ALL").build()?;

    let output = Command::new("sh")
        .args(["-c", "cd /; sudo pwd"])
        .exec(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());

    if sudo_test::is_original_sudo() {
        assert_contains!(
            output.stderr(),
            "values for \"CWD\" must start with a '/', '~', or '*'"
        );
    }

    Ok(())
}

#[test]
fn dot_slash_is_rejected() -> Result<()> {
    let env = Env("ALL ALL=(ALL:ALL) CWD=./usr ALL").build()?;

    let output = Command::new("sh")
        .args(["-c", "cd /; sudo pwd"])
        .exec(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());

    if sudo_test::is_original_sudo() {
        assert_contains!(
            output.stderr(),
            "values for \"CWD\" must start with a '/', '~', or '*'"
        );
    }

    Ok(())
}

#[test]
fn tilde_when_target_user_is_root() -> Result<()> {
    let env = Env("ALL ALL=(ALL:ALL) CWD=~ ALL").build()?;

    let stdout = Command::new("sh")
        .args(["-c", "cd /; sudo pwd"])
        .exec(&env)?
        .stdout()?;

    assert_eq!("/root", stdout);

    Ok(())
}

#[test]
fn tilde_when_target_user_is_regular_user() -> Result<()> {
    let env = Env("ALL ALL=(ALL:ALL) CWD=~ NOPASSWD: ALL")
        .user(User(USERNAME).create_home_directory())
        .build()?;

    let stdout = Command::new("sh")
        .arg("-c")
        .arg(format!("cd /; sudo -u {USERNAME} pwd"))
        .exec(&env)?
        .stdout()?;

    assert_eq!(format!("/home/{USERNAME}"), stdout);

    Ok(())
}

#[test]
fn tilde_username() -> Result<()> {
    let env = Env(format!("ALL ALL=(ALL:ALL) CWD=~{USERNAME} NOPASSWD: ALL"))
        .user(User(USERNAME).create_home_directory())
        .build()?;

    for target_user in ["root", USERNAME] {
        let stdout = Command::new("sh")
            .arg("-c")
            .arg(format!("cd /; sudo -u {target_user} pwd"))
            .exec(&env)?
            .stdout()?;

        assert_eq!(format!("/home/{USERNAME}"), stdout);
    }

    Ok(())
}

#[test]
fn path_does_not_exist() -> Result<()> {
    let env = Env("ALL ALL=(ALL:ALL) CWD=/path/to/nowhere NOPASSWD: ALL").build()?;

    let output = Command::new("sh")
        .arg("-c")
        .arg("cd /; sudo pwd")
        .exec(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());

    if sudo_test::is_original_sudo() {
        assert_contains!(
            output.stderr(),
            "sudo: unable to change directory to /path/to/nowhere: No such file or directory"
        );
    }

    Ok(())
}

#[test]
fn path_is_file() -> Result<()> {
    let env = Env("ALL ALL=(ALL:ALL) CWD=/dev/null NOPASSWD: ALL").build()?;

    let output = Command::new("sh")
        .arg("-c")
        .arg("cd /; sudo pwd")
        .exec(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());

    if sudo_test::is_original_sudo() {
        assert_contains!(
            output.stderr(),
            "sudo: unable to change directory to /dev/null: Not a directory"
        );
    }

    Ok(())
}

#[test]
fn target_user_has_insufficient_permissions() -> Result<()> {
    let env = Env("ALL ALL=(ALL:ALL) CWD=/root NOPASSWD: ALL")
        .user(USERNAME)
        .build()?;

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("cd /; sudo -u {USERNAME} pwd"))
        .exec(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());

    if sudo_test::is_original_sudo() {
        assert_contains!(
            output.stderr(),
            "sudo: unable to change directory to /root: Permission denied"
        );
    }

    Ok(())
}
