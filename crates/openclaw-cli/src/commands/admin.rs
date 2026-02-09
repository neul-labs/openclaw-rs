//! Admin user management commands.

use std::path::PathBuf;

use openclaw_gateway::auth::{
    setup::generate_password, AuthConfig, User, UserRole, UserStore,
};

use crate::ui;

/// Arguments for admin commands.
pub struct AdminArgs {
    /// The admin action to perform.
    pub action: AdminAction,
    /// Data directory override.
    pub data_dir: Option<PathBuf>,
}

/// Admin actions.
pub enum AdminAction {
    /// Create a new user.
    Create {
        username: String,
        password: Option<String>,
        role: String,
        generate_password: bool,
    },
    /// List all users.
    List,
    /// Reset a user's password.
    ResetPassword { username: String },
    /// Enable a user account.
    Enable { username: String },
    /// Disable a user account.
    Disable { username: String },
    /// Delete a user.
    Delete { username: String },
}

/// Run the admin command.
///
/// # Errors
///
/// Returns error if the operation fails.
pub async fn run_admin(args: AdminArgs) -> anyhow::Result<()> {
    let data_dir = args.data_dir.unwrap_or_else(|| {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("openclaw")
            .join("gateway")
    });

    // Ensure directory exists
    std::fs::create_dir_all(&data_dir)?;

    // Open user store
    let store = UserStore::open(&data_dir).map_err(|e| anyhow::anyhow!("Failed to open user store: {}", e))?;

    match args.action {
        AdminAction::Create {
            username,
            password,
            role,
            generate_password: gen_pwd,
        } => {
            create_user(&store, &username, password.as_deref(), &role, gen_pwd)?;
        }
        AdminAction::List => {
            list_users(&store)?;
        }
        AdminAction::ResetPassword { username } => {
            reset_password(&store, &username)?;
        }
        AdminAction::Enable { username } => {
            set_user_active(&store, &username, true)?;
        }
        AdminAction::Disable { username } => {
            set_user_active(&store, &username, false)?;
        }
        AdminAction::Delete { username } => {
            delete_user(&store, &username)?;
        }
    }

    Ok(())
}

fn create_user(
    store: &UserStore,
    username: &str,
    password: Option<&str>,
    role_str: &str,
    gen_pwd: bool,
) -> anyhow::Result<()> {
    // Parse role
    let role: UserRole = role_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid role: {}. Use: admin, operator, or viewer", role_str))?;

    // Get or generate password
    let password = if gen_pwd {
        let pwd = generate_password(16);
        ui::success(&format!("Generated password: {}", pwd));
        pwd
    } else {
        password
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("Password required. Use --password or --generate-password"))?
    };

    // Create user
    let user = User::new(username, &password, role)
        .map_err(|e| anyhow::anyhow!("Failed to create user: {}", e))?;

    store
        .create(&user)
        .map_err(|e| anyhow::anyhow!("Failed to save user: {}", e))?;

    ui::success(&format!(
        "Created user '{}' with role '{}'",
        username, role
    ));

    Ok(())
}

fn list_users(store: &UserStore) -> anyhow::Result<()> {
    let users = store
        .list()
        .map_err(|e| anyhow::anyhow!("Failed to list users: {}", e))?;

    if users.is_empty() {
        ui::info("No users configured.");
        ui::info("Run 'openclaw admin create --username admin --generate-password' to create an admin user.");
        return Ok(());
    }

    ui::info(&format!("Users ({}):", users.len()));
    println!();
    println!("{:<20} {:<10} {:<8} {:<24}", "USERNAME", "ROLE", "ACTIVE", "CREATED");
    println!("{}", "-".repeat(65));

    for user in users {
        let created = user.created_at.format("%Y-%m-%d %H:%M:%S");
        let active = if user.active { "yes" } else { "no" };
        println!(
            "{:<20} {:<10} {:<8} {:<24}",
            user.username, user.role, active, created
        );
    }

    Ok(())
}

fn reset_password(store: &UserStore, username: &str) -> anyhow::Result<()> {
    let mut user = store
        .get_by_username(username)
        .map_err(|e| anyhow::anyhow!("Failed to find user: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("User not found: {}", username))?;

    let new_password = generate_password(16);

    user.set_password(&new_password)
        .map_err(|e| anyhow::anyhow!("Failed to set password: {}", e))?;

    store
        .update(&user)
        .map_err(|e| anyhow::anyhow!("Failed to update user: {}", e))?;

    ui::success(&format!("Password reset for user '{}'", username));
    ui::success(&format!("New password: {}", new_password));

    Ok(())
}

fn set_user_active(store: &UserStore, username: &str, active: bool) -> anyhow::Result<()> {
    let mut user = store
        .get_by_username(username)
        .map_err(|e| anyhow::anyhow!("Failed to find user: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("User not found: {}", username))?;

    user.active = active;

    store
        .update(&user)
        .map_err(|e| anyhow::anyhow!("Failed to update user: {}", e))?;

    let status = if active { "enabled" } else { "disabled" };
    ui::success(&format!("User '{}' {}", username, status));

    Ok(())
}

fn delete_user(store: &UserStore, username: &str) -> anyhow::Result<()> {
    // First find the user to get their ID
    let user = store
        .get_by_username(username)
        .map_err(|e| anyhow::anyhow!("Failed to find user: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("User not found: {}", username))?;

    // Check if this is the last admin
    let users = store
        .list()
        .map_err(|e| anyhow::anyhow!("Failed to list users: {}", e))?;

    let admin_count = users.iter().filter(|u| u.role.is_admin() && u.active).count();

    if user.role.is_admin() && admin_count <= 1 {
        return Err(anyhow::anyhow!("Cannot delete the last admin user"));
    }

    store
        .delete(&user.id)
        .map_err(|e| anyhow::anyhow!("Failed to delete user: {}", e))?;

    ui::success(&format!("Deleted user '{}'", username));

    Ok(())
}
