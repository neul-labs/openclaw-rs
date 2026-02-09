//! Gateway server.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use tokio::sync::RwLock;

use openclaw_core::events::{
    EventStore, SessionEvent, SessionEventKind, SessionMessage, SessionProjection, SessionState,
};
use openclaw_core::types::{AgentId, ChannelId, SessionKey};
use openclaw_agents::runtime::{AgentContext, AgentRuntime};
use openclaw_agents::tools::ToolRegistry;
use openclaw_channels::{ChannelRegistry, ChannelProbe, ChannelCapabilities};

use crate::auth::{
    AuthConfig, AuthState, BootstrapManager, JwtManager, SetupStatus,
    User, UserRole, UserStore, setup::auto_setup_from_env,
};
use crate::events::{EventBroadcaster, UiEvent, UiEventEnvelope};
use crate::rpc::{self, RpcRequest, RpcResponse};
use crate::GatewayError;

#[cfg(feature = "ui")]
use crate::ui_server::UiServerConfig;

/// Gateway configuration.
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Port to listen on.
    pub port: u16,
    /// Bind address.
    pub bind_address: String,
    /// Enable CORS.
    pub cors: bool,
    /// Data directory for persistent storage.
    pub data_dir: PathBuf,
    /// Authentication configuration.
    pub auth: AuthConfig,
    /// UI server configuration (optional, requires "ui" feature).
    #[cfg(feature = "ui")]
    pub ui: Option<UiServerConfig>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("openclaw")
            .join("gateway");

        Self {
            port: 18789,
            bind_address: "127.0.0.1".to_string(),
            cors: true,
            data_dir,
            auth: AuthConfig::default(),
            #[cfg(feature = "ui")]
            ui: Some(UiServerConfig::default()),
        }
    }
}

/// Gateway server state shared across handlers.
pub struct GatewayState {
    /// Event store for session persistence.
    pub event_store: Arc<EventStore>,
    /// Agent runtimes by agent ID.
    pub agents: HashMap<String, Arc<AgentRuntime>>,
    /// Shared tool registry.
    pub tool_registry: Arc<ToolRegistry>,
    /// Authentication state.
    pub auth: Arc<AuthState>,
    /// Channel registry.
    pub channels: Arc<RwLock<ChannelRegistry>>,
    /// UI event broadcaster.
    pub events: EventBroadcaster,
    /// Gateway configuration.
    pub config: GatewayConfig,
}

/// Gateway server.
pub struct Gateway {
    config: GatewayConfig,
    state: Arc<RwLock<GatewayState>>,
}

/// Builder for constructing a Gateway with its dependencies.
pub struct GatewayBuilder {
    config: GatewayConfig,
    event_store: Option<Arc<EventStore>>,
    agents: HashMap<String, Arc<AgentRuntime>>,
    tool_registry: Arc<ToolRegistry>,
    auth_state: Option<Arc<AuthState>>,
    channel_registry: Option<Arc<RwLock<ChannelRegistry>>>,
    event_broadcaster: Option<EventBroadcaster>,
}

impl GatewayBuilder {
    /// Create a new builder with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GatewayConfig::default(),
            event_store: None,
            agents: HashMap::new(),
            tool_registry: Arc::new(ToolRegistry::new()),
            auth_state: None,
            channel_registry: None,
            event_broadcaster: None,
        }
    }

    /// Set gateway configuration.
    #[must_use]
    pub fn with_config(mut self, config: GatewayConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the event store.
    #[must_use]
    pub fn with_event_store(mut self, store: Arc<EventStore>) -> Self {
        self.event_store = Some(store);
        self
    }

    /// Register an agent runtime.
    #[must_use]
    pub fn with_agent(mut self, id: impl Into<String>, runtime: Arc<AgentRuntime>) -> Self {
        self.agents.insert(id.into(), runtime);
        self
    }

    /// Set the tool registry.
    #[must_use]
    pub fn with_tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = registry;
        self
    }

    /// Set the auth state.
    #[must_use]
    pub fn with_auth_state(mut self, auth: Arc<AuthState>) -> Self {
        self.auth_state = Some(auth);
        self
    }

    /// Set the channel registry.
    #[must_use]
    pub fn with_channel_registry(mut self, registry: Arc<RwLock<ChannelRegistry>>) -> Self {
        self.channel_registry = Some(registry);
        self
    }

    /// Set the event broadcaster.
    #[must_use]
    pub fn with_event_broadcaster(mut self, broadcaster: EventBroadcaster) -> Self {
        self.event_broadcaster = Some(broadcaster);
        self
    }

    /// Build the gateway.
    ///
    /// # Errors
    ///
    /// Returns error if event store is not configured or auth initialization fails.
    pub fn build(self) -> Result<Gateway, GatewayError> {
        let event_store = self.event_store.ok_or_else(|| {
            GatewayError::Config("Event store is required".to_string())
        })?;

        // Ensure data directory exists
        std::fs::create_dir_all(&self.config.data_dir)
            .map_err(|e| GatewayError::Config(format!("Failed to create data dir: {e}")))?;

        // Initialize auth state if not provided
        let auth = match self.auth_state {
            Some(auth) => auth,
            None => {
                let auth_config = self.config.auth.clone().with_env_overrides();
                Arc::new(
                    AuthState::initialize(auth_config, &self.config.data_dir)
                        .map_err(|e| GatewayError::Config(format!("Auth init failed: {e}")))?
                )
            }
        };

        // Auto-setup from environment if configured
        if let Err(e) = auto_setup_from_env(&auth.users) {
            tracing::warn!("Auto-setup from env failed: {}", e);
        }

        // Initialize channel registry
        let channels = self
            .channel_registry
            .unwrap_or_else(|| Arc::new(RwLock::new(ChannelRegistry::new())));

        // Initialize event broadcaster
        let events = self.event_broadcaster.unwrap_or_default();

        let state = GatewayState {
            event_store,
            agents: self.agents,
            tool_registry: self.tool_registry,
            auth,
            channels,
            events,
            config: self.config.clone(),
        };

        Ok(Gateway {
            config: self.config,
            state: Arc::new(RwLock::new(state)),
        })
    }
}

impl Default for GatewayBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Gateway {
    /// Create a new gateway (for backward compatibility).
    pub fn new(config: GatewayConfig) -> Result<Self, GatewayError> {
        // Ensure data directory exists
        std::fs::create_dir_all(&config.data_dir)
            .map_err(|e| GatewayError::Config(format!("Failed to create data dir: {e}")))?;

        // Create event store in data directory
        let event_store = Arc::new(
            EventStore::open(&config.data_dir.join("events"))
                .map_err(|e| GatewayError::Server(format!("Failed to open event store: {e}")))?
        );

        // Initialize auth
        let auth_config = config.auth.clone().with_env_overrides();
        let auth = Arc::new(
            AuthState::initialize(auth_config, &config.data_dir)
                .map_err(|e| GatewayError::Config(format!("Auth init failed: {e}")))?
        );

        // Auto-setup from environment if configured
        if let Err(e) = auto_setup_from_env(&auth.users) {
            tracing::warn!("Auto-setup from env failed: {}", e);
        }

        let state = GatewayState {
            event_store,
            agents: HashMap::new(),
            tool_registry: Arc::new(ToolRegistry::new()),
            auth,
            channels: Arc::new(RwLock::new(ChannelRegistry::new())),
            events: EventBroadcaster::new(),
            config: config.clone(),
        };

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(state)),
        })
    }

    /// Run the gateway server.
    ///
    /// Starts the API server and optionally the UI server (if the "ui" feature is enabled
    /// and UI configuration is present).
    pub async fn run(&self) -> Result<(), GatewayError> {
        let state = self.state.clone();

        // Check for bootstrap requirement
        {
            let state_read = state.read().await;
            let mut bootstrap = state_read.auth.bootstrap.write().await;
            if let Some(_token) = bootstrap.check_and_generate(&state_read.auth.users) {
                let base_url = format!("http://{}:{}", self.config.bind_address, self.config.port);
                bootstrap.print_bootstrap_info(&base_url);
            }
        }

        // Build API router
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/rpc", post(rpc_handler))
            .route("/ws", get(ws_handler))
            .with_state(state);

        let addr: SocketAddr = format!("{}:{}", self.config.bind_address, self.config.port)
            .parse()
            .map_err(|e| GatewayError::Config(format!("Invalid address: {e}")))?;

        tracing::info!("Gateway API listening on http://{}", addr);

        // Start API server
        let api_listener = tokio::net::TcpListener::bind(addr).await?;
        let api_handle = tokio::spawn(async move {
            axum::serve(api_listener, app).await
        });

        // Optionally start UI server
        #[cfg(feature = "ui")]
        let ui_handle = if let Some(ref ui_config) = self.config.ui {
            if ui_config.enabled {
                let config = ui_config.clone();
                Some(tokio::spawn(async move {
                    crate::ui_server::run_ui_server(config).await
                }))
            } else {
                None
            }
        } else {
            None
        };

        // Wait for servers to complete (or error)
        #[cfg(feature = "ui")]
        {
            tokio::select! {
                result = api_handle => {
                    result
                        .map_err(|e| GatewayError::Server(format!("API server panic: {e}")))?
                        .map_err(|e| GatewayError::Server(e.to_string()))?;
                }
                result = async {
                    match ui_handle {
                        Some(handle) => handle.await,
                        None => std::future::pending().await,
                    }
                } => {
                    result
                        .map_err(|e| GatewayError::Server(format!("UI server panic: {e}")))?
                        .map_err(|e| GatewayError::Server(e.to_string()))?;
                }
            }
        }

        #[cfg(not(feature = "ui"))]
        {
            api_handle
                .await
                .map_err(|e| GatewayError::Server(format!("API server panic: {e}")))?
                .map_err(|e| GatewayError::Server(e.to_string()))?;
        }

        Ok(())
    }
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn rpc_handler(
    State(state): State<Arc<RwLock<GatewayState>>>,
    headers: axum::http::HeaderMap,
    Json(request): Json<RpcRequest>,
) -> Json<RpcResponse> {
    let id = request.id.clone();

    // Extract auth token from header
    let auth_token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(JwtManager::extract_from_header);

    let result = dispatch_rpc(&state, &request.method, &request.params, auth_token).await;

    Json(match result {
        Ok(value) => RpcResponse::success(id, value),
        Err((code, message)) => RpcResponse::error(id, code, message),
    })
}

/// WebSocket query parameters.
#[derive(Debug, serde::Deserialize)]
struct WsParams {
    /// Auth token for WebSocket connection.
    token: Option<String>,
}

async fn ws_handler(
    State(state): State<Arc<RwLock<GatewayState>>>,
    axum::extract::Query(params): axum::extract::Query<WsParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, params.token))
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<RwLock<GatewayState>>,
    auth_token: Option<String>,
) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(tokio::sync::Mutex::new(sender));

    // Validate token if auth is required for WebSocket
    {
        let state_read = state.read().await;
        if state_read.auth.config.enabled && state_read.auth.config.require_auth_for_ws {
            match &auth_token {
                Some(token) => {
                    if let Err(e) = state_read.auth.validate_token(token) {
                        let error_response = RpcResponse::error(
                            None,
                            rpc::UNAUTHORIZED,
                            format!("Invalid token: {e}"),
                        );
                        let response_text = serde_json::to_string(&error_response).unwrap_or_default();
                        let mut guard = sender.lock().await;
                        let _ = guard.send(Message::Text(response_text.into())).await;
                        return;
                    }
                }
                None => {
                    let error_response = RpcResponse::error(
                        None,
                        rpc::UNAUTHORIZED,
                        "Authentication required".to_string(),
                    );
                    let response_text = serde_json::to_string(&error_response).unwrap_or_default();
                    let mut guard = sender.lock().await;
                    let _ = guard.send(Message::Text(response_text.into())).await;
                    return;
                }
            }
        }
    }

    // Create a channel to stop the event listener
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
    let subscribed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let subscribed_clone = subscribed.clone();
    let sender_clone = sender.clone();
    let state_clone = state.clone();

    // Spawn event listener task
    let event_task = tokio::spawn(async move {
        // Wait until subscribed
        loop {
            if subscribed_clone.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            if stop_rx.try_recv().is_ok() {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // Subscribe to events
        let mut event_rx = {
            let state_read = state_clone.read().await;
            state_read.events.subscribe()
        };

        loop {
            tokio::select! {
                _ = &mut stop_rx => {
                    break;
                }
                event_result = event_rx.recv() => {
                    match event_result {
                        Ok(envelope) => {
                            let event_msg = serde_json::json!({
                                "jsonrpc": "2.0",
                                "method": "event",
                                "params": envelope,
                            });
                            let msg_text = serde_json::to_string(&event_msg).unwrap_or_default();
                            let mut guard = sender_clone.lock().await;
                            if guard.send(Message::Text(msg_text.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("Event listener lagged, missed {} events", n);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Handle incoming RPC requests
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            Ok(_) => continue, // Ignore binary, ping, pong
            Err(e) => {
                tracing::warn!("WebSocket receive error: {}", e);
                break;
            }
        };

        // Parse JSON-RPC request
        let request: RpcRequest = match serde_json::from_str(&msg) {
            Ok(req) => req,
            Err(e) => {
                let error_response = RpcResponse::error(
                    None,
                    rpc::PARSE_ERROR,
                    format!("Parse error: {}", e),
                );
                let response_text = serde_json::to_string(&error_response).unwrap_or_default();
                let mut guard = sender.lock().await;
                if guard.send(Message::Text(response_text.into())).await.is_err() {
                    break;
                }
                continue;
            }
        };

        // Check for events.subscribe to enable event streaming
        if request.method == "events.subscribe" {
            subscribed.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        let id = request.id.clone();
        let token_ref = auth_token.as_deref();
        let result = dispatch_rpc(&state, &request.method, &request.params, token_ref).await;

        let response = match result {
            Ok(value) => RpcResponse::success(id, value),
            Err((code, message)) => RpcResponse::error(id, code, message),
        };

        let response_text = serde_json::to_string(&response).unwrap_or_default();
        let mut guard = sender.lock().await;
        if guard.send(Message::Text(response_text.into())).await.is_err() {
            break;
        }
    }

    // Stop event task
    let _ = stop_tx.send(());
    let _ = event_task.await;

    tracing::debug!("WebSocket connection closed");
}

/// Dispatch RPC request to appropriate handler.
async fn dispatch_rpc(
    state: &Arc<RwLock<GatewayState>>,
    method: &str,
    params: &serde_json::Value,
    auth_token: Option<&str>,
) -> RpcResult {
    let state_read = state.read().await;

    // Check if method requires auth
    if state_read.auth.requires_auth(method) {
        let token = auth_token.ok_or_else(|| {
            (rpc::UNAUTHORIZED, "Authentication required".to_string())
        })?;

        state_read.auth.validate_token(token).map_err(|e| {
            (rpc::UNAUTHORIZED, format!("Invalid token: {e}"))
        })?;
    }

    drop(state_read);

    match method {
        // Auth methods
        "auth.login" => handle_auth_login(state, params).await,
        "auth.logout" => handle_auth_logout(state, params).await,
        "auth.refresh" => handle_auth_refresh(state, params).await,
        "auth.me" => handle_auth_me(state, auth_token).await,

        // Setup methods
        "setup.status" => handle_setup_status(state).await,
        "setup.init" => handle_setup_init(state, params).await,

        // User management (admin only)
        "users.list" => handle_users_list(state, auth_token).await,
        "users.create" => handle_users_create(state, params, auth_token).await,
        "users.update" => handle_users_update(state, params, auth_token).await,
        "users.delete" => handle_users_delete(state, params, auth_token).await,

        // Session methods
        "session.create" => handle_session_create(state, params).await,
        "session.message" => handle_session_message(state, params).await,
        "session.history" => handle_session_history(state, params).await,
        "session.end" => handle_session_end(state, params).await,
        "session.list" => handle_session_list(state, params).await,
        "session.search" => handle_session_search(state, params).await,
        "session.stats" => handle_session_stats(state).await,
        "session.events" => handle_session_events(state, params).await,

        // Channel methods
        "channels.list" => handle_channels_list(state).await,
        "channels.status" => handle_channels_status(state).await,
        "channels.probe" => handle_channels_probe(state, params).await,

        // Agent methods
        "agent.list" => handle_agent_list(state).await,
        "agent.status" => handle_agent_status(state, params).await,
        "agent.get" => handle_agent_get(state, params).await,

        // Tool methods
        "tools.list" => handle_tools_list(state).await,
        "tools.execute" => handle_tools_execute(state, params).await,

        // System methods
        "system.health" => handle_system_health(state).await,
        "system.version" => handle_system_version().await,

        // Event subscription (WebSocket-only, returns ack)
        "events.subscribe" => handle_events_subscribe().await,

        _ => Err((rpc::METHOD_NOT_FOUND, format!("Method not found: {}", method))),
    }
}

type RpcResult = Result<serde_json::Value, (i32, String)>;

// ============================================================================
// Auth RPC Handlers
// ============================================================================

async fn handle_auth_login(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let username = params["username"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing username".to_string()))?;
    let password = params["password"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing password".to_string()))?;

    let state = state.read().await;

    // Find user
    let user = state
        .auth
        .users
        .get_by_username(username)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Storage error: {e}")))?
        .ok_or((rpc::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Check if active
    if !user.active {
        return Err((rpc::UNAUTHORIZED, "Account disabled".to_string()));
    }

    // Verify password
    user.verify_password(password)
        .map_err(|_| (rpc::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Update last login
    state
        .auth
        .users
        .update_last_login(&user.id)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to update login time: {e}")))?;

    // Generate tokens
    let token_pair = state
        .auth
        .jwt
        .create_token_pair(&user.id, &user.username, user.role)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Token generation failed: {e}")))?;

    Ok(serde_json::json!({
        "token": token_pair.access_token,
        "refresh_token": token_pair.refresh_token,
        "expires_at": token_pair.expires_at.to_rfc3339(),
        "user": user.to_public(),
    }))
}

async fn handle_auth_logout(
    _state: &Arc<RwLock<GatewayState>>,
    _params: &serde_json::Value,
) -> RpcResult {
    // For stateless JWT, logout is handled client-side by discarding the token
    // In a more complete implementation, we'd track refresh token families
    Ok(serde_json::json!({
        "success": true,
    }))
}

async fn handle_auth_refresh(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let refresh_token = params["refresh_token"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing refresh_token".to_string()))?;

    let state = state.read().await;

    let token_pair = state
        .auth
        .jwt
        .refresh_tokens(refresh_token)
        .map_err(|e| (rpc::UNAUTHORIZED, format!("Refresh failed: {e}")))?;

    Ok(serde_json::json!({
        "token": token_pair.access_token,
        "refresh_token": token_pair.refresh_token,
        "expires_at": token_pair.expires_at.to_rfc3339(),
    }))
}

async fn handle_auth_me(
    state: &Arc<RwLock<GatewayState>>,
    auth_token: Option<&str>,
) -> RpcResult {
    let token = auth_token.ok_or((rpc::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let state = state.read().await;
    let claims = state
        .auth
        .validate_token(token)
        .map_err(|e| (rpc::UNAUTHORIZED, format!("Invalid token: {e}")))?;

    let user = state
        .auth
        .users
        .get(&claims.sub)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Storage error: {e}")))?
        .ok_or((rpc::NOT_FOUND, "User not found".to_string()))?;

    Ok(serde_json::json!({
        "user": user.to_public(),
    }))
}

// ============================================================================
// Setup RPC Handlers
// ============================================================================

async fn handle_setup_status(state: &Arc<RwLock<GatewayState>>) -> RpcResult {
    let state = state.read().await;
    let bootstrap = state.auth.bootstrap.read().await;

    let base_url = format!(
        "http://{}:{}",
        state.config.bind_address, state.config.port
    );

    let status = bootstrap.status(&state.auth.users, Some(&base_url));

    Ok(serde_json::to_value(&status)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Serialization error: {e}")))?)
}

async fn handle_setup_init(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let bootstrap_token = params["bootstrap_token"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing bootstrap_token".to_string()))?;
    let username = params["admin_username"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing admin_username".to_string()))?;
    let password = params["admin_password"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing admin_password".to_string()))?;
    let email = params["email"].as_str().map(String::from);

    let state = state.read().await;
    let mut bootstrap = state.auth.bootstrap.write().await;

    let admin = bootstrap
        .complete_setup(&state.auth.users, bootstrap_token, username, password, email)
        .map_err(|e| (rpc::UNAUTHORIZED, format!("Setup failed: {e}")))?;

    // Generate tokens for the new admin
    let token_pair = state
        .auth
        .jwt
        .create_token_pair(&admin.id, &admin.username, admin.role)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Token generation failed: {e}")))?;

    Ok(serde_json::json!({
        "token": token_pair.access_token,
        "refresh_token": token_pair.refresh_token,
        "expires_at": token_pair.expires_at.to_rfc3339(),
        "user": admin.to_public(),
    }))
}

// ============================================================================
// User Management RPC Handlers (Admin only)
// ============================================================================

fn require_admin(state: &GatewayState, token: Option<&str>) -> Result<(), (i32, String)> {
    let token = token.ok_or((rpc::UNAUTHORIZED, "Not authenticated".to_string()))?;
    let claims = state
        .auth
        .validate_token(token)
        .map_err(|e| (rpc::UNAUTHORIZED, format!("Invalid token: {e}")))?;

    if !claims.role.is_admin() {
        return Err((rpc::FORBIDDEN, "Admin role required".to_string()));
    }

    Ok(())
}

async fn handle_users_list(
    state: &Arc<RwLock<GatewayState>>,
    auth_token: Option<&str>,
) -> RpcResult {
    let state = state.read().await;
    require_admin(&state, auth_token)?;

    let users = state
        .auth
        .users
        .list()
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Storage error: {e}")))?;

    let public_users: Vec<_> = users.iter().map(User::to_public).collect();

    Ok(serde_json::json!({
        "users": public_users,
        "total": public_users.len(),
    }))
}

async fn handle_users_create(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
    auth_token: Option<&str>,
) -> RpcResult {
    let state = state.read().await;
    require_admin(&state, auth_token)?;

    let username = params["username"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing username".to_string()))?;
    let password = params["password"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing password".to_string()))?;
    let role_str = params["role"].as_str().unwrap_or("viewer");
    let email = params["email"].as_str().map(String::from);

    let role: UserRole = role_str
        .parse()
        .map_err(|e| (rpc::INVALID_PARAMS, format!("Invalid role: {e}")))?;

    let mut user = User::new(username, password, role)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("User creation failed: {e}")))?;
    user.email = email;

    state
        .auth
        .users
        .create(&user)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Storage error: {e}")))?;

    Ok(serde_json::json!({
        "user": user.to_public(),
    }))
}

async fn handle_users_update(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
    auth_token: Option<&str>,
) -> RpcResult {
    let state = state.read().await;
    require_admin(&state, auth_token)?;

    let id = params["id"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing id".to_string()))?;

    let mut user = state
        .auth
        .users
        .get(id)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Storage error: {e}")))?
        .ok_or((rpc::NOT_FOUND, format!("User not found: {id}")))?;

    // Update fields if provided
    if let Some(role_str) = params["role"].as_str() {
        user.role = role_str
            .parse()
            .map_err(|e| (rpc::INVALID_PARAMS, format!("Invalid role: {e}")))?;
    }

    if let Some(active) = params["active"].as_bool() {
        user.active = active;
    }

    if let Some(email) = params["email"].as_str() {
        user.email = Some(email.to_string());
    }

    state
        .auth
        .users
        .update(&user)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Storage error: {e}")))?;

    Ok(serde_json::json!({
        "user": user.to_public(),
    }))
}

async fn handle_users_delete(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
    auth_token: Option<&str>,
) -> RpcResult {
    let state = state.read().await;
    require_admin(&state, auth_token)?;

    let id = params["id"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing id".to_string()))?;

    // Prevent deleting the last admin
    let users = state
        .auth
        .users
        .list()
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Storage error: {e}")))?;

    let admin_count = users.iter().filter(|u| u.role.is_admin() && u.active).count();
    let target_user = users.iter().find(|u| u.id == id);

    if let Some(user) = target_user {
        if user.role.is_admin() && admin_count <= 1 {
            return Err((rpc::FORBIDDEN, "Cannot delete the last admin".to_string()));
        }
    }

    let deleted = state
        .auth
        .users
        .delete(id)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Storage error: {e}")))?;

    Ok(serde_json::json!({
        "success": deleted,
    }))
}

// ============================================================================
// System RPC Handlers
// ============================================================================

async fn handle_system_health(state: &Arc<RwLock<GatewayState>>) -> RpcResult {
    let state = state.read().await;

    Ok(serde_json::json!({
        "status": "healthy",
        "auth_enabled": state.auth.config.enabled,
        "users_configured": !state.auth.users.is_empty(),
        "agents_count": state.agents.len(),
    }))
}

async fn handle_system_version() -> RpcResult {
    Ok(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "name": "openclaw-gateway",
    }))
}

/// Handle event subscription request.
/// This is a WebSocket-only method that returns an ack.
/// The actual event streaming is handled separately.
async fn handle_events_subscribe() -> RpcResult {
    Ok(serde_json::json!({
        "subscribed": true,
        "message": "Events will be pushed to this connection",
    }))
}

// ============================================================================
// Session RPC Handlers
// ============================================================================

async fn handle_session_create(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let agent_id = params["agent_id"]
        .as_str()
        .unwrap_or("default")
        .to_string();
    let channel = params["channel"]
        .as_str()
        .unwrap_or("api")
        .to_string();
    let peer_id = params["peer_id"]
        .as_str()
        .unwrap_or("anonymous")
        .to_string();

    let session_key = SessionKey::build(
        &AgentId::new(&agent_id),
        &ChannelId::new(&channel),
        "gateway",
        openclaw_core::types::PeerType::Dm,
        &openclaw_core::types::PeerId::new(&peer_id),
    );

    let event = SessionEvent::new(
        session_key.clone(),
        agent_id.clone(),
        SessionEventKind::SessionStarted {
            channel: channel.clone(),
            peer_id: peer_id.clone(),
        },
    );

    let state = state.read().await;
    state
        .event_store
        .append(&event)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to create session: {e}")))?;

    Ok(serde_json::json!({
        "session_key": session_key.as_ref(),
        "agent_id": agent_id,
        "channel": channel,
        "peer_id": peer_id,
    }))
}

async fn handle_session_message(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let session_key_str = params["session_key"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing session_key".to_string()))?;
    let message = params["message"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing message".to_string()))?;
    let agent_id_str = params["agent_id"]
        .as_str()
        .unwrap_or("default");

    let session_key = SessionKey::new(session_key_str);
    let state = state.read().await;

    // Log inbound message
    let recv_event = SessionEvent::new(
        session_key.clone(),
        agent_id_str.to_string(),
        SessionEventKind::MessageReceived {
            content: message.to_string(),
            attachments: vec![],
        },
    );
    state
        .event_store
        .append(&recv_event)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to log message: {e}")))?;

    // Get agent runtime
    let agent = state
        .agents
        .get(agent_id_str)
        .ok_or((rpc::INVALID_PARAMS, format!("Agent not found: {agent_id_str}")))?;

    // Get or create session projection
    let projection = state
        .event_store
        .get_projection(&session_key)
        .unwrap_or_else(|_| {
            SessionProjection::new(
                session_key.clone(),
                agent_id_str.to_string(),
                ChannelId::new("api"),
                "anonymous".to_string(),
            )
        });

    // Build agent context and process
    let mut ctx = AgentContext::new(
        AgentId::new(agent_id_str),
        session_key.clone(),
        projection,
        state.tool_registry.clone(),
    );

    let response = agent
        .process_message(&mut ctx, message)
        .await
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Agent error: {e}")))?;

    // Log agent response
    let resp_event = SessionEvent::new(
        session_key,
        agent_id_str.to_string(),
        SessionEventKind::AgentResponse {
            content: response.clone(),
            model: String::new(),
            tokens: openclaw_core::types::TokenUsage::default(),
        },
    );
    state
        .event_store
        .append(&resp_event)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to log response: {e}")))?;

    Ok(serde_json::json!({
        "response": response,
    }))
}

async fn handle_session_history(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let session_key_str = params["session_key"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing session_key".to_string()))?;

    let session_key = SessionKey::new(session_key_str);
    let state = state.read().await;

    let projection = state
        .event_store
        .get_projection(&session_key)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to get session: {e}")))?;

    Ok(serde_json::to_value(&projection)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Serialization error: {e}")))?)
}

async fn handle_session_end(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let session_key_str = params["session_key"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing session_key".to_string()))?;
    let reason = params["reason"]
        .as_str()
        .unwrap_or("user_requested")
        .to_string();

    let session_key = SessionKey::new(session_key_str);
    let state = state.read().await;

    let event = SessionEvent::new(
        session_key,
        "gateway".to_string(),
        SessionEventKind::SessionEnded { reason: reason.clone() },
    );

    state
        .event_store
        .append(&event)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to end session: {e}")))?;

    Ok(serde_json::json!({
        "status": "ended",
        "reason": reason,
    }))
}

/// Extended session list with filtering and pagination.
async fn handle_session_list(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let limit = params["limit"].as_u64().unwrap_or(50) as usize;
    let offset = params["offset"].as_u64().unwrap_or(0) as usize;
    let filter_channel = params["channel"].as_str();
    let filter_agent = params["agent"].as_str();
    let filter_state = params["state"].as_str();

    let state = state.read().await;
    let session_keys = state
        .event_store
        .list_sessions()
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to list sessions: {e}")))?;

    // Get projections and apply filters
    let mut sessions: Vec<SessionProjection> = session_keys
        .into_iter()
        .filter_map(|key| state.event_store.get_projection(&key).ok())
        .filter(|p| {
            // Apply filters
            if let Some(ch) = filter_channel {
                if p.channel.as_ref() != ch {
                    return false;
                }
            }
            if let Some(agent) = filter_agent {
                if p.agent_id != agent {
                    return false;
                }
            }
            if let Some(st) = filter_state {
                let state_match = match st {
                    "active" => p.state == SessionState::Active,
                    "paused" => p.state == SessionState::Paused,
                    "ended" => p.state == SessionState::Ended,
                    _ => true,
                };
                if !state_match {
                    return false;
                }
            }
            true
        })
        .collect();

    // Sort by last activity (most recent first)
    sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

    let total = sessions.len();

    // Apply pagination
    let sessions: Vec<_> = sessions.into_iter().skip(offset).take(limit).collect();

    Ok(serde_json::json!({
        "sessions": sessions,
        "total": total,
        "limit": limit,
        "offset": offset,
    }))
}

/// Search sessions by query.
async fn handle_session_search(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let query = params["query"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing query".to_string()))?
        .to_lowercase();
    let filter_channel = params["channel"].as_str();
    let filter_agent = params["agent"].as_str();
    let limit = params["limit"].as_u64().unwrap_or(20) as usize;

    let state = state.read().await;
    let session_keys = state
        .event_store
        .list_sessions()
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to list sessions: {e}")))?;

    // Search through sessions
    let mut results: Vec<SessionProjection> = session_keys
        .into_iter()
        .filter_map(|key| state.event_store.get_projection(&key).ok())
        .filter(|p| {
            // Apply channel/agent filters
            if let Some(ch) = filter_channel {
                if p.channel.as_ref() != ch {
                    return false;
                }
            }
            if let Some(agent) = filter_agent {
                if p.agent_id != agent {
                    return false;
                }
            }

            // Search in peer_id
            if p.peer_id.to_lowercase().contains(&query) {
                return true;
            }

            // Search in session key
            if p.session_key.as_ref().to_lowercase().contains(&query) {
                return true;
            }

            // Search in message content
            for msg in &p.messages {
                let content = match msg {
                    SessionMessage::Inbound(c) | SessionMessage::Outbound(c) => c,
                    SessionMessage::Tool { result, .. } => result,
                };
                if content.to_lowercase().contains(&query) {
                    return true;
                }
            }

            false
        })
        .collect();

    // Sort by relevance (for now, just by last activity)
    results.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
    results.truncate(limit);

    Ok(serde_json::json!({
        "sessions": results,
        "count": results.len(),
        "query": query,
    }))
}

/// Get session statistics.
async fn handle_session_stats(state: &Arc<RwLock<GatewayState>>) -> RpcResult {
    let state = state.read().await;
    let session_keys = state
        .event_store
        .list_sessions()
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to list sessions: {e}")))?;

    let mut total = 0;
    let mut active = 0;
    let mut by_channel: HashMap<String, u64> = HashMap::new();
    let mut by_agent: HashMap<String, u64> = HashMap::new();
    let mut total_messages: u64 = 0;

    for key in session_keys {
        if let Ok(projection) = state.event_store.get_projection(&key) {
            total += 1;
            if projection.state == SessionState::Active {
                active += 1;
            }
            *by_channel
                .entry(projection.channel.as_ref().to_string())
                .or_insert(0) += 1;
            *by_agent.entry(projection.agent_id.clone()).or_insert(0) += 1;
            total_messages += projection.message_count;
        }
    }

    Ok(serde_json::json!({
        "total": total,
        "active": active,
        "by_channel": by_channel,
        "by_agent": by_agent,
        "total_messages": total_messages,
    }))
}

/// Get events for a session.
async fn handle_session_events(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let session_key_str = params["session_key"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing session_key".to_string()))?;
    let since = params["since"].as_str().and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    });

    let session_key = SessionKey::new(session_key_str);
    let state = state.read().await;

    let events = if let Some(since_time) = since {
        state
            .event_store
            .get_events_since(&session_key, since_time)
            .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to get events: {e}")))?
    } else {
        state
            .event_store
            .get_events(&session_key)
            .map_err(|e| (rpc::INTERNAL_ERROR, format!("Failed to get events: {e}")))?
    };

    Ok(serde_json::json!({
        "events": events,
        "count": events.len(),
    }))
}

// ============================================================================
// Channel RPC Handlers
// ============================================================================

/// Channel info for API responses.
#[derive(Debug, Clone, serde::Serialize)]
struct ChannelInfo {
    id: String,
    label: String,
    capabilities: ChannelCapabilities,
}

async fn handle_channels_list(state: &Arc<RwLock<GatewayState>>) -> RpcResult {
    let state = state.read().await;
    let registry = state.channels.read().await;

    let channels: Vec<String> = registry.list().iter().map(|s| s.to_string()).collect();

    Ok(serde_json::json!({
        "channels": channels,
        "count": channels.len(),
    }))
}

async fn handle_channels_status(state: &Arc<RwLock<GatewayState>>) -> RpcResult {
    let state = state.read().await;
    let registry = state.channels.read().await;

    let probes = registry.probe_all().await;

    let statuses: HashMap<String, serde_json::Value> = probes
        .into_iter()
        .map(|(id, result)| {
            let status = match result {
                Ok(probe) => serde_json::json!({
                    "connected": probe.connected,
                    "account_id": probe.account_id,
                    "display_name": probe.display_name,
                    "error": probe.error,
                }),
                Err(e) => serde_json::json!({
                    "connected": false,
                    "error": e.to_string(),
                }),
            };
            (id, status)
        })
        .collect();

    Ok(serde_json::json!({
        "statuses": statuses,
    }))
}

async fn handle_channels_probe(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let channel_id = params["channel_id"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing channel_id".to_string()))?;

    let state = state.read().await;
    let registry = state.channels.read().await;

    let channel = registry
        .get(channel_id)
        .ok_or((rpc::NOT_FOUND, format!("Channel not found: {channel_id}")))?;

    let probe = channel
        .probe()
        .await
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Probe failed: {e}")))?;

    Ok(serde_json::json!({
        "channel_id": channel_id,
        "connected": probe.connected,
        "account_id": probe.account_id,
        "display_name": probe.display_name,
        "error": probe.error,
    }))
}

// ============================================================================
// Agent RPC Handlers
// ============================================================================

async fn handle_agent_list(state: &Arc<RwLock<GatewayState>>) -> RpcResult {
    let state = state.read().await;
    let agents: Vec<&str> = state.agents.keys().map(String::as_str).collect();

    Ok(serde_json::json!({
        "agents": agents,
    }))
}

async fn handle_agent_get(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let agent_id = params["agent_id"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing agent_id".to_string()))?;

    let state = state.read().await;
    let agent = state
        .agents
        .get(agent_id)
        .ok_or((rpc::NOT_FOUND, format!("Agent not found: {agent_id}")))?;

    // Return agent info
    Ok(serde_json::json!({
        "agent_id": agent_id,
        "available": true,
        "config": {
            "model": agent.model(),
            "system_prompt": agent.system_prompt(),
            "max_tokens": agent.max_tokens(),
            "temperature": agent.temperature(),
        },
    }))
}

async fn handle_agent_status(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let agent_id = params["agent_id"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing agent_id".to_string()))?;

    let state = state.read().await;
    let exists = state.agents.contains_key(agent_id);

    Ok(serde_json::json!({
        "agent_id": agent_id,
        "available": exists,
    }))
}

async fn handle_tools_list(state: &Arc<RwLock<GatewayState>>) -> RpcResult {
    let state = state.read().await;
    let tools: Vec<serde_json::Value> = state
        .tool_registry
        .as_tool_definitions()
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.input_schema,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "tools": tools,
    }))
}

async fn handle_tools_execute(
    state: &Arc<RwLock<GatewayState>>,
    params: &serde_json::Value,
) -> RpcResult {
    let tool_name = params["tool_name"]
        .as_str()
        .ok_or((rpc::INVALID_PARAMS, "Missing tool_name".to_string()))?;
    let tool_params = params.get("params").cloned().unwrap_or(serde_json::json!({}));

    let state = state.read().await;
    let result = state
        .tool_registry
        .execute(tool_name, tool_params)
        .await
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Tool error: {e}")))?;

    Ok(serde_json::to_value(&result)
        .map_err(|e| (rpc::INTERNAL_ERROR, format!("Serialization error: {e}")))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GatewayConfig::default();
        assert_eq!(config.port, 18789);
        assert_eq!(config.bind_address, "127.0.0.1");
    }

    #[test]
    fn test_builder() {
        let temp_dir = std::env::temp_dir().join("openclaw-gateway-test");
        let store = Arc::new(EventStore::open(&temp_dir).unwrap());

        let gateway = GatewayBuilder::new()
            .with_event_store(store)
            .build();

        assert!(gateway.is_ok());
    }
}
